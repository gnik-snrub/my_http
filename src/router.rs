use std::time::Duration;

use serde_json::{json, Value};

use crate::parser::{Request, Method};
use crate::response::{Response, StatusCode};

pub async fn router(req: Request) -> Response {
    let res = match (&req.method, req.path.as_str()) {
        (Method::GET, "/")      => handle_root_get(&req).await,
        (Method::POST, "/")     => handle_root_post(&req).await,
        (Method::PUT, "/")      => handle_unallowed_method().await,
        (Method::DELETE, "/")   => handle_unallowed_method().await,

        (Method::GET, "/echo")  => handle_echo_get(&req).await,
        (Method::POST, "/echo")  => handle_unallowed_method().await,
        (Method::PUT, "/echo")  => handle_unallowed_method().await,
        (Method::DELETE, "/echo")  => handle_unallowed_method().await,

        (Method::GET, "/sleep")  => handle_sleep().await,

        _                       => Response::not_found()
    };

    tracing::info!(
        method  = ?req.method,
        path    = %req.path,
        query   = ?req.query,
        status  = ?res.status,
        len     = res.body.len(),
        "served_request"
    );

    res
}

async fn handle_root_get(_req: &Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&"Hello")
}

async fn handle_root_post(req: &Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&req.body)
}

async fn handle_echo_get(req: &Request) -> Response {
    let mut json = json!({});
    for q in req.query.iter() {
        json[q.0] = Value::String(q.1.to_string())
    }
    Response::new().status(StatusCode::Ok).json(&json)
}

async fn handle_unallowed_method() -> Response {
    Response::new().status(StatusCode::MethodNotAllowed).text(&"405 Method Not Allowed")
}

async fn handle_sleep() -> Response {
    println!("Sleeping...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    Response::new().status(StatusCode::Ok).text(&"Slept for 5 seconds")
}
