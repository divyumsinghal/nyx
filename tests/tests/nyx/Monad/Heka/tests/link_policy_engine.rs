use heka::link_policy::{LinkPolicyEngine, LinkRule};
use nun::{IdentityId, LinkDirection, LinkPolicy, NyxApp};

fn id(seed: u128) -> IdentityId {
    let raw = format!("00000000-0000-0000-0000-{:012x}", seed);
    raw.parse().unwrap()
}

#[test]
fn default_private_when_no_rules_exist() {
    // #given
    let engine = LinkPolicyEngine::new();

    // #when
    let visible = engine.is_visible(id(1), id(2), NyxApp::Uzume, NyxApp::Themis);

    // #then
    assert!(!visible);
}

#[test]
fn alias_resolution_is_app_scoped_and_explicit() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let identity = id(3);
    engine.upsert_alias(identity, NyxApp::Uzume, "uzume_name");

    // #when
    let same_app = engine.resolve_alias(identity, NyxApp::Uzume);
    let other_app = engine.resolve_alias(identity, NyxApp::Themis);

    // #then
    assert_eq!(same_app, Some("uzume_name"));
    assert_eq!(other_app, None);
}

#[test]
fn identity_lookup_from_alias_respects_app_scope() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let identity = id(4);
    engine.upsert_alias(identity, NyxApp::Themis, "themis_name");

    // #when
    let same_app = engine.identity_from_alias("themis_name", NyxApp::Themis);
    let wrong_app = engine.identity_from_alias("themis_name", NyxApp::Uzume);

    // #then
    assert_eq!(same_app, Some(identity));
    assert_eq!(wrong_app, None);
}

#[test]
fn one_way_allows_only_source_to_target_direction() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let source = id(10);
    let target = id(20);
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::OneWay,
    });

    // #when
    let forward = engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Themis);
    let reverse = engine.is_visible(target, source, NyxApp::Themis, NyxApp::Uzume);

    // #then
    assert!(forward);
    assert!(!reverse);
}

#[test]
fn two_way_allows_both_directions_for_same_app_pair() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let source = id(30);
    let target = id(40);
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::TwoWay,
    });

    // #when
    let forward = engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Themis);
    let reverse = engine.is_visible(target, source, NyxApp::Themis, NyxApp::Uzume);

    // #then
    assert!(forward);
    assert!(reverse);
}

#[test]
fn app_selective_allows_only_selected_apps() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let source = id(50);
    let target = id(60);
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::AppSelective {
            apps: vec![NyxApp::Themis],
            direction: LinkDirection::OneWay,
        },
    });

    // #when
    let selected = engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Themis);
    let unselected = engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Anteros);

    // #then
    assert!(selected);
    assert!(!unselected);
}

#[test]
fn revoked_policy_immediately_denies_visibility() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let source = id(70);
    let target = id(80);
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::TwoWay,
    });
    assert!(engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Themis));

    // #when
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::Revoked,
    });

    // #then
    assert!(!engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Themis));
    assert!(!engine.is_visible(target, source, NyxApp::Themis, NyxApp::Uzume));
}

#[test]
fn precedence_is_deterministic_revoked_overrides_allow() {
    // #given
    let mut engine = LinkPolicyEngine::new();
    let source = id(90);
    let target = id(100);
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::TwoWay,
    });

    // #when a conflicting revoke for the same link tuple is applied later
    engine.upsert(LinkRule {
        owner: source,
        viewer: target,
        from_app: NyxApp::Uzume,
        to_app: NyxApp::Themis,
        policy: LinkPolicy::Revoked,
    });

    // #then revoke deterministically wins and returns to private default
    assert!(!engine.is_visible(source, target, NyxApp::Uzume, NyxApp::Themis));
}
