use std::net::SocketAddr;

use axum::{middleware, routing::get, Router};
use nun::config::NyxConfig;

use crate::middleware::{app_context::app_context, request_id::request_id, tracing::trace_request};

#[derive(Clone)]
pub struct NyxServer {
    router: Router,
    addr: SocketAddr,
}

pub struct NyxServerBuilder {
    config: Option<NyxConfig>,
    routes: Option<Router>,
}

impl NyxServer {
    pub fn builder() -> NyxServerBuilder {
        NyxServerBuilder {
            config: None,
            routes: None,
        }
    }

    pub async fn serve(self) -> Result<(), std::io::Error> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, self.router).await
    }

    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

impl NyxServerBuilder {
    pub fn with_config(mut self, config: NyxConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_routes(mut self, routes: Router) -> Self {
        self.routes = Some(routes);
        self
    }

    pub fn build(self) -> nun::Result<NyxServer> {
        let config = self.config.ok_or_else(|| {
            nun::NyxError::bad_request("missing_config", "NyxServer config is required")
        })?;

        let app = self.routes.unwrap_or_default();

        let router = app
            .route("/healthz", get(|| async { "ok" }))
            .layer(middleware::from_fn(trace_request))
            .layer(middleware::from_fn(request_id))
            .layer(middleware::from_fn(app_context));

        let addr: SocketAddr = config
            .server
            .addr()
            .parse()
            .map_err(|e| nun::NyxError::bad_request("invalid_server_addr", format!("{e}")))?;

        Ok(NyxServer { router, addr })
    }
}
