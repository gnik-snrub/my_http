use crate::parser::{Request, Method};
use crate::response::{Response, StatusCode};

pub fn router(req: Request) -> Response {
    match (&req.method, req.path.as_str()) {
        (Method::GET, "/")      => handle_root_get(req),
        (Method::POST, "/")     => handle_root_post(req),
        (Method::PUT, "/")      => handle_unallowed_method(),
        (Method::DELETE, "/")   => handle_unallowed_method(),
        _                       => Response::not_found()
    }
}

fn handle_root_get(_req: Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&"Hello")
}

fn handle_root_post(req: Request) -> Response {
    Response::new().status(StatusCode::Ok).text(&req.body)
}

fn handle_unallowed_method() -> Response {
    Response::new().status(StatusCode::MethodNotAllowed).text(&"405 Method Not Allowed")
}
