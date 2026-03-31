//! API envelope primitives.

use serde::Serialize;

/// Standard API response wrapper.
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_wraps_data() {
        let response = ApiResponse::new(42_u8);
        let json = serde_json::to_string(&response).expect("json serialization should work");
        assert_eq!(json, "{\"data\":42}");
    }
}
