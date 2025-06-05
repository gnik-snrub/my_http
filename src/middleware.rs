use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;

use crate::{parser::Request, response::Response};

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
