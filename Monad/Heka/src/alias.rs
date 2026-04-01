//! App-scoped alias resolution.
//!
//! Aliases are stored in the `nyx.app_aliases` PostgreSQL table.
//! This module provides helpers for alias generation. The actual SQL
//! queries live in each service's own queries module.
use nun::{Id, NyxApp};

/// Generates and resolves app-scoped user aliases.
///
/// In production, resolution is backed by a PostgreSQL query and a
/// DragonflyDB cache. This struct provides only the stateless helpers;
/// the database layer is owned by each service.
pub struct AliasResolver;

impl AliasResolver {
    /// Generate a default alias for a new user in an app.
    ///
    /// Format: `{app_prefix}_{short_id}` where `short_id` is the last 8 hex
    /// characters of the identity UUID (without dashes).
    pub fn generate_default<T>(identity_id: &Id<T>, app: NyxApp) -> String {
        let id_str = identity_id.to_string().replace('-', "");
        let short = &id_str[id_str.len().saturating_sub(8)..];
        format!("{}_{}", app.subject_prefix().to_lowercase(), short)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nun::Id;

    struct Identity;
    type IdentityId = Id<Identity>;

    #[test]
    fn generates_alias_with_app_prefix() {
        let id = IdentityId::new();
        let alias = AliasResolver::generate_default(&id, NyxApp::Uzume);
        assert!(alias.starts_with("uzume_"));
        assert_eq!(alias.len(), "uzume_".len() + 8);
    }
}
