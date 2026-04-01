use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CursorMeta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<CursorMeta>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data,
            pagination: None,
        }
    }

    pub fn paginated(data: T, next_cursor: Option<String>, has_more: bool) -> Self {
        Self {
            data,
            pagination: Some(CursorMeta {
                next_cursor,
                has_more,
            }),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
