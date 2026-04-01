use nyx_tests::generators::{arb_app, arb_email, arb_username, arb_uuid};
use proptest::prelude::*;

proptest! {
    #[test]
    fn generated_uuid_is_always_16_bytes(uuid in arb_uuid()) {
        prop_assert_eq!(uuid.as_bytes().len(), 16);
    }

    #[test]
    fn generated_email_contains_at_sign(email in arb_email()) {
        prop_assert!(email.contains('@'));
    }

    #[test]
    fn generated_alias_length_is_bounded(alias in arb_username()) {
        prop_assert!((3..=20).contains(&alias.len()));
    }

    #[test]
    fn generated_app_is_known_variant(app in arb_app()) {
        let serialized = format!("{app:?}");
        prop_assert!(serialized == "Uzume" || serialized == "Anteros" || serialized == "Themis");
    }
}
