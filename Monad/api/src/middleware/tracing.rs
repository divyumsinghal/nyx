use axum::{extract::Request, middleware::Next, response::Response};

pub async fn trace_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    tracing::debug!(%method, %path, "request_start");
    let res = next.run(req).await;
    tracing::debug!(status = %res.status(), "request_end");
    res
}
