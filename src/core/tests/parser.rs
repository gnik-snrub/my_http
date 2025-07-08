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
