//! `X-Auth-Mode` request-header detection. Native (mobile) clients opt into the
//! bearer delivery path by sending `X-Auth-Mode: bearer`; everything else is the
//! default cookie flow.

use axum::http::HeaderMap;

/// Request header that opts a client into bearer-token delivery.
pub const AUTH_MODE_HEADER: &str = "x-auth-mode";

const AUTH_MODE_BEARER: &str = "bearer";

/// True when the request asks for bearer-mode delivery (`X-Auth-Mode: bearer`,
/// case-insensitive). Absent or any other value → false (cookie mode).
pub fn wants_bearer(headers: &HeaderMap) -> bool {
    headers
        .get(AUTH_MODE_HEADER)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case(AUTH_MODE_BEARER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers_with(mode: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTH_MODE_HEADER, HeaderValue::from_str(mode).unwrap());
        h
    }

    #[test]
    fn bearer_value_detected() {
        assert!(wants_bearer(&headers_with("bearer")));
    }

    #[test]
    fn bearer_value_is_case_insensitive() {
        assert!(wants_bearer(&headers_with("Bearer")));
    }

    #[test]
    fn missing_header_is_cookie_mode() {
        assert!(!wants_bearer(&HeaderMap::new()));
    }

    #[test]
    fn other_value_is_cookie_mode() {
        assert!(!wants_bearer(&headers_with("cookie")));
    }
}
