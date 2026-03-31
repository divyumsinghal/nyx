use std::collections::HashMap;

use nun::{IdentityId, LinkDirection, LinkPolicy, NyxApp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AliasKey {
    identity: IdentityId,
    app: NyxApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RuleKey {
    owner: IdentityId,
    viewer: IdentityId,
    from_app: NyxApp,
    to_app: NyxApp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRule {
    pub owner: IdentityId,
    pub viewer: IdentityId,
    pub from_app: NyxApp,
    pub to_app: NyxApp,
    pub policy: LinkPolicy,
}

#[derive(Debug, Clone, Default)]
pub struct LinkPolicyEngine {
    aliases: HashMap<AliasKey, String>,
    alias_lookup: HashMap<(NyxApp, String), IdentityId>,
    rules: HashMap<RuleKey, LinkPolicy>,
}

impl LinkPolicyEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_alias(&mut self, identity: IdentityId, app: NyxApp, alias: impl Into<String>) {
        let alias = alias.into();
        let key = AliasKey { identity, app };

        if let Some(previous) = self.aliases.insert(key, alias.clone()) {
            self.alias_lookup.remove(&(app, previous));
        }

        self.alias_lookup.insert((app, alias), identity);
    }

    pub fn resolve_alias(&self, identity: IdentityId, app: NyxApp) -> Option<&str> {
        self.aliases
            .get(&AliasKey { identity, app })
            .map(String::as_str)
    }

    pub fn identity_from_alias(&self, alias: &str, app: NyxApp) -> Option<IdentityId> {
        self.alias_lookup.get(&(app, alias.to_owned())).copied()
    }

    pub fn upsert(&mut self, rule: LinkRule) {
        let key = RuleKey {
            owner: rule.owner,
            viewer: rule.viewer,
            from_app: rule.from_app,
            to_app: rule.to_app,
        };
        self.rules.insert(key, rule.policy);
    }

    pub fn is_visible(
        &self,
        owner: IdentityId,
        viewer: IdentityId,
        from_app: NyxApp,
        to_app: NyxApp,
    ) -> bool {
        self.evaluate(owner, viewer, from_app, to_app)
            .is_some_and(VisibilityDecision::allowed)
    }

    fn evaluate(
        &self,
        owner: IdentityId,
        viewer: IdentityId,
        from_app: NyxApp,
        to_app: NyxApp,
    ) -> Option<VisibilityDecision> {
        if let Some(direct) = self.evaluate_direct(owner, viewer, from_app, to_app) {
            return Some(direct);
        }

        self.evaluate_reverse(owner, viewer, from_app, to_app)
    }

    fn evaluate_direct(
        &self,
        owner: IdentityId,
        viewer: IdentityId,
        from_app: NyxApp,
        to_app: NyxApp,
    ) -> Option<VisibilityDecision> {
        let key = RuleKey {
            owner,
            viewer,
            from_app,
            to_app,
        };
        self.rules
            .get(&key)
            .map(|policy| evaluate_policy(policy.clone(), to_app, false))
    }

    fn evaluate_reverse(
        &self,
        owner: IdentityId,
        viewer: IdentityId,
        from_app: NyxApp,
        to_app: NyxApp,
    ) -> Option<VisibilityDecision> {
        let key = RuleKey {
            owner: viewer,
            viewer: owner,
            from_app: to_app,
            to_app: from_app,
        };
        self.rules
            .get(&key)
            .map(|policy| evaluate_policy(policy.clone(), from_app, true))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisibilityDecision {
    Allow,
    Deny,
}

impl VisibilityDecision {
    fn allowed(self) -> bool {
        matches!(self, Self::Allow)
    }
}

fn evaluate_policy(policy: LinkPolicy, target_app: NyxApp, reverse: bool) -> VisibilityDecision {
    match policy {
        LinkPolicy::Revoked => VisibilityDecision::Deny,
        LinkPolicy::OneWay => {
            if reverse {
                VisibilityDecision::Deny
            } else {
                VisibilityDecision::Allow
            }
        }
        LinkPolicy::TwoWay => VisibilityDecision::Allow,
        LinkPolicy::AppSelective { apps, direction } => {
            if !apps.contains(&target_app) {
                return VisibilityDecision::Deny;
            }

            match (direction, reverse) {
                (LinkDirection::OneWay, true) => VisibilityDecision::Deny,
                _ => VisibilityDecision::Allow,
            }
        }
    }
}
