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
