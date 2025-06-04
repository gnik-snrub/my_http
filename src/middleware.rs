use std::{future::Future, pin::Pin};

use async_trait::async_trait;

use crate::{parser::Request, response::Response};
#[async_trait]
trait Middleware: Send + Sync {
    async fn handle(
        &self,
        req: Request,
        next: Box<dyn FnOnce(Request) -> ResponseFuture + Send>,
    ) -> Response;
}

type ResponseFuture = Pin<Box<dyn Future<Output = Response> + Send>>;
