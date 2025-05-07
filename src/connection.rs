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
    method: String,
    path: String,
    version: String,
}

fn parse_request(bytes: &Vec<u8>) -> Option<(usize, Request)> {
    for i in 1..bytes.len() {
        if bytes[i - 1] == b"\r"[0] && bytes[i] == b"\n"[0] {
            let req_chars: Vec<char> = bytes[..i].iter().map(|b| *b as char).collect();
            let req_string = req_chars[0..req_chars.len()].iter().collect::<String>();
            let req_vec: Vec<&str> = req_string.split(" ").collect();
            let request = Request {
                method: req_vec[0].to_string(),
                path: req_vec[1].to_string(),
                version: req_vec[2].to_string(),
            };

            if !request.version.starts_with("HTTP/1.") {
                return None
            }

            return Some((i + 1, request))
        }
    }
    None
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
