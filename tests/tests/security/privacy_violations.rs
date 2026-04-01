use nyx_tests::security::privacy;
use serde_json::json;

#[test]
fn privacy_check_blocks_global_identity_leakage() {
    let leaking = json!({"identity_id": "00000000-0000-0000-0000-000000000001"});
    assert!(!privacy::should_not_expose_identity_id(&leaking));

    let safe = json!({"alias": "owner_alias"});
    assert!(privacy::should_not_expose_identity_id(&safe));
}

#[test]
fn privacy_check_blocks_cross_app_alias_leakage() {
    let leaking = json!({"anteros_alias": "hidden_alias"});
    assert!(!privacy::should_not_leak_cross_app_alias(&leaking, "uzume"));
}

#[test]
fn privacy_check_accepts_masked_or_absent_pii() {
    let masked = json!({"email": "a***@example.com"});
    let absent = json!({"alias": "owner_alias"});
    assert!(privacy::should_mask_pii(&masked, "email"));
    assert!(privacy::should_mask_pii(&absent, "email"));
}
