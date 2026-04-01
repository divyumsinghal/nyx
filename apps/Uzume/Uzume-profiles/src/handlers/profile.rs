//! Axum handlers for profile endpoints.
//!
//! Each handler follows the pattern:
//! 1. Extract validated inputs from the request.
//! 2. Call queries to load / persist data.
//! 3. Apply domain logic from the services layer.
//! 4. Publish any domain events.
//! 5. Return an `ApiResponse<T>`.

use axum::{
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use nyx_api::{ApiResponse, AuthUser, ValidatedJson};
use nyx_events::{subjects, Publisher};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    models::profile::ProfileInsert,
    queries::profiles as profile_queries,
    services::profile::{
        check_profile_visibility, PatchProfileRequest, ProfileResponse,
    },
    state::AppState,
};
use nun::NyxError;

// ── Optional auth extractor ───────────────────────────────────────────────────

/// An optional authenticated user.
///
/// Unlike [`AuthUser`] this extractor does not reject the request when no
/// `Authorization` header is present — it simply returns `None`. This allows
/// public endpoints to optionally accept an identity for personalisation.
pub struct MaybeAuthUser(pub Option<Uuid>);

impl<S: Send + Sync> FromRequestParts<S> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let maybe = AuthUser::from_request_parts(parts, state).await.ok();
        Ok(Self(maybe.map(|u| u.user_id)))
    }
}

// ── GET /profiles/:alias ─────────────────────────────────────────────────────

/// Return the public profile for the given alias.
///
/// Private profiles are visible only to the owning identity.
#[instrument(skip(state), fields(alias = %alias))]
pub async fn get_profile(
    State(state): State<AppState>,
    MaybeAuthUser(viewer_identity_id): MaybeAuthUser,
    Path(alias): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let row = profile_queries::get_profile_by_alias(&state.db, &alias)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    check_profile_visibility(&row, viewer_identity_id)?;

    Ok(ApiResponse::ok(ProfileResponse::from(row)))
}

// ── GET /profiles/me ─────────────────────────────────────────────────────────

/// Return the authenticated user's own profile.
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn get_my_profile(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let row = profile_queries::get_profile_by_identity(&state.db, user.user_id)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    Ok(ApiResponse::ok(ProfileResponse::from(row)))
}

// ── PATCH /profiles/me ───────────────────────────────────────────────────────

/// Update the authenticated user's profile.
///
/// Only fields present in the request body are modified.
#[instrument(skip(state), fields(user_id = %user.user_id))]
pub async fn patch_my_profile(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<PatchProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let existing = profile_queries::get_profile_by_identity(&state.db, user.user_id)
        .await?
        .ok_or_else(|| NyxError::not_found("profile_not_found", "Profile not found"))?;

    let update = body.into_profile_update();
    let updated = profile_queries::update_profile(&state.db, existing.id, &update).await?;

    // Publish profile.updated event for Brizo (search sync) and other consumers.
    let publisher = Publisher::new(state.nats.clone(), "Uzume");
    let payload = serde_json::json!({
        "id": updated.id,
        "alias": updated.alias,
        "display_name": updated.display_name,
        "bio": updated.bio,
        "avatar_url": updated.avatar_url,
        "is_private": updated.is_private,
        "is_verified": updated.is_verified,
    });
    if let Err(err) = publisher.publish(subjects::UZUME_PROFILE_UPDATED, payload).await {
        // Log but don't fail the request — the DB write already succeeded.
        tracing::warn!(?err, "failed to publish profile.updated event");
    }

    Ok(ApiResponse::ok(ProfileResponse::from(updated)))
}

// ── Internal: create profile stub ────────────────────────────────────────────

/// Create a profile stub for a newly registered identity.
///
/// Called from the `profile_stub` NATS worker — not exposed over HTTP.
pub async fn create_profile_stub(
    db: &sqlx::PgPool,
    identity_id: Uuid,
    alias: &str,
    display_name: &str,
) -> Result<ProfileResponse, NyxError> {
    let insert = ProfileInsert {
        id: Uuid::now_v7(),
        identity_id,
        alias: alias.to_string(),
        display_name: display_name.to_string(),
    };

    let row = profile_queries::create_profile(db, &insert).await?;
    Ok(ProfileResponse::from(row))
}

// ── Error conversion ─────────────────────────────────────────────────────────

/// Wrapper that converts `NyxError` into an Axum HTTP response.
pub struct ApiError(NyxError);

impl From<NyxError> for ApiError {
    fn from(err: NyxError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = Json(self.0.to_error_response(None));
        (status, body).into_response()
    }
}
