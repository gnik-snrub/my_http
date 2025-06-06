use std::collections::HashMap;

use bytes::BytesMut;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

pub enum ParseError {
    BadRequest,
}

pub fn parse_request(bytes: &BytesMut) -> Result<(usize, Request), ParseError> {
    for i in 1..bytes.len() {
        if bytes[i - 1] == b"\r"[0] && bytes[i] == b"\n"[0] {
            let req_chars: Vec<char> = bytes[..i].iter().map(|b| *b as char).collect();
            let req_string = req_chars[0..req_chars.len()].iter().collect::<String>();
            let req_vec: Vec<&str> = req_string.split(" ").collect();
            let method = match req_vec[0] {
                "GET" => Method::GET,
                "POST" => Method::POST,
                "PUT" => Method::PUT,
                "DELETE" => Method::DELETE,
                _ => {
                    println!("{}", req_vec[0]);
                    Method::GET
                }
            };
            let mut path_query_split: Vec<String> = req_vec[1].split("?").map(|s| s.to_string()).collect();
            let path = path_query_split[0].to_string();
            let mut query_map: HashMap<String, String> = HashMap::new();
            if path_query_split.len() > 1 {
                for string in path_query_split.iter_mut() {
                    let out = percent_decoder(string)?;
                    *string = out;
                }

                for query in path_query_split[1].split("&") {
                    let query_segments: Vec<&str> = query.split("=").collect();
                    if query_segments.len() <= 1 {
                        query_map.insert(query_segments[0].to_string(), "".to_string());
                        continue;
                    }
                    query_map.insert(query_segments[0].to_string(), query_segments[1].to_string());
                }
            }

            let request = Request {
                method,
                path,
                version: req_vec[2].to_string(),
                query: query_map,
                headers: HashMap::new(),
                body: vec![],
            };

            if !request.version.starts_with("HTTP/1.") {
                return Err(ParseError::BadRequest);
            }

            return Ok((i + 1, request))
        }
    }
    return Err(ParseError::BadRequest);
}

pub fn generate_headers(master_buffer: &mut BytesMut, idx: &mut usize) -> HashMap<String, String> {
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

pub fn generate_body(length: Option<&String>, master_buffer: &mut BytesMut, idx: usize) -> Vec<u8>{
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

fn percent_decoder(input: &str) -> Result<String, ParseError> {
    let mut iter = input.chars().peekable();
    let mut out = String::new();

    while let Some(ch) = iter.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }

        let hi = iter.next().ok_or(ParseError::BadRequest)?;
        let lo = iter.next().ok_or(ParseError::BadRequest)?;

        let hi_val = hi.to_digit(16).ok_or(ParseError::BadRequest)?;
        let lo_val = lo.to_digit(16).ok_or(ParseError::BadRequest)?;

        let byte = (hi_val * 16 + lo_val) as u8;

        if byte > 0x7F {
            return Err(ParseError::BadRequest);
        }

        out.push(byte as char);
    }
    Ok(out)
}
