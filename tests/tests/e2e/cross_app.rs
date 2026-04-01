use nyx_tests::security::privacy;
use serde_json::json;

#[test]
fn cross_app_flow_enforces_alias_isolation() {
    let response = json!({
        "profile": {
            "alias": "uzume_alias"
        }
    });

    assert!(privacy::should_not_leak_cross_app_alias(&response, "uzume"));
}
