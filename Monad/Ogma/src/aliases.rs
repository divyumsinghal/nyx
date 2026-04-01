//! Maps Nyx app-scoped aliases to Matrix user IDs and room aliases.
//!
//! Matrix user IDs have the form `@{localpart}:{homeserver_domain}`.
//! Nyx uses app-scoped aliases as the localpart so that a user's identity
//! within one app cannot be correlated with another app's Matrix presence
//! without explicit cross-app consent.

/// Stateless helpers for translating between Nyx app-scoped aliases and
/// Matrix identifiers.
///
/// These helpers are pure functions — no I/O, no database. They exist to
/// centralise the naming convention so that every caller agrees on the
/// format.
pub struct AliasMapper;

impl AliasMapper {
    /// Convert an app-scoped alias into a fully-qualified Matrix user ID.
    ///
    /// Format: `@{alias}:{homeserver_domain}`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ogma::AliasMapper;
    ///
    /// let matrix_id = AliasMapper::to_matrix_user_id("uzume_deadbeef", "matrix.nyx.app");
    /// assert_eq!(matrix_id, "@uzume_deadbeef:matrix.nyx.app");
    /// ```
    pub fn to_matrix_user_id(alias: &str, homeserver_domain: &str) -> String {
        format!("@{alias}:{homeserver_domain}")
    }

    /// Extract the app-scoped alias from a fully-qualified Matrix user ID.
    ///
    /// Returns `None` if the string does not match the `@localpart:domain` form.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ogma::AliasMapper;
    ///
    /// let alias = AliasMapper::from_matrix_user_id("@uzume_deadbeef:matrix.nyx.app");
    /// assert_eq!(alias, Some("uzume_deadbeef".to_string()));
    ///
    /// assert_eq!(AliasMapper::from_matrix_user_id("not-a-matrix-id"), None);
    /// ```
    pub fn from_matrix_user_id(matrix_id: &str) -> Option<String> {
        // Must start with '@' and contain ':'
        let without_at = matrix_id.strip_prefix('@')?;
        let colon_pos = without_at.find(':')?;
        let localpart = &without_at[..colon_pos];
        if localpart.is_empty() {
            return None;
        }
        Some(localpart.to_owned())
    }

    /// Generate a deterministic DM room alias for two users within an app.
    ///
    /// Aliases are sorted lexicographically so that
    /// `dm_room_alias(a, b, app) == dm_room_alias(b, a, app)`.
    ///
    /// Format: `#{app}_dm_{alias_min}_{alias_max}:{homeserver_domain}` — the
    /// homeserver domain is **not** embedded here; callers append it when
    /// registering the alias with the homeserver.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ogma::AliasMapper;
    ///
    /// let a = AliasMapper::dm_room_alias("uzume_aaaa", "uzume_bbbb", "uzume");
    /// let b = AliasMapper::dm_room_alias("uzume_bbbb", "uzume_aaaa", "uzume");
    /// assert_eq!(a, b);
    /// assert_eq!(a, "uzume_dm_uzume_aaaa_uzume_bbbb");
    /// ```
    pub fn dm_room_alias(alias_a: &str, alias_b: &str, app: &str) -> String {
        let (lo, hi) = if alias_a <= alias_b {
            (alias_a, alias_b)
        } else {
            (alias_b, alias_a)
        };
        format!("{app}_dm_{lo}_{hi}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_matrix_user_id_produces_correct_format() {
        assert_eq!(
            AliasMapper::to_matrix_user_id("uzume_deadbeef", "matrix.nyx.app"),
            "@uzume_deadbeef:matrix.nyx.app"
        );
    }

    #[test]
    fn from_matrix_user_id_extracts_alias() {
        assert_eq!(
            AliasMapper::from_matrix_user_id("@uzume_deadbeef:matrix.nyx.app"),
            Some("uzume_deadbeef".to_string())
        );
    }

    #[test]
    fn from_matrix_user_id_rejects_invalid() {
        assert_eq!(AliasMapper::from_matrix_user_id("not-a-matrix-id"), None);
        assert_eq!(AliasMapper::from_matrix_user_id("@:nodomain"), None);
        assert_eq!(AliasMapper::from_matrix_user_id("no-at:domain"), None);
    }

    #[test]
    fn dm_room_alias_is_commutative() {
        let a = AliasMapper::dm_room_alias("uzume_aaaa", "uzume_bbbb", "uzume");
        let b = AliasMapper::dm_room_alias("uzume_bbbb", "uzume_aaaa", "uzume");
        assert_eq!(a, b);
    }

    #[test]
    fn dm_room_alias_format() {
        let alias = AliasMapper::dm_room_alias("uzume_aaaa", "uzume_bbbb", "uzume");
        assert_eq!(alias, "uzume_dm_uzume_aaaa_uzume_bbbb");
    }

    #[test]
    fn roundtrip_alias_via_matrix_id() {
        let original = "uzume_cafebabe";
        let matrix_id = AliasMapper::to_matrix_user_id(original, "matrix.nyx.app");
        let recovered = AliasMapper::from_matrix_user_id(&matrix_id).unwrap();
        assert_eq!(recovered, original);
    }
}
