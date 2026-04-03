use nun::IdentityId;

/// The resolved Nyx platform identity for an authenticated user.
///
/// Produced by [`KratosClient::validate_session`] and
/// [`KratosClient::get_identity`]. Passed through the auth middleware into
/// every handler via the `AuthUser` extractor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyxIdentity {
    /// Kratos identity UUID — the stable, globally unique platform identifier.
    /// Never exposed to users directly; app-scoped aliases are used instead.
    pub id: IdentityId,

    /// Verified email address. `None` for freshly created OIDC accounts that
    /// haven't completed email verification yet.
    pub email: Option<String>,

    /// The user's chosen Nyx handle (e.g. "alice.nyx").
    /// Unique across the platform. `None` for OIDC users still in the
    /// registration continuation flow (choosing their handle).
    pub nyx_id: Option<String>,

    /// Optional display name (e.g. "Alice Smith").
    /// Can be set/changed via the settings flow.
    pub display_name: Option<String>,
}

/// An app-scoped alias record mapping a platform identity to an app-specific
/// handle. Stored in `nyx.app_aliases`, never in Kratos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppAlias {
    pub app:   String,
    pub alias: String,
}
