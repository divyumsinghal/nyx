// oidc.google.jsonnet
// Maps Google OIDC claims to Nyx identity traits.
//
// Google provides: email, email_verified, name, given_name, family_name, picture, sub
//
// We only map email (and only if Google has verified it).
// nyx_id is intentionally NOT mapped here — the user will be prompted to choose
// one via Kratos's registration continuation flow (422 response with nyx_id field).
//
// display_name is optionally populated from Google's name claim.
//
// Docs: https://www.ory.sh/docs/kratos/social-signin/google

local claims = {
  email_verified: false,
} + std.extVar('claims');

{
  identity: {
    traits: {
      // Only populate email when Google has verified it.
      // Unverified emails are a security risk (account takeover via email change).
      [if 'email' in claims && claims.email_verified then 'email' else null]: claims.email,

      // Optionally pre-fill display_name from Google profile.
      // The user can change this later via the settings flow.
      [if 'name' in claims && std.length(claims.name) > 0 then 'display_name' else null]: claims.name,
    },
  },
}
