//! Validation functions for platform-universal field formats.
//!
//! These validators cover fields that appear across multiple Nyx apps. They
//! validate **format** (what makes a valid alias?) not **constraints** (how
//! long can a caption be?) — constraints are app-specific and should use the
//! `validator` crate's built-in `#[validate(length(...))]`.
//!
//! # Scope
//!
//! | Validator        | Used by                          |
//! |------------------|----------------------------------|
//! | `phone`          | Kratos identity (all apps)       |
//! | `email`          | Kratos identity (all apps)       |
//! | `alias`          | App-scoped usernames (all apps)  |
//! | `display_name`   | Profile display names (all apps) |
//! | `hashtag`        | Content tagging (Uzume, Themis)  |
//!
//! # Usage
//!
//! ```rust
//! use nun::validation;
//!
//! assert!(validation::alias("cool_user_42").is_ok());
//! assert!(validation::alias("ab").is_err()); // too short
//! ```

use crate::error::{FieldError, NyxError, Result};
use crate::LinkPolicy;

// ── Phone ───────────────────────────────────────────────────────────────────

/// Validate an E.164 phone number.
///
/// E.164 format: `+` followed by 1–15 digits, no spaces or dashes.
/// Examples: `+14155551234`, `+353861234567`.
///
/// This validates format only — it does not verify the number is reachable.
pub fn phone(phone: &str) -> Result<()> {
    let bytes = phone.as_bytes();

    if bytes.first() != Some(&b'+') {
        return Err(field_error(
            "phone",
            "invalid_format",
            "Must start with '+'",
        ));
    }

    let digits = &phone[1..];

    if digits.is_empty() || digits.len() > 15 {
        return Err(field_error(
            "phone",
            "invalid_length",
            "Must have 1–15 digits after '+'",
        ));
    }

    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(field_error(
            "phone",
            "invalid_format",
            "Must contain only digits after '+'",
        ));
    }

    Ok(())
}

// ── Email ───────────────────────────────────────────────────────────────────

/// Validate an email address (basic format check).
///
/// Checks for: non-empty local part, `@` separator, non-empty domain with at
/// least one dot. This is intentionally permissive — the true validation is
/// the verification email that Kratos sends.
pub fn email(email: &str) -> Result<()> {
    let email = email.trim();

    if email.len() > 254 {
        return Err(field_error(
            "email",
            "too_long",
            "Email must be at most 254 characters",
        ));
    }

    let Some((local, domain)) = email.rsplit_once('@') else {
        return Err(field_error("email", "invalid_format", "Must contain '@'"));
    };

    if local.is_empty() {
        return Err(field_error(
            "email",
            "invalid_format",
            "Local part (before @) must not be empty",
        ));
    }

    if domain.is_empty() || !domain.contains('.') {
        return Err(field_error(
            "email",
            "invalid_format",
            "Domain must contain at least one '.'",
        ));
    }

    // Domain labels must not start or end with hyphens
    for label in domain.split('.') {
        if label.is_empty() || label.starts_with('-') || label.ends_with('-') {
            return Err(field_error(
                "email",
                "invalid_format",
                "Invalid domain format",
            ));
        }
    }

    Ok(())
}

// ── Alias ───────────────────────────────────────────────────────────────────

/// Minimum alias length.
pub const ALIAS_MIN_LEN: usize = 3;
/// Maximum alias length.
pub const ALIAS_MAX_LEN: usize = 30;

/// Validate an app-scoped alias (username).
///
/// Rules:
/// - 3–30 characters
/// - Lowercase alphanumeric and underscores only
/// - Must start with a letter
/// - Must not end with an underscore
/// - No consecutive underscores
/// - Input is checked as-is (caller should lowercase before calling if needed)
pub fn alias(alias: &str) -> Result<()> {
    if alias.len() < ALIAS_MIN_LEN {
        return Err(field_error(
            "alias",
            "too_short",
            format!("Must be at least {ALIAS_MIN_LEN} characters"),
        ));
    }

    if alias.len() > ALIAS_MAX_LEN {
        return Err(field_error(
            "alias",
            "too_long",
            format!("Must be at most {ALIAS_MAX_LEN} characters"),
        ));
    }

    let bytes = alias.as_bytes();

    // Must start with a letter
    if !bytes[0].is_ascii_lowercase() {
        return Err(field_error(
            "alias",
            "invalid_format",
            "Must start with a lowercase letter",
        ));
    }

    // Must not end with underscore
    if bytes[bytes.len() - 1] == b'_' {
        return Err(field_error(
            "alias",
            "invalid_format",
            "Must not end with an underscore",
        ));
    }

    // Check all characters and no consecutive underscores
    let mut prev_underscore = false;
    for &b in bytes {
        match b {
            b'a'..=b'z' | b'0'..=b'9' => {
                prev_underscore = false;
            }
            b'_' => {
                if prev_underscore {
                    return Err(field_error(
                        "alias",
                        "invalid_format",
                        "Must not contain consecutive underscores",
                    ));
                }
                prev_underscore = true;
            }
            _ => {
                return Err(field_error(
                    "alias",
                    "invalid_format",
                    "Must contain only lowercase letters, digits, and underscores",
                ));
            }
        }
    }

    Ok(())
}

// ── Display name ────────────────────────────────────────────────────────────

/// Minimum display name length.
pub const DISPLAY_NAME_MIN_LEN: usize = 1;
/// Maximum display name length.
pub const DISPLAY_NAME_MAX_LEN: usize = 50;

/// Validate a profile display name.
///
/// Rules:
/// - 1–50 characters
/// - Must not be only whitespace
/// - No control characters (newlines, tabs, etc.)
pub fn display_name(name: &str) -> Result<()> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(field_error(
            "display_name",
            "required",
            "Display name must not be empty",
        ));
    }

    if name.len() > DISPLAY_NAME_MAX_LEN {
        return Err(field_error(
            "display_name",
            "too_long",
            format!("Must be at most {DISPLAY_NAME_MAX_LEN} characters"),
        ));
    }

    if name.chars().any(char::is_control) {
        return Err(field_error(
            "display_name",
            "invalid_format",
            "Must not contain control characters",
        ));
    }

    Ok(())
}

// ── Hashtag ─────────────────────────────────────────────────────────────────

/// Maximum hashtag length (excluding the `#` prefix).
pub const HASHTAG_MAX_LEN: usize = 100;

/// Validate a hashtag.
///
/// Rules:
/// - Optionally starts with `#` (stripped for validation)
/// - 1–100 characters (after stripping `#`)
/// - Alphanumeric and underscores only (Unicode letters allowed)
/// - Must start with a letter or underscore
pub fn hashtag(tag: &str) -> Result<()> {
    let tag = tag.strip_prefix('#').unwrap_or(tag);

    if tag.is_empty() {
        return Err(field_error(
            "hashtag",
            "required",
            "Hashtag must not be empty",
        ));
    }

    if tag.len() > HASHTAG_MAX_LEN {
        return Err(field_error(
            "hashtag",
            "too_long",
            format!("Must be at most {HASHTAG_MAX_LEN} characters"),
        ));
    }

    let mut chars = tag.chars();
    let first = chars.next().unwrap(); // safe: tag is non-empty
    if !first.is_alphabetic() && first != '_' {
        return Err(field_error(
            "hashtag",
            "invalid_format",
            "Must start with a letter or underscore",
        ));
    }

    if !chars.all(|c| c.is_alphanumeric() || c == '_') {
        return Err(field_error(
            "hashtag",
            "invalid_format",
            "Must contain only letters, digits, and underscores",
        ));
    }

    Ok(())
}

pub fn link_policy(policy: &LinkPolicy) -> Result<()> {
    if let LinkPolicy::AppSelective { apps, .. } = policy {
        if apps.is_empty() {
            return Err(field_error(
                "link_policy",
                "invalid_value",
                "app_selective policy must include at least one app",
            ));
        }

        let unique_count = apps
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len();
        if unique_count != apps.len() {
            return Err(field_error(
                "link_policy",
                "invalid_value",
                "app_selective policy must not contain duplicate apps",
            ));
        }
    }

    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Create a validation error for a specific field.
fn field_error(
    field: &'static str,
    code: &'static str,
    message: impl Into<std::borrow::Cow<'static, str>>,
) -> NyxError {
    NyxError::validation(vec![FieldError::new(field, code, message)])
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Phone ───────────────────────────────────────────────────────────

    #[test]
    fn valid_phones() {
        assert!(phone("+14155551234").is_ok());
        assert!(phone("+353861234567").is_ok());
        assert!(phone("+1").is_ok()); // minimum: country code only
        assert!(phone("+123456789012345").is_ok()); // max 15 digits
    }

    #[test]
    fn invalid_phones() {
        assert!(phone("14155551234").is_err()); // no +
        assert!(phone("+").is_err()); // no digits
        assert!(phone("+1234567890123456").is_err()); // 16 digits
        assert!(phone("+1-415-555-1234").is_err()); // dashes
        assert!(phone("+1 415 555 1234").is_err()); // spaces
        assert!(phone("").is_err()); // empty
    }

    // ── Email ───────────────────────────────────────────────────────────

    #[test]
    fn valid_emails() {
        assert!(email("user@example.com").is_ok());
        assert!(email("a@b.co").is_ok());
        assert!(email("user+tag@example.com").is_ok());
        assert!(email("user.name@sub.example.com").is_ok());
    }

    #[test]
    fn invalid_emails() {
        assert!(email("noat").is_err());
        assert!(email("@example.com").is_err()); // empty local
        assert!(email("user@").is_err()); // empty domain
        assert!(email("user@nodot").is_err()); // no dot in domain
        assert!(email("user@-invalid.com").is_err()); // label starts with hyphen
        assert!(email("user@invalid-.com").is_err()); // label ends with hyphen
    }

    // ── Alias ───────────────────────────────────────────────────────────

    #[test]
    fn valid_aliases() {
        assert!(alias("abc").is_ok()); // minimum length
        assert!(alias("cool_user_42").is_ok());
        assert!(alias("a".repeat(30).as_str()).is_ok()); // max length
        assert!(alias("user123").is_ok());
    }

    #[test]
    fn invalid_aliases() {
        assert!(alias("ab").is_err()); // too short
        assert!(alias(&"a".repeat(31)).is_err()); // too long
        assert!(alias("_user").is_err()); // starts with underscore
        assert!(alias("123user").is_err()); // starts with digit
        assert!(alias("user_").is_err()); // ends with underscore
        assert!(alias("user__name").is_err()); // consecutive underscores
        assert!(alias("User").is_err()); // uppercase
        assert!(alias("user name").is_err()); // space
        assert!(alias("user-name").is_err()); // hyphen
    }

    // ── Display name ────────────────────────────────────────────────────

    #[test]
    fn valid_display_names() {
        assert!(display_name("Alice").is_ok());
        assert!(display_name("A").is_ok()); // minimum
        assert!(display_name("Alice McAliceFace the Third").is_ok());
        assert!(display_name("日本語の名前").is_ok()); // unicode
    }

    #[test]
    fn invalid_display_names() {
        assert!(display_name("").is_err()); // empty
        assert!(display_name("   ").is_err()); // whitespace only
        assert!(display_name(&"a".repeat(51)).is_err()); // too long
        assert!(display_name("has\nnewline").is_err()); // control char
        assert!(display_name("has\ttab").is_err()); // control char
    }

    // ── Hashtag ─────────────────────────────────────────────────────────

    #[test]
    fn valid_hashtags() {
        assert!(hashtag("rust").is_ok());
        assert!(hashtag("#rust").is_ok()); // with prefix
        assert!(hashtag("_private").is_ok()); // starts with underscore
        assert!(hashtag("CamelCase123").is_ok());
        assert!(hashtag("日本語").is_ok()); // unicode letters
    }

    #[test]
    fn invalid_hashtags() {
        assert!(hashtag("").is_err());
        assert!(hashtag("#").is_err()); // just prefix
        assert!(hashtag("123abc").is_err()); // starts with digit
        assert!(hashtag("has space").is_err());
        assert!(hashtag("has-dash").is_err());
    }

    #[test]
    fn valid_link_policies() {
        assert!(link_policy(&crate::LinkPolicy::OneWay).is_ok());
        assert!(link_policy(&crate::LinkPolicy::TwoWay).is_ok());
        assert!(link_policy(&crate::LinkPolicy::Revoked).is_ok());
        assert!(link_policy(&crate::LinkPolicy::AppSelective {
            apps: vec![crate::NyxApp::Uzume],
            direction: crate::LinkDirection::TwoWay,
        })
        .is_ok());
    }

    #[test]
    fn invalid_link_policy_rejects_empty_app_selective_apps() {
        let invalid = crate::LinkPolicy::AppSelective {
            apps: vec![],
            direction: crate::LinkDirection::OneWay,
        };
        assert!(link_policy(&invalid).is_err());
    }

    #[test]
    fn invalid_link_policy_rejects_duplicate_app_selective_apps() {
        let invalid = crate::LinkPolicy::AppSelective {
            apps: vec![crate::NyxApp::Uzume, crate::NyxApp::Uzume],
            direction: crate::LinkDirection::TwoWay,
        };

        assert!(link_policy(&invalid).is_err());
    }
}
