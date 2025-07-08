use super::*;
use serde_json::json;

#[test]
fn basic_ok_response_with_body() {
    let mut res = Response::new()
        .status(StatusCode::Ok)
        .text(&"Hello World");

    let bytes = res.finalize();
    let response = String::from_utf8_lossy(&bytes);

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains("content-type: text/plain; charset=utf-8\r\n"));
    assert!(response.contains("content-length: 11\r\n"));
    assert!(response.ends_with("Hello World"))
}

#[test]
fn content_type_helpers_work_correctly() {
    let mut text = Response::new().status(StatusCode::Ok).text(&"text");
    let mut html = Response::new().status(StatusCode::Ok).html("<p>html</p>");
    let mut json = Response::new().status(StatusCode::Ok).json(&json!({"a": 1}));

    let text_str = String::from_utf8_lossy(&text.finalize()).into_owned();
    let html_str = String::from_utf8_lossy(&html.finalize()).into_owned();
    let json_str = String::from_utf8_lossy(&json.finalize()).into_owned();

    assert!(text_str.contains("content-type: text/plain; charset=utf-8"));
    assert!(html_str.contains("content-type: text/html; charset=utf-8"));
    assert!(json_str.contains("content-type: application/json"));
    assert!(json_str.contains("{\"a\":1}"));
}

#[test]
fn json_serialization_failure_fallback() {
    use serde::ser::{Serialize, Serializer};
    struct BadJson {
        #[allow(dead_code)]
        bad: f64,
    }

    impl Serialize for BadJson {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer {
                Err(serde::ser::Error::custom("intentional failure"))
        }
    }

    let data = BadJson { bad: std::f64::NAN };
    let mut res = Response::new().status(StatusCode::Ok).json(&data);
    let body = String::from_utf8_lossy(&res.finalize()).into_owned();

    assert!(body.contains("HTTP/1.1 500 Internal Error"));
    assert!(body.contains("x-serialize-error: true"));
    assert!(body.contains("{\"Error\": \"Could not serialize JSON\"}"));
}

#[test]
fn valid_and_invalikd_custom_headers() {
    let good = Response::new().header("X-Test", "123");
    let bad = Response::new().header("Bad:Header", "nope");

    assert!(good.headers.contains_key("x-test"));
    assert!(!bad.headers.contains_key("bad:header"));
}

#[test]
fn not_found_response() {
    let mut res = Response::not_found();
    let out = String::from_utf8_lossy(&res.finalize()).into_owned();

    assert!(out.contains("HTTP/1.1 404 Not Found"));
    assert!(out.ends_with("404 Not Found"));
}

#[test]
fn empty_response_has_correct_headers() {
    let mut res = Response::new();
    let out = String::from_utf8_lossy(&res.finalize()).into_owned();

    assert!(out.contains("HTTP/1.1 404 Not Found"));
    assert!(out.contains("content-length: 0"));
    assert!(out.ends_with("\r\n\r\n"));
}

#[test]
fn overriding_headers_works() {
    let mut res = Response::new()
        .text(&"some")
        .header("Content-Type", "override/type");
    let out = String::from_utf8_lossy(&res.finalize()).into_owned();

    assert!(out.contains("content-type: override/type"));
    assert!(!out.contains("text/plain; charset=utf-8"));
}
