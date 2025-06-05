use serde::Serialize;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum StatusCode {
    Ok,                     // 200
    BadRequest,             // 400
    Unauthorized,           // 401
    NotFound,               // 404
    MethodNotAllowed,       // 405
    InternalError,          // 500
}

impl Response {
    pub fn new() -> Response {
        Response { status: StatusCode::NotFound, headers: HashMap::new(), body: vec![] }
    }

    pub fn not_found() -> Response {
        Response { status: StatusCode::NotFound, headers: HashMap::new(), body: Vec::from(b"404 Not Found") }
    }

    pub fn text<T: AsRef<[u8]>>(mut self, body: &T) -> Response {
        self.headers.insert("content-type".to_string(), "text/plain; charset=utf-8".to_string());
        self.body = body.as_ref().to_vec();
        self
    }

    pub fn html(mut self, body: &str) -> Response {
        self.headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
        self.body = body.bytes().collect();
        self
    }

    pub fn json<T: Serialize>(mut self, json: &T) -> Response {
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        self.body = match serde_json::to_vec(json) {
            Ok(ok) => ok,
            Err(_) => {
                self = self.header("x-serialize-error", "true").status(StatusCode::InternalError);
                b"{\"Error\": \"Could not serialize JSON\"}".to_vec()
            },
        };
        self
    }

    pub fn status(mut self, status: StatusCode) -> Response {
        self.status = status;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Response {
        if key.contains(":") || key.contains("\r") || key.contains("\n") {
            println!("Error: Invalid header entered");
            return self
        }
        let lower = key.trim().to_lowercase();
        self.headers.insert(lower.to_string(), value.to_string());
        self
    }

    pub fn finalize(&mut self) -> Vec<u8> {
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());
        let mut buffer = String::from("HTTP/1.1 ");
        match &self.status {
            StatusCode::Ok => {
                buffer += "200 OK\r\n";
            }
            StatusCode::NotFound => {
                buffer += "404 Not Found\r\n";
            }
            StatusCode::BadRequest => {
                buffer += "400 Bad Request\r\n";
            }
            StatusCode::MethodNotAllowed => {
                buffer += "405 Method Not Allowed\r\n";
            }
            StatusCode::InternalError => {
                buffer += "500 Internal Error\r\n";
            }
        }
        for (key, val) in self.headers.iter() {
            buffer += format!("{}: {}\r\n", key, val).as_str();
        }
        buffer += "\r\n";
        let mut bytes = buffer.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}
