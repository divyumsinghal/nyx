use axum::extract::{FromRequestParts, Query};
use axum::http::{request::Parts, StatusCode};
use nun::pagination::PageRequest;

#[derive(Debug, Clone)]
pub struct CursorPagination(pub PageRequest);

impl<S> FromRequestParts<S> for CursorPagination
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(page) = Query::<PageRequest>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        Ok(Self(page))
    }
}
