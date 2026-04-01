pub mod explore;
pub mod search;

use axum::{http::StatusCode, response::IntoResponse, Json};
use nun::NyxError;

/// Wrapper that converts [`NyxError`] into an Axum HTTP response.
///
/// The HTTP status code is derived from [`NyxError::status_code`], and the
/// body follows the platform-wide `ErrorResponse` shape from [`NyxError::to_error_response`].
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
