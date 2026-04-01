//! # Ogma — Matrix/Continuwuity messaging client for the Nyx platform
//!
//! Privacy-isolated messaging. App-scoped rooms tagged with `nyx.app` state events.
//! Cross-app visibility requires explicit consent via Heka linking.
//!
//! ## Planned modules
//! - `client` — `MatrixClient` wrapping Continuwuity HTTP API
//! - `rooms` — room lifecycle, app-scoped creation, invite
//! - `messages` — send/receive, media attachments
//! - `aliases` — `NyxId + NyxApp → Matrix user` mapping
//! - `privacy` — filter by app tag, cross-app link checks

// TODO: implement Ogma modules
