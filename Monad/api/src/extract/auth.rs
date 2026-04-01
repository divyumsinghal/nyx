use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

use crate::middleware::auth::AuthContext;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<AuthContext>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(Self {
            user_id: ctx.user_id,
        })
    }
}
