pub mod add_header;
pub mod auth;
pub mod logger;
pub mod timer;
pub mod set_cookie;
pub mod session_tracker;

use std::{future::Future, pin::Pin, sync::Arc };

use async_trait::async_trait;

use crate::core::{parser::Request, response::Response};

#[derive(Clone)]
pub struct Dispatcher {
    middleware: Vec<Arc<dyn Middleware>>,
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {
            middleware: vec![]
        }
    }

    pub fn add(&mut self, mw: impl Middleware + 'static) {
        self.middleware.push(Arc::new(mw));
    }

    pub async fn dispatch(&self, req: Request) -> Response {
        let handler: Next = Arc::new(|_|
            Box::pin(async move {
                Response::new()
            })
        );

        let composed: Next = self.middleware
            .iter()
            .rev()
            .fold(handler, |next, mw| {
                let mw = Arc::clone(mw);
                Arc::new(move |req: Request| {
                    let next = Arc::clone(&next);
                    let cloned_middleware = mw.clone();
                    Box::pin(async move {
                        cloned_middleware.handle(req, next).await
                    })
                })
            });
        composed(req).await
    }
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Response;
}

type Next = Arc<dyn Fn(Request) -> ResponseFuture + Send + Sync>;
type ResponseFuture = Pin<Box<dyn Future<Output = Response> + Send>>;

#[cfg(test)]
#[path ="tests/middleware.rs"]
mod middleware_tests;
