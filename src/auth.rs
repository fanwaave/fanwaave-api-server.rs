#![forbid(unsafe_code)]

use crate::error::ApiError;

pub fn require_bearer(header: Option<&str>) -> Result<&str, ApiError> {
    header
        .and_then(|value| value.strip_prefix("Bearer "))
        // Bearer credentials are a single printable ASCII token. Treat
        // whitespace/control separators as authentication failures rather
        // than forwarding malformed credentials to downstream services.
        .filter(|token| !token.is_empty() && token.bytes().all(|b| b.is_ascii_graphic()))
        .ok_or(ApiError::Unauthenticated)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_printable_bearer_token() {
        assert_eq!(
            require_bearer(Some("Bearer abc.def-_~+/=")).unwrap(),
            "abc.def-_~+/="
        );
    }

    #[test]
    fn rejects_missing_blank_or_split_bearer_tokens() {
        for value in [
            None,
            Some(""),
            Some("Bearer"),
            Some("Bearer "),
            Some("Bearer  token"),
            Some("Bearer token other"),
            Some("Bearer token\tmore"),
            Some("Bearer token\n"),
            Some("bearer token"),
            Some("Basic token"),
        ] {
            assert!(require_bearer(value).is_err(), "unexpectedly accepted {value:?}");
        }
    }
}
