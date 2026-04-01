//! Custom assertion helpers for Nyx tests.

use serde::de::DeserializeOwned;

/// Assert that an HTTP response has the expected status code.
#[macro_export]
macro_rules! assert_status {
    ($response:expr, $expected:expr) => {
        assert_eq!(
            $response.status(),
            $expected,
            "expected status {}, got {}",
            $expected,
            $response.status()
        );
    };
}

/// Assert that an HTTP response is successful (2xx).
#[macro_export]
macro_rules! assert_success {
    ($response:expr) => {
        assert!(
            $response.status().is_success(),
            "expected success status, got {}",
            $response.status()
        );
    };
}

/// Assert that an HTTP response is a client error (4xx).
#[macro_export]
macro_rules! assert_client_error {
    ($response:expr) => {
        assert!(
            $response.status().is_client_error(),
            "expected client error status, got {}",
            $response.status()
        );
    };
}

/// Assert that an HTTP response is a server error (5xx).
#[macro_export]
macro_rules! assert_server_error {
    ($response:expr) => {
        assert!(
            $response.status().is_server_error(),
            "expected server error status, got {}",
            $response.status()
        );
    };
}

/// Assert that a JSON response matches expected structure.
#[macro_export]
macro_rules! assert_json_eq {
    ($actual:expr, $expected:expr) => {
        assert_eq!(
            serde_json::to_value($actual).unwrap(),
            serde_json::to_value($expected).unwrap(),
            "JSON mismatch"
        );
    };
}

/// Helper to parse JSON response body.
pub async fn parse_json_response<T: DeserializeOwned>(
    response: axum::response::Response,
) -> anyhow::Result<T> {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let value: T = serde_json::from_slice(&body_bytes)?;
    Ok(value)
}

/// Assert that a response contains JSON with specific field value.
#[macro_export]
macro_rules! assert_json_contains {
    ($json:expr, $key:expr, $value:expr) => {
        let json_value = serde_json::to_value($json).unwrap();
        let actual = json_value
            .get($key)
            .unwrap_or_else(|| panic!("key {} not found in JSON", $key));
        let expected = serde_json::to_value($value).unwrap();
        assert_eq!(
            *actual, expected,
            "expected {}={}, got {}",
            $key, expected, actual
        );
    };
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn json_eq_macro_works() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"name": "Alice", "age": 30});
        assert_json_eq!(a, b);
    }

    #[test]
    fn json_contains_macro_works() {
        let data = json!({"user": {"name": "Bob"}, "count": 5});
        assert_json_contains!(data, "count", 5);
    }
}
