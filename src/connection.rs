use std::{collections::HashMap, io::{Read, Write}, net::TcpStream};


pub fn handle_client(stream: &mut TcpStream) {
    println!("Hello world");
    println!("Connected stream: {:?}", stream);

    let mut master_buffer: Vec<u8> = vec![];
    let mut scratch: [u8; 512] = [0u8; 512];

    collect_stream(stream, &mut scratch, &mut master_buffer);

    let request = parse_request(&master_buffer);
    match request {
        Some((mut idx, req)) => {
            let mut header_chars = vec![];
            //let header_chars: Vec<char> = master_buffer[header_start_idx..].iter().map(|b| *b as char).collect();
            while !header_chars.ends_with(&['\r', '\n', '\r', '\n']) {
                header_chars.push(master_buffer[idx] as char);
                idx += 1;
            }
            let header_string = header_chars[0..header_chars.len()].iter().collect::<String>();
            let headers = generate_headers(header_string);

            println!("Request: {:?}", req);
            println!("Headers: {:?}", headers);
            if req.path != "/".to_string() {
                let response = b"HTTP/1.1 404 Not Found\r\n\r\n404 Not Found\n";
                send_response(stream, response.to_vec());
                return;
            }

            if req.method == "GET".to_string() {
                let response = b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nHello\n";
                send_response(stream, response.to_vec());
                return;
            }

            if req.method == "POST".to_string() {
                match headers.get("Content-Length") {
                    Some(len) => {
                        let parsed = len.parse::<usize>();
                        let length = if parsed.is_ok() {
                            parsed.unwrap()
                        } else {
                            0
                        };
                        let received_content = &master_buffer[idx..idx + length];
                        let body = received_content.iter().map(|b| *b as char).collect::<String>();
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}\n", body.len() + 1, body).into_bytes();
                        send_response(stream, response);
                        return;
                    }
                    None => {

                    }
                }
            }

        },
        None => {
            println!("Error in request, disconnecting...");
            return;
        }
    }
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

fn generate_headers(header_string: String) -> HashMap<String, String> {
    let mut header_map: HashMap<String, String> = HashMap::new();
    for item in header_string.split("\r\n") {
        let pair: Vec<&str> = item.split(": ").collect();
        if pair.len() > 1 {
            header_map.insert(pair[0].to_string(), pair[1].to_string());
        }
    }
    header_map
}

fn send_response(stream: &mut TcpStream ,res_bytes: Vec<u8>) {
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
