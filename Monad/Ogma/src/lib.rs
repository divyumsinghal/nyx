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
//!
//! let msg_client = ogma::MessageClient::new(&homeserver_url, server_token);
//! let event_id = msg_client.send_message(ogma::SendMessageRequest {
//!     room_id,
//!     body: "Hello!".to_string(),
//!     msg_type: ogma::MsgType::Text,
//! }).await?;
//!
//! let matrix_id = ogma::AliasMapper::to_matrix_user_id("uzume_deadbeef", "matrix.nyx.app");
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

pub mod aliases;
pub mod client;
pub mod messages;
pub mod privacy;
pub mod room;

pub use aliases::AliasMapper;
pub use client::MatrixClient;
pub use messages::{MessageClient, MessageEvent, MsgType, SendMessageRequest};
pub use privacy::PrivacyGuard;
pub use room::MatrixRoom;
