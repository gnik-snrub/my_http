use std::{future::Future, pin::Pin, sync::Arc, time::SystemTime};

use async_trait::async_trait;

use crate::{parser::Request, response::{Response, StatusCode}};

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

pub struct AddHeader;
#[async_trait]
impl Middleware for AddHeader {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let res = next(req).await.header("X-Example", "It works! :D");
        res
    }
}

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

pub struct Auth;
#[async_trait]
impl Middleware for Auth {
    async fn handle(&self, req: Request, next: Next) -> Response {
        if req.headers.get("Authorization").is_none() {
            return Response::new().status(StatusCode::Unauthorized);
        }

        let res = next(req).await;

        res
    }
}

pub struct Timer;
#[async_trait]
impl Middleware for Timer {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let now = SystemTime::now();

        let res = next(req).await;

        match now.elapsed() {
            Ok(duration) => {
                res.header("X-Duration", duration.as_nanos().to_string().as_str())
            }
            Err(_) => {
                res
            }
        }
    }
}
