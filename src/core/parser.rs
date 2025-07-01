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
    pub cookies: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug)]
pub enum ParseError {
    BadRequest,
}

pub fn parse_request(bytes: &BytesMut) -> Result<(usize, Request), ParseError> {
    for i in 1..bytes.len() {
        if bytes[i - 1] == b"\r"[0] && bytes[i] == b"\n"[0] {
            let req_chars: Vec<char> = bytes[..i].iter().map(|b| *b as char).collect();
            let req_string = req_chars[0..req_chars.len()].iter().collect::<String>();
            let req_vec: Vec<&str> = req_string.split(" ").collect();

            if req_vec.len() < 3 {
                return Err(ParseError::BadRequest);
            }

            let method = match req_vec[0] {
                "GET" => Method::GET,
                "POST" => Method::POST,
                "PUT" => Method::PUT,
                "DELETE" => Method::DELETE,
                _ => {
                    println!("Error in request method: {}", req_vec[0]);
                    return Err(ParseError::BadRequest);
                }
            };

            let mut path_query_split: Vec<String> = req_vec[1].split("?").map(|s| s.to_string()).collect();
            let path = path_query_split[0].to_string();
            if path_query_split.is_empty() || path.is_empty() {
                return Err(ParseError::BadRequest);
            }

            let version = req_vec[2].trim().to_string();
            if !version.starts_with("HTTP/1.") {
                return Err(ParseError::BadRequest);
            }

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
                version,
                query: query_map,
                headers: HashMap::new(),
                body: vec![],
                cookies: None,
            };

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
            let max = master_buffer.len();
            let end = if idx + length >= max { max } else { idx + length };
            let received_content: Vec<u8> = master_buffer[idx..end].to_vec();
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

pub fn generate_cookies(req: &Request) -> HashMap<String, String>{
    let mut cookies = HashMap::new();

    if let Some(cookies_string) = req.headers.get(&"Cookie".to_string()) {
        cookies_string.split(";").for_each(|c| {
            let cookie = c.trim();
            let pair: Vec<&str> = cookie.splitn(2, "=").collect();

            cookies.insert(pair[0].trim().to_string(), pair[1].trim().to_string());
        });
    } else {
        eprintln!("Cookies header not found");
    }


    cookies
}

#[cfg(test)]
mod tests {
    use super::*;

    // parse_request tests
    #[test]
    fn parses_basic_get_request() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"GET /index.html HTTP/1.1\r\n\r\n");
        let req = parse_request(&buf).unwrap();
        assert_eq!(req.1.method, Method::GET);
        assert_eq!(req.1.path, "/index.html");
        assert_eq!(req.1.version, "HTTP/1.1");
    }

    #[test]
    fn parses_basic_post_request() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"POST /submit HTTP/1.0\r\n\r\n");
        let req = parse_request(&buf).unwrap();
        assert_eq!(req.1.method, Method::POST);
        assert_eq!(req.1.path, "/submit");
        assert_eq!(req.1.version, "HTTP/1.0");
    }

    #[test]
    fn errors_on_missing_parts() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"GET /missingversion\r\n\r\n");
        let result = parse_request(&buf);
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn errors_on_empty_input() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"");
        let result = parse_request(&buf);
        assert!(result.is_err());
    }

    // generate_headers tests
    #[test]
    fn parses_single_header() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Host: localhost\r\n\r\n");
        let mut idx = 0;
        let headers = generate_headers(&mut buf, &mut idx);

        assert_eq!(headers.get("Host").unwrap(), "localhost");
        assert_eq!(idx, 19);
    }

    #[test]
    fn handles_multiple_headers() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Content-Type: text/plain\r\nUser-Agent: Test\r\n\r\n");
        let mut idx = 0;
        let headers = generate_headers(&mut buf, &mut idx);

        assert_eq!(headers.get("Content-Type").unwrap(), "text/plain");
        assert_eq!(headers.get("User-Agent").unwrap(), "Test");
        assert_eq!(idx, 46);
    }

    #[test]
    fn skips_malformed_lines_and_parses_valid_ones() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Good-Header: value\r\nBadHeaderLine\r\nAnother: ok\r\n\r\n");
        let mut idx = 0;
        let headers = generate_headers(&mut buf, &mut idx);

        assert_eq!(headers.get("Good-Header").unwrap(), "value");
        assert_eq!(headers.get("Another").unwrap(), "ok");
        assert!(!headers.contains_key("nBadHeaderLine"));
        assert_eq!(idx, 50);
    }

    // generate_body tests
    #[test]
    fn extracts_body_correctly() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Header: test\r\n\r\nbody data");
        let idx = 16;
        let length = "9".to_string();

        let body = generate_body(Some(&length), &mut buf, idx);
        assert_eq!(body, b"body data")
    }

    #[test]
    fn returns_empty_vec_when_length_is_zero() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Header: test\r\n\r\n");

        let idx = 16;
        let length = "0".to_string();

        let body = generate_body(Some(&length), &mut buf, idx);
        assert!(body.is_empty());
    }

    #[test]
    fn extracts_remaining_bytes_when_length_is_none() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Header: test\r\n\r\nsome data here");

        let idx = 16;

        let body = generate_body(None, &mut buf, idx);
        assert!(body.is_empty());
    }
    #[test]
    fn handles_length_larger_than_remaining_data_gracefully() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"Header: test\r\n\r\nshort");

        let idx = 16;
        let length = "999".to_string();

        let body = generate_body(Some(&length), &mut buf, idx);
        assert_eq!(body, b"short");
    }

    // generate_cookies tests
    use std::collections::HashMap;

    fn build_request_with_cookie_header(cookie_header: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert("Cookie".to_string(), cookie_header.to_string());

        Request {
            method: Method::GET,
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            query: HashMap::new(),
            headers,
            body: Vec::new(),
            cookies: None,
        }
    }

    #[test]
    fn parses_single_cookie_correct() {
        let req = build_request_with_cookie_header("sessionid=abc123");
        let cookies = generate_cookies(&req);
        assert_eq!(cookies.get("sessionid").unwrap(), "abc123");
    }

    #[test]
    fn parses_multiple_cookies() {
        let req = build_request_with_cookie_header("a=1; b=2; c=3");
        let cookies = generate_cookies(&req);
        assert_eq!(cookies.get("a").unwrap(), "1");
        assert_eq!(cookies.get("b").unwrap(), "2");
        assert_eq!(cookies.get("c").unwrap(), "3");
    }

    #[test]
    fn returns_empty_map_when_no_cookie_header_present() {
        let req = Request {
            method: Method::GET,
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            cookies: None,
        };

        let cookies = generate_cookies(&req);
        assert!(cookies.is_empty());
    }

    // percent_decoder tests
    #[test]
    fn decodes_basic_percent_encoding() {
        let input = "hello%20world".to_string();
        let decoded = percent_decoder(&input);
        assert_eq!(decoded.unwrap(), "hello world");
    }

    #[test]
    fn decodes_multiple_encodings() {
        let input = "%48%65%6C%6C%6F".to_string();
        let decoded = percent_decoder(&input);
        assert_eq!(decoded.unwrap(), "Hello");
    }

    #[test]
    fn ignores_invalid_percent_sequence() {
        let input = "bad%2Gdata%".to_string();
        let result = percent_decoder(&input);
        assert!(result.is_err());
    }

    #[test]
    fn handles_no_encodings_gracefully() {
        let input = "cleanpath".to_string();
        let decoded = percent_decoder(&input);
        assert_eq!(decoded.unwrap(), "cleanpath");
    }
}
