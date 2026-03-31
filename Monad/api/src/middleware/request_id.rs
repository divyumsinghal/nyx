use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub async fn request_id(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    req.extensions_mut().insert(request_id.clone());
    let mut res = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        res.headers_mut().insert(REQUEST_ID_HEADER.clone(), value);
    }
    res
}
