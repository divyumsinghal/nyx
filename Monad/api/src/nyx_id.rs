//! Nyx ID API handlers — platform-wide unique handle management.
//!
//! These endpoints support the Instagram-style registration flow:
//! 1. Check Nyx ID availability (real-time during registration)
//! 2. Reserve Nyx ID during registration completion
//! 3. Login with Nyx ID (resolve to email for Kratos)

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use heka::{NyxIdRegistry, NyxIdStatus};

/// Shared state for Nyx ID handlers
#[derive(Clone)]
pub struct NyxIdState {
    pub registry: Arc<NyxIdRegistry>,
}

/// Request: Check if a Nyx ID is available
#[derive(Debug, Deserialize)]
pub struct CheckAvailabilityRequest {
    pub nyx_id: String,
}

/// Response: Nyx ID availability status
#[derive(Debug, Serialize)]
pub struct CheckAvailabilityResponse {
    pub nyx_id: String,
    pub available: bool,
    pub reason: Option<String>,
}

/// Request: Reserve a Nyx ID for an identity
#[derive(Debug, Deserialize)]
pub struct ReserveRequest {
    pub identity_id: String,
    pub nyx_id: String,
}

/// Response: Reservation result
#[derive(Debug, Serialize)]
pub struct ReserveResponse {
    pub success: bool,
    pub nyx_id: String,
    pub message: String,
}

/// Request: Lookup identity by Nyx ID (for login)
#[derive(Debug, Deserialize)]
pub struct LookupRequest {
    pub nyx_id: String,
}

/// Response: Identity lookup result
#[derive(Debug, Serialize)]
pub struct LookupResponse {
    pub found: bool,
    pub identity_id: Option<String>,
    pub email: Option<String>,
}

/// GET /api/nyx/id/check-availability?nyx_id=alice
/// 
/// Check if a Nyx ID is available for use.
/// Rate-limited to prevent enumeration attacks.
pub async fn check_availability(
    State(state): State<NyxIdState>,
    Query(params): Query<CheckAvailabilityRequest>,
) -> Result<Json<CheckAvailabilityResponse>, StatusCode> {
    match state.registry.check_availability(&params.nyx_id).await {
        Ok(status) => {
            let (available, reason) = match status {
                NyxIdStatus::Available => (true, None),
                NyxIdStatus::Taken => (false, Some("This Nyx ID is already taken".to_string())),
                NyxIdStatus::Invalid { reason } => (false, Some(reason)),
            };

            Ok(Json(CheckAvailabilityResponse {
                nyx_id: params.nyx_id,
                available,
                reason,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to check Nyx ID availability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/nyx/id/reserve
///
/// Reserve a Nyx ID for an identity during registration completion.
/// Called after email verification but before account activation.
pub async fn reserve_id(
    State(state): State<NyxIdState>,
    Json(body): Json<ReserveRequest>,
) -> Result<Json<ReserveResponse>, StatusCode> {
    let identity_id = match body.identity_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return Ok(Json(ReserveResponse {
                success: false,
                nyx_id: body.nyx_id,
                message: "Invalid identity ID format".to_string(),
            }));
        }
    };

    match state.registry.reserve(&identity_id, &body.nyx_id).await {
        Ok(()) => Ok(Json(ReserveResponse {
            success: true,
            nyx_id: body.nyx_id,
            message: "Nyx ID reserved successfully".to_string(),
        })),
        Err(e) => {
            tracing::warn!("Failed to reserve Nyx ID: {}", e);
            Ok(Json(ReserveResponse {
                success: false,
                nyx_id: body.nyx_id,
                message: e.to_string(),
            }))
        }
    }
}

/// GET /api/nyx/id/lookup?nyx_id=alice
///
/// Lookup an identity by Nyx ID.
/// Used during login when user enters their Nyx ID instead of email.
pub async fn lookup_by_nyx_id(
    State(state): State<NyxIdState>,
    Query(params): Query<LookupRequest>,
) -> Result<Json<LookupResponse>, StatusCode> {
    match state.registry.lookup_by_nyx_id(&params.nyx_id).await {
        Ok(Some(identity_id)) => {
            // TODO: Also fetch email from cache or database
            Ok(Json(LookupResponse {
                found: true,
                identity_id: Some(identity_id.to_string()),
                email: None, // Would need to fetch from Kratos or cache
            }))
        }
        Ok(None) => Ok(Json(LookupResponse {
            found: false,
            identity_id: None,
            email: None,
        })),
        Err(e) => {
            tracing::error!("Failed to lookup Nyx ID: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
