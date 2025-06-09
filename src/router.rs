use std::time::Duration;

use serde_json::{json, Value};

use crate::file_serving::serve_file;
use crate::parser::{Request, Method};
use crate::response::{Response, StatusCode};

pub async fn router(req: Request, res: Response) -> Response {
    let res = match (&req.method, req.path.as_str()) {
        (Method::GET, "/")      => handle_root_get(&req, res).await,
        (Method::POST, "/")     => handle_root_post(&req, res).await,
        (Method::PUT, "/")      => handle_unallowed_method().await,
        (Method::DELETE, "/")   => handle_unallowed_method().await,

        (Method::GET, "/echo")  => handle_echo_get(&req, res).await,
        (Method::POST, "/echo")  => handle_unallowed_method().await,
        (Method::PUT, "/echo")  => handle_unallowed_method().await,
        (Method::DELETE, "/echo")  => handle_unallowed_method().await,

        (Method::GET, "/page")  => handle_page_get(&req, res).await,

        (Method::GET, "/sleep")  => handle_sleep().await,

        (Method::GET, path) if path.starts_with("/static/") => handle_static(&req, res).await,

        _                       => Response::not_found()
    };

    tracing::info!(
        status  = ?res.status,
        len     = res.body.len(),
        "served_request"
    );

    res
}

async fn handle_root_get(_req: &Request, res: Response) -> Response {
    res.status(StatusCode::Ok).text(&"Hello")
}

async fn handle_root_post(req: &Request, res: Response) -> Response {
    res.status(StatusCode::Ok).text(&req.body)
}

async fn handle_echo_get(req: &Request, res: Response) -> Response {
    let mut json = json!({});
    for q in req.query.iter() {
        json[q.0] = Value::String(q.1.to_string())
    }
    res.status(StatusCode::Ok).json(&json)
}

async fn handle_page_get(_req: &Request, res: Response) -> Response {
    let html = "<html></html>";
    res.status(StatusCode::Ok).html(html)
}

async fn handle_unallowed_method() -> Response {
    Response::new().status(StatusCode::MethodNotAllowed).text(&"405 Method Not Allowed")
}

async fn handle_sleep() -> Response {
    println!("Sleeping...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    Response::new().status(StatusCode::Ok).text(&"Slept for 5 seconds")
}

async fn handle_static(req: &Request, res: Response) -> Response {
    let file_response = serve_file(&req, res).await;
    file_response
}
