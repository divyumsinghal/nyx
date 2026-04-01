//! # Ogma — Matrix/Continuwuity messaging + privacy enforcement
//!
//! Privacy-isolated messaging for the Nyx platform. Each app creates
//! Matrix rooms tagged with `nyx.app` state events. Cross-app visibility
//! requires explicit user consent via Heka linking.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let client = ogma::MatrixClient::new(&config.messaging, server_token);
//! let room_id = client.create_dm_room(&matrix_user_id).await?;
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod privacy;
pub mod room;

pub use client::MatrixClient;
pub use privacy::PrivacyGuard;
