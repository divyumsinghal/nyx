use axum::{extract::Request, middleware::Next, response::Response};
use nun::types::NyxApp;

#[derive(Debug, Clone, Copy)]
pub struct AppContext {
    pub app: NyxApp,
}

pub async fn app_context(mut req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let app = if path.starts_with("/uzume") {
        NyxApp::Uzume
    } else if path.starts_with("/anteros") {
        NyxApp::Anteros
    } else {
        NyxApp::Themis
    };

    req.extensions_mut().insert(AppContext { app });
    next.run(req).await
}
