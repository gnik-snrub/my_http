use async_trait::async_trait;

use crate::core::{parser::Request, response::Response};

use super::{Middleware, Next};

pub struct Logger;
#[async_trait]
impl Middleware for Logger {
    async fn handle(&self, req: Request, next: Next) -> Response {
        tracing::info!(
            method  = ?req.method,
            path    = %req.path,
            query   = ?req.query,
            "request_received"
        );

        next(req).await
    }
}
