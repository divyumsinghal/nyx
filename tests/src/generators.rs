//! Property-based testing generators (proptest, quickcheck, arbitrary).

use nun::id::Id;
use nun::NyxApp;
use proptest::prelude::*;
use uuid::Uuid;

/// Generate arbitrary UUIDs for property-based testing.
pub fn arb_uuid() -> impl Strategy<Value = Uuid> {
    any::<[u8; 16]>().prop_map(|bytes| Uuid::from_bytes(bytes))
}

/// Generate arbitrary typed IDs.
pub fn arb_id<T>() -> impl Strategy<Value = Id<T>> {
    arb_uuid().prop_map(Id::from_uuid)
}

/// Generate arbitrary NyxApp variants.
pub fn arb_app() -> impl Strategy<Value = NyxApp> {
    prop_oneof![
        Just(NyxApp::Uzume),
        Just(NyxApp::Anteros),
        Just(NyxApp::Themis),
    ]
}

/// Generate arbitrary email addresses.
pub fn arb_email() -> impl Strategy<Value = String> {
    "[a-z]{3,10}@[a-z]{3,8}\\.(com|net|org)"
}

/// Generate arbitrary phone numbers (E.164 format).
pub fn arb_phone() -> impl Strategy<Value = String> {
    "\\+1[0-9]{10}".prop_map(|s| s.to_string())
}

/// Generate arbitrary usernames (alphanumeric + underscore, 3-20 chars).
pub fn arb_username() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,19}"
}

/// Generate arbitrary non-empty strings (1-100 chars).
pub fn arb_non_empty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,100}"
}

/// Generate arbitrary content strings (10-500 chars).
pub fn arb_content() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,!?]{10,500}"
}

/// Generate arbitrary positive integers (1..10000).
pub fn arb_positive_int() -> impl Strategy<Value = i32> {
    1..10000
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn uuid_generator_produces_valid_uuids(uuid in arb_uuid()) {
            // Just check it doesn't panic
            let _id: Id<()> = Id::from_uuid(uuid);
        }

        #[test]
        fn email_generator_produces_valid_format(email in arb_email()) {
            assert!(email.contains('@'));
            assert!(email.contains('.'));
        }

        #[test]
        fn phone_generator_produces_e164(phone in arb_phone()) {
            assert!(phone.starts_with("+1"));
            assert_eq!(phone.len(), 12);
        }

        #[test]
        fn username_generator_produces_valid_usernames(username in arb_username()) {
            assert!(username.len() >= 3);
            assert!(username.len() <= 20);
            assert!(username.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        }
    }
}
