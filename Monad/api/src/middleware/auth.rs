use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub token: String,
}

pub async fn auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let bearer = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();

    let user_id = Uuid::now_v7();
    req.extensions_mut().insert(AuthContext {
        user_id,
        token: bearer,
    });

    Ok(next.run(req).await)
}
