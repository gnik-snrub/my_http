use std::{collections::HashMap, io::{Read, Write}, net::TcpStream};
use serde::Serialize;

use crate::parser::{generate_body, generate_headers, parse_request, Request, Method};

pub fn handle_client(stream: &mut TcpStream) {
    println!("Hello world");
    println!("Connected stream: {:?}", stream);

    let mut master_buffer: Vec<u8> = vec![];
    let mut scratch: [u8; 512] = [0u8; 512];

    collect_stream(stream, &mut scratch, &mut master_buffer);

    let parse_result = parse_request(&master_buffer);
    let mut response = match parse_result {
        Ok((mut idx, mut req)) => {
            req.headers = generate_headers(&mut master_buffer, &mut idx);
            req.body = generate_body(req.headers.get("Content-Length"), &mut master_buffer, idx);

            router(req)
        }
        Err(_) => {
            Response::new().status(StatusCode::BadRequest)
        }
    };
    send_response(stream, response.finalize().to_vec());
}

fn router(req: Request) -> Response {
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

fn collect_stream(stream: &mut TcpStream, scratch: &mut [u8; 512], master_buffer: &mut Vec<u8>) {
    loop {
        if master_buffer.len() >= 8000 {
            break;
        }
        let bytes_read = stream.read(scratch);
        match bytes_read {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                master_buffer.extend_from_slice(&scratch[..n]);
            },
            Err(e) => {
                println!("Error reading stream data: {:?}", e);
            }
        }
        for window in master_buffer.windows(4) {
            if window == [13, 10, 13, 10] {
                return;
            }
        }
    }
}

fn send_response(stream: &mut TcpStream, res_bytes: Vec<u8>) {
    let result = stream.write_all(&res_bytes);
    match result {
        Ok(_) => {
            println!("Response sent...");
        }
        Err(e) => {
            println!("Error sending response: {:?}", e);
        }
    }
    let _ = stream.flush();
}

struct Response {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

enum StatusCode {
    Ok,                     // 200
    BadRequest,             // 400
    NotFound,               // 404
    MethodNotAllowed,       // 405
    InternalError,          // 500
}

impl Response {
    fn new() -> Response {
        Response { status: StatusCode::NotFound, headers: HashMap::new(), body: vec![] }
    }

    fn not_found() -> Response {
        Response { status: StatusCode::NotFound, headers: HashMap::new(), body: Vec::from(b"404 Not Found") }
    }

    fn text<T: AsRef<[u8]>>(mut self, body: &T) -> Response {
        self.headers.insert("content-type".to_string(), "text/plain; charset=utf-8".to_string());
        self.body = body.as_ref().to_vec();
        self
    }

    fn html(mut self, body: &str) -> Response {
        self.headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
        self.body = body.bytes().collect();
        self
    }

    fn json<T: Serialize>(mut self, json: &T) -> Response {
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

    fn status(mut self, status: StatusCode) -> Response {
        self.status = status;
        self
    }

    fn header(mut self, key: &str, value: &str) -> Response {
        if !key.contains(":") || !key.contains("\r") || !key.contains("\n") {
            println!("Error: Invalid header entered");
            return self
        }
        let lower = key.trim().to_lowercase();
        self.headers.insert(lower.to_string(), value.to_string());
        self
    }

    fn finalize(&mut self) -> Vec<u8> {
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
