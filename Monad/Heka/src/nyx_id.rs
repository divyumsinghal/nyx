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
//! 1. User completes Kratos registration with email + password
//! 2. Kratos webhooks notify Nyx backend of new identity
//! 3. Backend prompts user to choose Nyx ID via settings flow
//! 4. NyxIdRegistry validates uniqueness and reserves the ID
//! 5. Identity is marked "complete" and can fully use the platform
//!
//! # Security
//!
//! - All operations are idempotent where possible
//! - Race conditions handled via PostgreSQL UPSERT/locking
//! - Cache layer (DragonflyDB) for read-heavy existence checks
//! - Rate limiting on ID availability checks (prevent enumeration attacks)

use anyhow::{Context, Result};
use nun::IdentityId;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info, warn};

/// Rules for valid Nyx IDs.
pub const NYX_ID_MIN_LENGTH: usize = 3;
pub const NYX_ID_MAX_LENGTH: usize = 32;
pub const NYX_ID_PATTERN: &str = r"^[a-zA-Z0-9_]+$";

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
    pub async fn reserve(
        &self,
        identity_id: &IdentityId,
        nyx_id: &str,
    ) -> Result<()> {
        // Validate format first
        if let Err(reason) = Self::validate_format(nyx_id) {
            anyhow::bail!("Invalid Nyx ID format: {}", reason);
        }

        let nyx_id_lower = nyx_id.to_lowercase();

        // Attempt to insert the Nyx ID
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
            // Check if it's the same identity (idempotent) or a conflict
            let existing: Option<(String,)> = sqlx::query_as(
                r#"
                SELECT id FROM nyx.identities WHERE nyx_id = $1
                "#,
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
    ///
    /// Returns error if the new ID is already taken.
    pub async fn update(
        &self,
        identity_id: &IdentityId,
        new_nyx_id: &str,
    ) -> Result<()> {
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
    pub async fn lookup_by_nyx_id(&self, nyx_id: &str) -> Result<Option<IdentityId>> {
        let nyx_id_lower = nyx_id.to_lowercase();

        let result: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT id FROM nyx.identities WHERE nyx_id = $1
            "#,
        )
        .bind(&nyx_id_lower)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to lookup identity by Nyx ID")?;

        Ok(result.map(|(id_str,)| {
            id_str.parse().expect("Stored identity ID should be valid UUID")
        }))
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
    /// - 3-32 characters
    /// - Alphanumeric and underscores only
    /// - Must start with a letter
    /// - Cannot be all numbers
    fn validate_format(nyx_id: &str) -> Result<(), String> {
        let len = nyx_id.len();

        if len < NYX_ID_MIN_LENGTH {
            return Err(format!(
                "Nyx ID must be at least {} characters",
                NYX_ID_MIN_LENGTH
            ));
        }

        if len > NYX_ID_MAX_LENGTH {
            return Err(format!(
                "Nyx ID must be at most {} characters",
                NYX_ID_MAX_LENGTH
            ));
        }

        // Check first character is a letter
        let first_char = nyx_id.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return Err("Nyx ID must start with a letter".to_string());
        }

        // Check pattern (alphanumeric + underscore)
        let pattern = regex::Regex::new(NYX_ID_PATTERN).unwrap();
        if !pattern.is_match(nyx_id) {
            return Err(
                "Nyx ID can only contain letters, numbers, and underscores".to_string(),
            );
        }

        // Cannot be all numbers
        if nyx_id.chars().all(|c| c.is_ascii_digit()) {
            return Err("Nyx ID cannot be all numbers".to_string());
        }

        // Reserved names check
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

        // Invalid: too short
        assert!(NyxIdRegistry::validate_format("ab").is_err());

        // Invalid: starts with number
        assert!(NyxIdRegistry::validate_format("1alice").is_err());

        // Invalid: all numbers
        assert!(NyxIdRegistry::validate_format("12345").is_err());

        // Invalid: special characters
        assert!(NyxIdRegistry::validate_format("alice.bob").is_err());
        assert!(NyxIdRegistry::validate_format("alice@bob").is_err());

        // Invalid: reserved
        assert!(NyxIdRegistry::validate_format("admin").is_err());
        assert!(NyxIdRegistry::validate_format("root").is_err());
    }
}
