use super::*;
use crate::core::{parser::{Request, Method}, response::{Response, StatusCode}};
use async_trait::async_trait;
use std::{collections::HashMap, sync::{Arc, Mutex}};

#[derive(Clone)]
struct DummyMiddleware {
    label: String,
    log: Arc<Mutex<Vec<String>>>
}

#[async_trait]
impl Middleware for DummyMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        self.log.lock().unwrap().push(self.label.clone());
        next(req).await
    }
}

#[tokio::test]
async fn dispatch_returns_default_response_with_no_middleware() {
    let dispatcher = Dispatcher::new();
    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        headers: Default::default(),
        body: vec![],
        version: "HTTP 1.1".to_string(),
        query: HashMap::new(),
        cookies: None,
    };

    let res = dispatcher.dispatch(req).await;
    println!("{:?}", res);
    assert_eq!(res.status, StatusCode::NotFound);
}

#[tokio::test]
async fn single_middleware_is_invoked() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut dispatcher = Dispatcher::new();
    dispatcher.add(DummyMiddleware {
        label: "MW1".into(),
        log: log.clone(),
    });

    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        headers: Default::default(),
        body: vec![],
        version: "HTTP 1.1".to_string(),
        query: HashMap::new(),
        cookies: None,
    };

    dispatcher.dispatch(req).await;

    let logs = log.lock().unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0], "MW1");
}

#[tokio::test]
async fn middleware_is_invoked_in_reverse_order() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut dispatcher = Dispatcher::new();

    dispatcher.add(DummyMiddleware {
        label: "First".into(),
        log: log.clone(),
    });

    dispatcher.add(DummyMiddleware {
        label: "Second".into(),
        log: log.clone(),
    });

    dispatcher.add(DummyMiddleware {
        label: "Third".into(),
        log: log.clone(),
    });

    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        headers: Default::default(),
        body: vec![],
        version: "HTTP 1.1".to_string(),
        query: HashMap::new(),
        cookies: None,
    };

    dispatcher.dispatch(req).await;

    let logs = log.lock().unwrap();
    assert_eq!(&logs[..], ["First", "Second", "Third"]);
}
