use std::{collections::HashMap, io::{Read, Write}, net::TcpStream};


pub fn handle_client(stream: &mut TcpStream) {
    println!("Hello world");
    println!("Connected stream: {:?}", stream);

    let mut master_buffer: Vec<u8> = vec![];
    let mut scratch: [u8; 512] = [0u8; 512];

    collect_stream(stream, &mut scratch, &mut master_buffer);

    let (mut idx, mut req) = parse_request(&master_buffer);
    req.headers = generate_headers(&mut master_buffer, &mut idx);
    req.body = generate_body(req.headers.get("Content-Length"), &mut master_buffer, idx);

    let response = router(req);
    send_response(stream, response.finalize().to_vec());
}

fn router(req: Request) -> Response {
    match (&req.method, req.path.as_str()) {
        (Method::GET, "/")      => handle_root_get(req),
        (Method::POST, "/")     => handle_root_post(req),
        _                       => Response::not_found()
    }
}

fn handle_root_get(req: Request) -> Response {
    Response::new(StatusCode::Ok, req.headers, Vec::from(b"Hello"))
}

fn handle_root_post(req: Request) -> Response {
    Response::new(StatusCode::Ok, req.headers, req.body)
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

#[derive(Debug)]
struct Request {
    method: Method,
    path: String,
    version: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum Method {
    GET,
    POST,
    ERROR,
}

fn parse_request(bytes: &Vec<u8>) -> (usize, Request) {
    let default_request = Request {
        method: Method::ERROR,
        path: "".to_string(),
        version: "".to_string(),
        query: HashMap::new(),
        headers: HashMap::new(),
        body: vec![],
    };

    for i in 1..bytes.len() {
        if bytes[i - 1] == b"\r"[0] && bytes[i] == b"\n"[0] {
            let req_chars: Vec<char> = bytes[..i].iter().map(|b| *b as char).collect();
            let req_string = req_chars[0..req_chars.len()].iter().collect::<String>();
            let query_split: Vec<&str> = req_string.split("=").collect();
            let req_vec: Vec<&str> = query_split[0].split(" ").collect();
            let method = match req_vec[0] {
                "GET" => Method::GET,
                "POST" => Method::POST,
                _ => {
                    println!("{}", req_vec[0]);
                    Method::GET
                }
            };
            let mut query_map: HashMap<String, String> = HashMap::new();
            if query_split.len() > 1 {
                for query in query_split[1].split("&") {
                    let query_segments: Vec<&str> = query.split("=").collect();
                    if !query_segments.len() > 1 {
                        query_map.insert(query_segments[0].to_string(), "".to_string());
                        continue;
                    }
                    query_map.insert(query_segments[0].to_string(), query_segments[1].to_string());
                }
            }
            let request = Request {
                method,
                path: req_vec[1].to_string(),
                version: req_vec[2].to_string(),
                query: query_map,
                headers: HashMap::new(),
                body: vec![],
            };

            if !request.version.starts_with("HTTP/1.") {
                return (i + 1, default_request);
            }

            return (i + 1, request)
        }
    }
    return (0, default_request);
}

fn generate_headers(master_buffer: &mut Vec<u8>, idx: &mut usize) -> HashMap<String, String> {
    let mut header_chars = vec![];

    while !header_chars.ends_with(&['\r', '\n', '\r', '\n']) {
        header_chars.push(master_buffer[*idx] as char);
        *idx += 1;
    }
    let header_string = header_chars[0..header_chars.len()].iter().collect::<String>();

    let mut header_map: HashMap<String, String> = HashMap::new();
    for item in header_string.split("\r\n") {
        let pair: Vec<&str> = item.split(": ").collect();
        if pair.len() > 1 {
            header_map.insert(pair[0].to_string(), pair[1].to_string());
        }
    }
    header_map
}

fn generate_body(length: Option<&String>, master_buffer: &mut Vec<u8>, idx: usize) -> Vec<u8>{
    match length {
        Some(len) => {
            let parsed = len.parse::<usize>();
            let length = if parsed.is_ok() {
                parsed.unwrap()
            } else {
                0
            };
            let received_content: Vec<u8> = master_buffer[idx..idx + length].to_vec();
            return received_content;
        },
        None => {
            vec![]
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
    Ok,
    NotFound,
}

impl Response {
    fn new(status: StatusCode, headers: HashMap<String, String>, body: Vec<u8>) -> Response {
        Response { status , headers, body }
    }

    fn not_found() -> Response {
        Response { status: StatusCode::NotFound, headers: HashMap::new(), body: Vec::from(b"404 Not Found") }
    }

    fn finalize(&self) -> Vec<u8> {
        let mut buffer = String::from("HTTP/1.1 ");
        match &self.status {
            StatusCode::Ok => {
                buffer += "200 OK\r\n";
            }
            StatusCode::NotFound => {
                buffer += "404 Not Found\r\n";
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
