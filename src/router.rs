use serde_json::{json, Value};

use crate::parser::{Request, Method};
use crate::response::{Response, StatusCode};

pub fn router(req: Request) -> Response {
    match (&req.method, req.path.as_str()) {
        (Method::GET, "/")      => handle_root_get(req),
        (Method::POST, "/")     => handle_root_post(req),
        (Method::PUT, "/")      => handle_unallowed_method(),
        (Method::DELETE, "/")   => handle_unallowed_method(),

        (Method::GET, "/echo")  => handle_echo_get(req),
        (Method::POST, "/echo")  => handle_unallowed_method(),
        (Method::PUT, "/echo")  => handle_unallowed_method(),
        (Method::DELETE, "/echo")  => handle_unallowed_method(),

        _                       => Response::not_found()
    }
}

fn handle_root_get(_req: Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&"Hello")
}

fn handle_root_post(req: Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&req.body)
}

fn handle_echo_get(req: Request) -> Response {
    let mut json = json!({});
    for q in req.query.iter() {
        json[q.0] = Value::String(q.1.to_string())
    }
    Response::new().status(StatusCode::Ok).json(&json)
}

fn handle_unallowed_method() -> Response {
    Response::new().status(StatusCode::MethodNotAllowed).text(&"405 Method Not Allowed")
}
