//! Nyx ID registry — platform-wide unique handle management.
//!
//! The Nyx ID is the user's public-facing identifier (like @username on Instagram).
//! It must be:
//! - Unique across the entire platform (enforced by PostgreSQL UNIQUE constraint)
//! - Chosen by the user during registration completion
//! - Valid for login (alternative to email)
//!
//! # Registration Flow
//!
//! 1. User completes Kratos registration with email + OTP
//! 2. Backend prompts user to choose a Nyx ID via the registration flow
//! 3. NyxIdRegistry validates uniqueness and reserves the ID
//! 4. Identity is marked "complete" and can fully use the platform
//!
//! # Security
//!
//! - All operations are idempotent where possible
//! - Race conditions handled via PostgreSQL UPSERT/locking
//! - Rate limiting on ID availability checks (prevent enumeration attacks)

use std::sync::OnceLock;

use anyhow::{Context, Result};
use nun::IdentityId;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info, warn};

/// Rules for valid Nyx IDs.
/// Must be kept in sync with `Prithvi/config/kratos/identity.schema.json`.
pub const NYX_ID_MIN_LENGTH: usize = 3;
pub const NYX_ID_MAX_LENGTH: usize = 32;

/// Pattern: ASCII letters, digits, underscores only (no dots, no hyphens).
/// The schema enforces the same pattern — they must stay aligned.
pub const NYX_ID_PATTERN: &str = r"^[a-zA-Z0-9_]+$";

/// Compiled regex for `NYX_ID_PATTERN`, initialised once on first use.
static NYX_ID_RE: OnceLock<Regex> = OnceLock::new();

fn nyx_id_regex() -> &'static Regex {
    NYX_ID_RE.get_or_init(|| {
        Regex::new(NYX_ID_PATTERN).expect("NYX_ID_PATTERN is a valid regex literal")
    })
}

/// Nyx ID validation and uniqueness check result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NyxIdStatus {
    /// ID is available and valid
    Available,
    /// ID is already taken by another user
    Taken,
    /// ID format is invalid
    Invalid { reason: String },
}

/// Nyx ID registry for managing unique platform handles.
#[derive(Clone)]
pub struct NyxIdRegistry {
    pool: PgPool,
}

impl NyxIdRegistry {
    /// Create a new registry instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if a Nyx ID is available (valid format + not taken).
    ///
    /// This is the endpoint called by the frontend during registration
    /// to show real-time availability (with rate limiting).
    pub async fn check_availability(&self, nyx_id: &str) -> Result<NyxIdStatus> {
        // First validate format
        if let Err(reason) = Self::validate_format(nyx_id) {
            return Ok(NyxIdStatus::Invalid { reason });
        }

        // Check database for existence
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM nyx.identities
                WHERE nyx_id = $1
            )
            "#,
        )
        .bind(nyx_id.to_lowercase())
        .fetch_one(&self.pool)
        .await
        .context("Failed to check Nyx ID availability")?;

        if exists {
            Ok(NyxIdStatus::Taken)
        } else {
            Ok(NyxIdStatus::Available)
        }
    }

    /// Reserve a Nyx ID for a new identity.
    ///
    /// This is called during the registration completion flow.
    /// Uses UPSERT semantics to handle races gracefully.
    pub async fn reserve(&self, identity_id: &IdentityId, nyx_id: &str) -> Result<()> {
        if let Err(reason) = Self::validate_format(nyx_id) {
            anyhow::bail!("Invalid Nyx ID format: {}", reason);
        }

        let nyx_id_lower = nyx_id.to_lowercase();

        let result = sqlx::query(
            r#"
            INSERT INTO nyx.identities (id, nyx_id, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (nyx_id) DO NOTHING
            "#,
        )
        .bind(identity_id.to_string())
        .bind(&nyx_id_lower)
        .execute(&self.pool)
        .await
        .context("Failed to reserve Nyx ID")?;

        if result.rows_affected() == 0 {
            // Check if it's the same identity (idempotent) or a real conflict.
            let existing: Option<(String,)> = sqlx::query_as(
                r#"SELECT id FROM nyx.identities WHERE nyx_id = $1"#,
            )
            .bind(&nyx_id_lower)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to check existing Nyx ID reservation")?;

            match existing {
                Some((existing_id,)) if existing_id == identity_id.to_string() => {
                    debug!("Nyx ID {} already reserved for this identity", nyx_id);
                    Ok(())
                }
                _ => {
                    warn!("Nyx ID {} is already taken by another user", nyx_id);
                    anyhow::bail!("Nyx ID '{}' is already taken", nyx_id);
                }
            }
        } else {
            info!(
                identity_id = %identity_id,
                nyx_id = %nyx_id_lower,
                "Reserved Nyx ID for identity"
            );
            Ok(())
        }
    }

    /// Update an existing identity's Nyx ID (e.g., username change).
    pub async fn update(&self, identity_id: &IdentityId, new_nyx_id: &str) -> Result<()> {
        if let Err(reason) = Self::validate_format(new_nyx_id) {
            anyhow::bail!("Invalid Nyx ID format: {}", reason);
        }

        let new_nyx_id_lower = new_nyx_id.to_lowercase();

        let result = sqlx::query(
            r#"
            UPDATE nyx.identities
            SET nyx_id = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&new_nyx_id_lower)
        .bind(identity_id.to_string())
        .execute(&self.pool)
        .await
        .context("Failed to update Nyx ID")?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Identity not found");
        }

        info!(
            identity_id = %identity_id,
            new_nyx_id = %new_nyx_id_lower,
            "Updated Nyx ID for identity"
        );
        Ok(())
    }

    /// Look up an identity by their Nyx ID (for login).
    ///
    /// Returns the Kratos identity ID associated with this Nyx ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails or if a stored identity ID
    /// cannot be parsed as a UUID (data corruption guard — does not panic).
    pub async fn lookup_by_nyx_id(&self, nyx_id: &str) -> Result<Option<IdentityId>> {
        let nyx_id_lower = nyx_id.to_lowercase();

        let result: Option<(String,)> = sqlx::query_as(
            r#"SELECT id FROM nyx.identities WHERE nyx_id = $1"#,
        )
        .bind(&nyx_id_lower)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to lookup identity by Nyx ID")?;

        // Use transpose() instead of expect() — a malformed UUID in the DB
        // returns a proper error instead of panicking.
        result
            .map(|(id_str,)| {
                id_str
                    .parse::<IdentityId>()
                    .with_context(|| {
                        format!("Identity in database has malformed UUID: {id_str}")
                    })
            })
            .transpose()
    }

    /// Release a Nyx ID (e.g., on account deletion).
    pub async fn release(&self, identity_id: &IdentityId) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE nyx.identities
            SET nyx_id = NULL, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(identity_id.to_string())
        .execute(&self.pool)
        .await
        .context("Failed to release Nyx ID")?;

        info!(identity_id = %identity_id, "Released Nyx ID");
        Ok(())
    }

    /// Validate Nyx ID format.
    ///
    /// Rules:
    /// - 3–32 characters
    /// - ASCII letters, digits, and underscores only
    /// - Must start with a letter
    /// - Cannot be all digits
    /// - Reserved names are blocked
    pub fn validate_format(nyx_id: &str) -> Result<(), String> {
        let len = nyx_id.len();

        if len < NYX_ID_MIN_LENGTH {
            return Err(format!(
                "Nyx ID must be at least {NYX_ID_MIN_LENGTH} characters"
            ));
        }

        if len > NYX_ID_MAX_LENGTH {
            return Err(format!(
                "Nyx ID must be at most {NYX_ID_MAX_LENGTH} characters"
            ));
        }

        // First character must be a letter.
        let first_char = nyx_id.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return Err("Nyx ID must start with a letter".to_string());
        }

        // Must match the allowed character set (uses pre-compiled static regex).
        if !nyx_id_regex().is_match(nyx_id) {
            return Err(
                "Nyx ID can only contain letters, numbers, and underscores".to_string(),
            );
        }

        // Cannot be all digits (the first-char check handles this, but be explicit).
        if nyx_id.chars().all(|c| c.is_ascii_digit()) {
            return Err("Nyx ID cannot be all numbers".to_string());
        }

        // Reserved names.
        let reserved = ["admin", "root", "system", "api", "nyx", "test"];
        if reserved.contains(&nyx_id.to_lowercase().as_str()) {
            return Err("This Nyx ID is reserved".to_string());
        }

        Ok(())
    }
}

/// Validate a Nyx ID string using the platform's canonical rules.
pub fn validate_nyx_id(nyx_id: &str) -> Result<(), String> {
    NyxIdRegistry::validate_format(nyx_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_nyx_id_format() {
        // Valid IDs
        assert!(NyxIdRegistry::validate_format("alice").is_ok());
        assert!(NyxIdRegistry::validate_format("alice_123").is_ok());
        assert!(NyxIdRegistry::validate_format("a_1").is_ok());
        assert!(NyxIdRegistry::validate_format(&"a".repeat(32)).is_ok());

        // Invalid: too short
        assert!(NyxIdRegistry::validate_format("ab").is_err());

        // Invalid: too long (33 chars)
        assert!(NyxIdRegistry::validate_format(&"a".repeat(33)).is_err());

        // Exactly at the limit (32 chars) — should pass
        assert!(NyxIdRegistry::validate_format(&"a".repeat(32)).is_ok());

        // Invalid: starts with number
        assert!(NyxIdRegistry::validate_format("1alice").is_err());

        // Invalid: special characters
        assert!(NyxIdRegistry::validate_format("alice.bob").is_err());
        assert!(NyxIdRegistry::validate_format("alice@bob").is_err());
        assert!(NyxIdRegistry::validate_format("alice-bob").is_err());

        // Invalid: reserved
        assert!(NyxIdRegistry::validate_format("admin").is_err());
        assert!(NyxIdRegistry::validate_format("root").is_err());
    }
}
