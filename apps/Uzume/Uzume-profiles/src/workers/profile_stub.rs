//! NATS worker: `nyx.user.created` → create Uzume profile stub.
//!
//! When the Nyx platform emits a `nyx.user.created` event (a new Kratos
//! identity was registered), this worker creates a minimal profile row in
//! `uzume.profiles` so the new user appears in the system immediately.
//!
//! # Idempotency
//!
//! Profile creation uses `ON CONFLICT DO NOTHING` on `(identity_id)`, so
//! replaying the event is safe.

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{handlers::profile::create_profile_stub, state::AppState};
use nyx_events::{subjects, Subscriber};

/// Payload shape for the `nyx.user.created` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedPayload {
    /// Kratos identity UUID.
    pub identity_id: Uuid,
    /// Initial app-scoped alias chosen at registration.
    pub alias: String,
    /// Display name — defaults to the alias if not supplied.
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Spawn the `nyx.user.created` subscriber task.
///
/// Runs indefinitely in the background — caller should `tokio::spawn` this.
pub async fn run(state: AppState) {
    let subscriber = Subscriber::new(state.nats.clone());

    let mut stream = match subscriber
        .subscribe::<UserCreatedPayload>(subjects::USER_CREATED)
        .await
    {
        Ok(s) => s,
        Err(err) => {
            error!(?err, "profile_stub worker: failed to subscribe");
            return;
        }
    };

    info!(
        "profile_stub worker: listening on {}",
        subjects::USER_CREATED
    );

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => handle_user_created(&state, event.payload).await,
            Err(err) => {
                error!(?err, "profile_stub worker: failed to receive event");
            }
        }
    }
}

#[instrument(skip(state), fields(identity_id = %payload.identity_id, alias = %payload.alias))]
async fn handle_user_created(state: &AppState, payload: UserCreatedPayload) {
    let display_name = payload.display_name.as_deref().unwrap_or(&payload.alias);

    match create_profile_stub(&state.db, payload.identity_id, &payload.alias, display_name).await {
        Ok(profile) => {
            info!(profile_id = %profile.id, "profile stub created");
        }
        Err(err) => {
            // Conflict means the profile already exists — log at debug level.
            if err.status_code() == 409 {
                tracing::debug!(
                    identity_id = %payload.identity_id,
                    "profile already exists, skipping"
                );
            } else {
                error!(?err, "profile_stub worker: failed to create profile stub");
            }
        }
    }
}
