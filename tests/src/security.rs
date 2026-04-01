//! Security testing utilities and payloads.
//!
//! Comprehensive test payloads for:
//! - SQL injection
//! - XSS (Cross-Site Scripting)
//! - Authorization bypass
//! - Privacy violations (PII exposure, alias leakage)
//! - Edge cases and boundary conditions

/// SQL injection test payloads.
pub mod sql_injection {
    pub const CLASSIC_OR: &str = "' OR '1'='1";
    pub const UNION_SELECT: &str = "' UNION SELECT NULL, NULL, NULL--";
    pub const COMMENT_OUT: &str = "admin'--";
    pub const STACKED_QUERIES: &str = "'; DROP TABLE users;--";
    pub const TIME_BASED: &str = "' OR SLEEP(5)--";
    pub const BOOLEAN_BASED: &str = "' AND 1=1--";
    pub const ERROR_BASED: &str = "' AND (SELECT 1 FROM (SELECT COUNT(*),CONCAT((SELECT @@version),FLOOR(RAND()*2))x FROM information_schema.tables GROUP BY x)y)--";

    /// All SQL injection payloads.
    pub fn all_payloads() -> Vec<&'static str> {
        vec![
            CLASSIC_OR,
            UNION_SELECT,
            COMMENT_OUT,
            STACKED_QUERIES,
            TIME_BASED,
            BOOLEAN_BASED,
            ERROR_BASED,
        ]
    }
}

/// XSS (Cross-Site Scripting) test payloads.
pub mod xss {
    pub const SCRIPT_TAG: &str = "<script>alert('XSS')</script>";
    pub const IMG_ONERROR: &str = "<img src=x onerror=alert('XSS')>";
    pub const SVG_ONLOAD: &str = "<svg/onload=alert('XSS')>";
    pub const BODY_ONLOAD: &str = "<body onload=alert('XSS')>";
    pub const IFRAME_JAVASCRIPT: &str = "<iframe src=\"javascript:alert('XSS')\"></iframe>";
    pub const A_HREF_JAVASCRIPT: &str = "<a href=\"javascript:alert('XSS')\">Click</a>";
    pub const INPUT_AUTOFOCUS: &str = "<input autofocus onfocus=alert('XSS')>";
    pub const MARKDOWN_XSS: &str = "[Click](javascript:alert('XSS'))";

    /// All XSS payloads.
    pub fn all_payloads() -> Vec<&'static str> {
        vec![
            SCRIPT_TAG,
            IMG_ONERROR,
            SVG_ONLOAD,
            BODY_ONLOAD,
            IFRAME_JAVASCRIPT,
            A_HREF_JAVASCRIPT,
            INPUT_AUTOFOCUS,
            MARKDOWN_XSS,
        ]
    }
}

/// Privacy violation test cases.
pub mod privacy {
    /// Test that IdentityId is never exposed in API responses.
    pub fn should_not_expose_identity_id(json: &serde_json::Value) -> bool {
        !json.to_string().contains("identity_id") && !json.to_string().contains("identityId")
    }

    /// Test that Kratos ID is never exposed in API responses.
    pub fn should_not_expose_kratos_id(json: &serde_json::Value) -> bool {
        let s = json.to_string();
        // Allow in test/debug contexts but not in user-facing fields
        !s.contains("kratos_id") && !s.contains("kratosId")
    }

    /// Test that cross-app aliases are not leaked.
    pub fn should_not_leak_cross_app_alias(json: &serde_json::Value, current_app: &str) -> bool {
        let s = json.to_string();
        // Check that aliases from other apps aren't present
        if current_app == "uzume" {
            !s.contains("anteros_alias") && !s.contains("themis_alias")
        } else if current_app == "anteros" {
            !s.contains("uzume_alias") && !s.contains("themis_alias")
        } else {
            true
        }
    }

    /// Test that email/phone are properly masked or omitted.
    pub fn should_mask_pii(json: &serde_json::Value, field: &str) -> bool {
        if let Some(value) = json.get(field) {
            if let Some(s) = value.as_str() {
                // Should be masked or omitted entirely
                s.contains("***") || s.is_empty()
            } else {
                false
            }
        } else {
            // Field not present is also acceptable
            true
        }
    }
}

/// Authorization bypass test scenarios.
pub mod authz {
    use serde_json::json;

    /// Test payload for accessing another user's private resource.
    pub fn access_other_user_resource(victim_id: &str, attacker_id: &str) -> serde_json::Value {
        json!({
            "resource_id": victim_id,
            "actor_id": attacker_id,
            "action": "read"
        })
    }

    /// Test payload for privilege escalation.
    pub fn privilege_escalation(user_id: &str, target_role: &str) -> serde_json::Value {
        json!({
            "user_id": user_id,
            "role": target_role,
            "action": "elevate"
        })
    }

    /// Test payload for token manipulation.
    pub fn manipulated_token(original_user_id: &str, target_user_id: &str) -> String {
        format!(
            "Bearer manipulated-token-{}-as-{}",
            original_user_id, target_user_id
        )
    }
}

/// Edge case and boundary condition test data.
pub mod edge_cases {
    /// Empty string.
    pub const EMPTY: &str = "";

    /// Single character.
    pub const SINGLE_CHAR: &str = "a";

    /// Maximum reasonable length for user input (100KB).
    pub fn max_length_string() -> String {
        "a".repeat(100_000)
    }

    /// Unicode edge cases.
    pub const UNICODE_EMOJI: &str = "Hello 👋 World 🌍";
    pub const UNICODE_RTL: &str = "مرحبا بالعالم"; // Arabic (RTL)
    pub const UNICODE_COMBINING: &str = "e\u{0301}"; // é (combining acute accent)
    pub const UNICODE_ZERO_WIDTH: &str = "a\u{200B}b"; // Zero-width space

    /// Special characters.
    pub const NULL_BYTE: &str = "test\0null";
    pub const NEWLINES: &str = "line1\nline2\rline3\r\nline4";
    pub const TABS: &str = "col1\tcol2\tcol3";

    /// Numeric edge cases.
    pub const I32_MIN: i32 = i32::MIN;
    pub const I32_MAX: i32 = i32::MAX;
    pub const I64_MIN: i64 = i64::MIN;
    pub const I64_MAX: i64 = i64::MAX;

    /// Timestamp edge cases.
    pub const EPOCH_ZERO: i64 = 0;
    pub const YEAR_2038: i64 = 2_147_483_647; // Unix timestamp overflow
    pub const YEAR_9999: i64 = 253_402_300_799; // Far future

    /// All edge case strings.
    pub fn all_string_edge_cases() -> Vec<&'static str> {
        vec![
            EMPTY,
            SINGLE_CHAR,
            UNICODE_EMOJI,
            UNICODE_RTL,
            UNICODE_COMBINING,
            UNICODE_ZERO_WIDTH,
            NULL_BYTE,
            NEWLINES,
            TABS,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sql_injection_payloads_are_present() {
        let payloads = sql_injection::all_payloads();
        assert!(!payloads.is_empty());
        assert!(payloads.contains(&sql_injection::CLASSIC_OR));
    }

    #[test]
    fn xss_payloads_are_present() {
        let payloads = xss::all_payloads();
        assert!(!payloads.is_empty());
        assert!(payloads.contains(&xss::SCRIPT_TAG));
    }

    #[test]
    fn privacy_checks_detect_identity_id() {
        let bad_json = json!({"user": {"identity_id": "123"}});
        assert!(!privacy::should_not_expose_identity_id(&bad_json));

        let good_json = json!({"user": {"alias_id": "456"}});
        assert!(privacy::should_not_expose_identity_id(&good_json));
    }

    #[test]
    fn edge_cases_cover_unicode() {
        let cases = edge_cases::all_string_edge_cases();
        assert!(cases.contains(&edge_cases::UNICODE_EMOJI));
        assert!(cases.contains(&edge_cases::UNICODE_RTL));
    }
}
