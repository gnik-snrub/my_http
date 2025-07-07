use super::*;

#[test]
fn returns_correct_mime_for_html() {
    assert_eq!(get_mime_type("index.html"), "text/html");
}

#[test]
fn returns_correct_mime_for_css() {
    assert_eq!(get_mime_type("styles.css"), "text/css");
}

#[test]
fn returns_correct_mime_for_js() {
    assert_eq!(get_mime_type("app.js"), "application/javascript");
}

#[test]
fn returns_correct_mime_for_png() {
    assert_eq!(get_mime_type("image.png"), "image/png");
}

#[test]
fn returns_octet_stream_for_unknown_extension() {
    assert_eq!(get_mime_type("data.xyz"), "application/octet-stream");
}

#[test]
fn returns_empty_string_for_no_extension() {
    assert_eq!(get_mime_type("README"), "");
}

use std::fs::{create_dir_all, write, remove_file, remove_dir_all};
use crate::core::parser::{Request, Method};
use crate::core::response::{Response, StatusCode};
use tokio;

fn make_request(path: &str) -> Request {
    Request {
        method: Method::GET,
        path: path.to_string(),
        headers: Default::default(),
        body: vec![],
        version: "HTTP 1.1".to_string(),
        query: HashMap::new(),
        cookies: None
    }
}

#[tokio::test]
async fn serves_existing_file() {
    create_dir_all("public").unwrap();
    write("public/test.txt", b"hello").unwrap();

    let req = make_request("/static/test.txt");
    let res = serve_file(&req, Response::new()).await;
    let body = String::from_utf8_lossy(&res.body);
    
    assert_eq!(res.status, StatusCode::Ok);
    assert_eq!(body, "hello");
    assert!(res.headers.get("content-type").unwrap().contains("text/plain"));

    remove_file("public/test.txt").unwrap()
}

#[tokio::test]
async fn returns_404_for_missing_file() {
    let req = make_request("/static/does_not_exist.txt");
    let res = serve_file(&req, Response::new()).await;
    assert_eq!(res.status, StatusCode::NotFound);
}

#[tokio::test]
async fn serves_directory_listing_as_json() {
    create_dir_all("public/dir_test").unwrap();
    write("public/dir_test/file1.txt", b"test").unwrap();
    write("public/dir_test/file2.txt", b"test").unwrap();

    let req = make_request("/static/dir_test");
    let res = serve_file(&req, Response::new()).await;
    assert_eq!(res.status, StatusCode::Ok);

    let body = String::from_utf8_lossy(&res.body);
    assert!(body.contains("file1.txt"));
    assert!(body.contains("file2.txt"));

    remove_file("public/dir_test/file1.txt").unwrap();
    remove_file("public/dir_test/file2.txt").unwrap();
    remove_dir_all("public/dir_test").unwrap();
}
