use nyx_tests::security::authz;

#[test]
fn authz_payload_targets_other_user_resource() {
    let payload = authz::access_other_user_resource("victim-id", "attacker-id");
    assert_eq!(payload["resource_id"], "victim-id");
    assert_eq!(payload["actor_id"], "attacker-id");
    assert_eq!(payload["action"], "read");
}

#[test]
fn authz_payload_captures_privilege_escalation_attempt() {
    let payload = authz::privilege_escalation("attacker-id", "admin");
    assert_eq!(payload["user_id"], "attacker-id");
    assert_eq!(payload["role"], "admin");
    assert_eq!(payload["action"], "elevate");
}

#[test]
fn authz_payload_captures_token_manipulation_attempt() {
    let token = authz::manipulated_token("attacker-id", "victim-id");
    assert!(token.starts_with("Bearer manipulated-token"));
    assert!(token.contains("attacker-id-as-victim-id"));
}
