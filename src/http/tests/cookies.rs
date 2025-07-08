use super::*;
use chrono::{Utc, Duration};

#[test]
fn default_cookie_serialization_contains_expected_fields() {
    let cookie = Cookie::new("session".to_string(), "abc123".to_string());
    let serialized = cookie.serialize();

    assert!(serialized.contains("session=abc123"));
    assert!(serialized.contains("Path=/"));
    assert!(serialized.contains("Expires="));
    assert!(serialized.contains("HttpOnly;"));
    assert!(serialized.contains("Secure;"));
}

#[test]
fn cookie_serialization_skips_flags_when_false() {
    let cookie = Cookie {
        name: "id".to_string(),
        value: "xyz789".to_string(),
        path: "/test".to_string(),
        expires: Utc::now() + Duration::days(1),
        http_only: false,
        secure: false,
    };
    let serialized = cookie.serialize();

    assert!(serialized.contains("id=xyz789"));
    assert!(serialized.contains("Path=/test"));
    assert!(serialized.contains("Expires="));
    assert!(!serialized.contains("HttpOnly;"));
    assert!(!serialized.contains("Secure;"));
}
