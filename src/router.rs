use serde_json::{json, Value};

use crate::parser::{Request, Method};
use crate::response::{Response, StatusCode};

pub async fn router(req: Request) -> Response {
    match (&req.method, req.path.as_str()) {
        (Method::GET, "/")      => handle_root_get(req).await,
        (Method::POST, "/")     => handle_root_post(req).await,
        (Method::PUT, "/")      => handle_unallowed_method().await,
        (Method::DELETE, "/")   => handle_unallowed_method().await,

        (Method::GET, "/echo")  => handle_echo_get(req).await,
        (Method::POST, "/echo")  => handle_unallowed_method().await,
        (Method::PUT, "/echo")  => handle_unallowed_method().await,
        (Method::DELETE, "/echo")  => handle_unallowed_method().await,

        _                       => Response::not_found()
    }
}

async fn handle_root_get(_req: Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&"Hello")
}

async fn handle_root_post(req: Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&req.body)
}

async fn handle_echo_get(req: Request) -> Response {
    let mut json = json!({});
    for q in req.query.iter() {
        json[q.0] = Value::String(q.1.to_string())
    }
    Response::new().status(StatusCode::Ok).json(&json)
}

async fn handle_unallowed_method() -> Response {
    Response::new().status(StatusCode::MethodNotAllowed).text(&"405 Method Not Allowed")
}
