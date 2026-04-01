pub mod highlights;
pub mod stories;

use axum::{Json, http::StatusCode, response::IntoResponse};
use nun::NyxError;

/// Wrapper that converts `NyxError` into an Axum response.
pub struct ApiError(pub NyxError);

impl From<NyxError> for ApiError {
    fn from(err: NyxError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = Json(self.0.to_error_response(None));
        (status, body).into_response()
    }
}
