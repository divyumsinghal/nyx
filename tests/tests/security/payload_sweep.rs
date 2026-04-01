use nyx_tests::security;
use serde_json::json;

#[test]
fn sql_injection_payloads_are_non_empty_and_diverse() {
    let payloads = security::sql_injection::all_payloads();
    assert!(payloads.len() >= 6);
    assert!(payloads.iter().all(|payload| !payload.trim().is_empty()));
    assert!(payloads.iter().any(|payload| payload.contains("UNION")));
    assert!(payloads
        .iter()
        .any(|payload| payload.contains("DROP TABLE")));
}

#[test]
fn xss_payloads_cover_script_and_event_handler_vectors() {
    let payloads = security::xss::all_payloads();
    assert!(payloads.len() >= 6);
    assert!(payloads.iter().any(|payload| payload.contains("<script")));
    assert!(payloads.iter().any(|payload| payload.contains("onerror")));
    assert!(payloads
        .iter()
        .any(|payload| payload.contains("javascript:")));
}

#[test]
fn authz_bypass_payloads_cover_header_and_token_manipulation() {
    let payload = security::authz::access_other_user_resource("victim", "attacker");
    let escalation = security::authz::privilege_escalation("attacker", "admin");
    let manipulated = security::authz::manipulated_token("attacker", "victim");

    assert_eq!(payload["resource_id"], "victim");
    assert_eq!(escalation["role"], "admin");
    assert!(manipulated.contains("Bearer manipulated-token"));
}

#[test]
fn privacy_payloads_include_alias_and_identity_leak_patterns() {
    let response = json!({
        "profile": {
            "alias": "alice",
            "bio": "hello"
        }
    });

    assert!(security::privacy::should_not_expose_identity_id(&response));
    assert!(security::privacy::should_not_expose_kratos_id(&response));
    assert!(security::privacy::should_not_leak_cross_app_alias(
        &response, "uzume"
    ));
}
