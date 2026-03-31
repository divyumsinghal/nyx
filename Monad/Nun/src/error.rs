//! Error handling for the Nyx platform.
//!
//! [`NyxError`] is the single error type returned by all fallible platform
//! functions. It is a struct (not an enum) containing:
//!
//! - An [`ErrorKind`] that maps to an HTTP status code.
//! - A machine-readable `code` string (e.g., `"post_not_found"`).
//! - A human-readable `message`.
//! - An optional source error (for logging / error chains).
//! - Optional [`ErrorMetadata`] (validation details, rate limit info).
//!
//! # Usage
//!
//! ```rust
//! use nun::{NyxError, Result};
//!
//! fn find_post(id: &str) -> Result<()> {
//!     Err(NyxError::not_found("post_not_found", "Post does not exist"))
//! }
//!
//! fn validate_input(email: &str) -> Result<()> {
//!     Err(NyxError::validation(vec![
//!         FieldError::new("email", "invalid_format", "Not a valid email address"),
//!     ]))
//! }
//!
//! fn db_call() -> Result<()> {
//!     let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::other("boom"));
//!     result.map_err(NyxError::internal)?;
//!     Ok(())
//! }
//! ```

use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use serde::Serialize;

/// Platform-wide result type. Every fallible function returns `Result<T>`.
pub type Result<T> = std::result::Result<T, NyxError>;

// ── NyxError ────────────────────────────────────────────────────────────────

/// The platform error type.
///
/// Constructed via named constructors ([`NyxError::not_found`],
/// [`NyxError::internal`], etc.) rather than enum variants.
/// This keeps the API stable — new error kinds can be added to [`ErrorKind`]
/// without breaking callers.
pub struct NyxError {
    kind: ErrorKind,
    /// Machine-readable error code, e.g. `"post_not_found"`, `"rate_limited"`.
    code: Cow<'static, str>,
    /// Human-readable error message.
    message: Cow<'static, str>,
    /// The underlying error, if any. Logged server-side, never sent to clients.
    source: Option<Box<dyn Error + Send + Sync>>,
    /// Optional structured metadata (validation errors, rate limit info).
    metadata: Option<ErrorMetadata>,
}

impl NyxError {
    // ── Constructors ────────────────────────────────────────────────────

    /// 400 Bad Request — malformed input that isn't a validation error.
    pub fn bad_request(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// 401 Unauthorized — missing or invalid authentication.
    pub fn unauthorized(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::Unauthorized,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// 403 Forbidden — authenticated but lacking permission.
    pub fn forbidden(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::Forbidden,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// 404 Not Found — the requested entity does not exist.
    pub fn not_found(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::NotFound,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// 409 Conflict — uniqueness violation or state conflict.
    pub fn conflict(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::Conflict,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// 413 Payload Too Large.
    pub fn payload_too_large(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::PayloadTooLarge,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// 422 Unprocessable Entity — structured validation failure with per-field errors.
    pub fn validation(fields: Vec<FieldError>) -> Self {
        Self {
            kind: ErrorKind::UnprocessableEntity,
            code: Cow::Borrowed("validation_failed"),
            message: Cow::Borrowed("One or more fields failed validation"),
            source: None,
            metadata: Some(ErrorMetadata::Validation(fields)),
        }
    }

    /// 429 Rate Limited.
    pub fn rate_limited(retry_after_secs: u32) -> Self {
        Self {
            kind: ErrorKind::RateLimited,
            code: Cow::Borrowed("rate_limited"),
            message: Cow::Borrowed("Too many requests"),
            source: None,
            metadata: Some(ErrorMetadata::RateLimit { retry_after_secs }),
        }
    }

    /// 500 Internal Server Error — wraps an underlying error.
    ///
    /// The source error is captured for server-side logging but **never**
    /// exposed to clients. The client sees a generic "internal error" message.
    pub fn internal(source: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            code: Cow::Borrowed("internal_error"),
            message: Cow::Borrowed("An internal error occurred"),
            source: Some(source.into()),
            metadata: None,
        }
    }

    /// 503 Service Unavailable — a downstream dependency is unreachable.
    pub fn service_unavailable(
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::ServiceUnavailable,
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    /// Custom error with an arbitrary HTTP status code.
    ///
    /// Escape hatch for status codes not covered by the named constructors.
    pub fn custom(
        status: u16,
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            kind: ErrorKind::Custom(status),
            code: code.into(),
            message: message.into(),
            source: None,
            metadata: None,
        }
    }

    // ── Builder methods (chainable) ─────────────────────────────────────

    /// Attach a source error for logging.
    pub fn with_source(mut self, source: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        self.source = Some(source.into());
        self
    }

    // ── Accessors ───────────────────────────────────────────────────────

    /// The error kind (maps to an HTTP status code).
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The HTTP status code as a `u16`.
    pub fn status_code(&self) -> u16 {
        self.kind.status_code()
    }

    /// The machine-readable error code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The human-readable error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The structured metadata, if any.
    pub fn metadata(&self) -> Option<&ErrorMetadata> {
        self.metadata.as_ref()
    }

    /// Build the JSON-serializable error response for the wire.
    ///
    /// `request_id` is injected by the API middleware layer (nyx-api), not by Nun.
    pub fn to_error_response(&self, request_id: Option<String>) -> ErrorResponse {
        let (fields, retry_after) = match &self.metadata {
            Some(ErrorMetadata::Validation(field_errors)) => {
                let fields: Vec<FieldErrorResponse> = field_errors
                    .iter()
                    .map(|f| FieldErrorResponse {
                        field: f.field.to_string(),
                        message: f.message.to_string(),
                        code: f.code.to_string(),
                    })
                    .collect();
                (Some(fields), None)
            }
            Some(ErrorMetadata::RateLimit { retry_after_secs }) => {
                (None, Some(*retry_after_secs))
            }
            None => (None, None),
        };

        ErrorResponse {
            error: self.message.to_string(),
            code: self.code.to_string(),
            request_id,
            fields,
            retry_after,
        }
    }
}

// ── std::error::Error ───────────────────────────────────────────────────────

impl Error for NyxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

impl fmt::Display for NyxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.kind.as_str(), self.code, self.message)
    }
}

impl fmt::Debug for NyxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("NyxError");
        d.field("kind", &self.kind)
            .field("code", &self.code.as_ref())
            .field("message", &self.message.as_ref());
        if let Some(ref source) = self.source {
            d.field("source", source);
        }
        if let Some(ref metadata) = self.metadata {
            d.field("metadata", metadata);
        }
        d.finish()
    }
}

// ── From impls for common error types ───────────────────────────────────────

impl From<serde_json::Error> for NyxError {
    fn from(err: serde_json::Error) -> Self {
        Self::bad_request("invalid_json", format!("Invalid JSON: {err}"))
    }
}

impl From<uuid::Error> for NyxError {
    fn from(err: uuid::Error) -> Self {
        Self::bad_request("invalid_id", format!("Invalid ID format: {err}"))
    }
}

impl From<config::ConfigError> for NyxError {
    fn from(err: config::ConfigError) -> Self {
        Self::internal(err)
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for NyxError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => {
                Self::not_found("record_not_found", "Record not found")
            }
            sqlx::Error::Database(db_err) => {
                // PostgreSQL error codes:
                // 23505 = unique_violation (duplicate key)
                // 23503 = foreign_key_violation
                // 23502 = not_null_violation
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => {
                            return Self::conflict(
                                "duplicate_record",
                                "A record with this value already exists",
                            );
                        }
                        "23503" => {
                            return Self::bad_request(
                                "foreign_key_violation",
                                "Referenced record does not exist",
                            );
                        }
                        _ => {}
                    }
                }
                Self::internal(err)
            }
            _ => Self::internal(err),
        }
    }
}

#[cfg(feature = "validator")]
impl From<validator::ValidationErrors> for NyxError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let fields: Vec<FieldError> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| {
                    FieldError::new(
                        *field,
                        e.code.as_ref(),
                        e.message
                            .as_ref()
                            .map_or_else(|| e.code.to_string(), ToString::to_string),
                    )
                })
            })
            .collect();
        Self::validation(fields)
    }
}

// ── ErrorKind ───────────────────────────────────────────────────────────────

/// Categorizes an error by its HTTP status code.
///
/// `#[non_exhaustive]` — new variants can be added without breaking downstream
/// `match` arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// 400
    BadRequest,
    /// 401
    Unauthorized,
    /// 403
    Forbidden,
    /// 404
    NotFound,
    /// 409
    Conflict,
    /// 413
    PayloadTooLarge,
    /// 422
    UnprocessableEntity,
    /// 429
    RateLimited,
    /// 500
    Internal,
    /// 503
    ServiceUnavailable,
    /// Arbitrary status code for edge cases.
    Custom(u16),
}

impl ErrorKind {
    /// Map to the corresponding HTTP status code.
    pub fn status_code(self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::PayloadTooLarge => 413,
            Self::UnprocessableEntity => 422,
            Self::RateLimited => 429,
            Self::Internal => 500,
            Self::ServiceUnavailable => 503,
            Self::Custom(code) => code,
        }
    }

    /// Short string label for logging.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::PayloadTooLarge => "payload_too_large",
            Self::UnprocessableEntity => "unprocessable_entity",
            Self::RateLimited => "rate_limited",
            Self::Internal => "internal",
            Self::ServiceUnavailable => "service_unavailable",
            Self::Custom(_) => "custom",
        }
    }
}

// ── Metadata ────────────────────────────────────────────────────────────────

/// Structured metadata attached to specific error kinds.
#[derive(Debug)]
pub enum ErrorMetadata {
    /// Per-field validation errors (for 422 responses).
    Validation(Vec<FieldError>),
    /// Rate limit information (for 429 responses).
    RateLimit {
        /// Seconds until the client can retry.
        retry_after_secs: u32,
    },
}

/// A single field validation error.
#[derive(Debug, Clone)]
pub struct FieldError {
    /// The field name that failed validation (e.g., `"email"`, `"bio"`).
    pub field: Cow<'static, str>,
    /// Machine-readable code (e.g., `"too_short"`, `"invalid_format"`).
    pub code: Cow<'static, str>,
    /// Human-readable message.
    pub message: Cow<'static, str>,
}

impl FieldError {
    pub fn new(
        field: impl Into<Cow<'static, str>>,
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

// ── Wire format ─────────────────────────────────────────────────────────────

/// The JSON body returned to clients on error.
///
/// All Nyx services return this exact structure for all errors. Clients can
/// rely on the `code` field for programmatic error handling and `error` for
/// display.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Human-readable error message.
    pub error: String,
    /// Machine-readable error code.
    pub code: String,
    /// Request ID for correlation (injected by middleware).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Per-field validation errors (only for 422).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldErrorResponse>>,
    /// Seconds until the client can retry (only for 429).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u32>,
}

/// A single field error in the wire format.
#[derive(Debug, Serialize)]
pub struct FieldErrorResponse {
    pub field: String,
    pub code: String,
    pub message: String,
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_has_correct_status() {
        let err = NyxError::not_found("post_not_found", "Post does not exist");
        assert_eq!(err.status_code(), 404);
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.code(), "post_not_found");
        assert_eq!(err.message(), "Post does not exist");
    }

    #[test]
    fn internal_hides_source_in_message() {
        let source = std::io::Error::other("db connection failed");
        let err = NyxError::internal(source);
        assert_eq!(err.status_code(), 500);
        assert_eq!(err.message(), "An internal error occurred");
        // Source is captured for logging
        assert!(err.source().is_some());
    }

    #[test]
    fn validation_carries_field_errors() {
        let err = NyxError::validation(vec![
            FieldError::new("email", "invalid_format", "Not a valid email"),
            FieldError::new("name", "too_short", "Must be at least 2 characters"),
        ]);
        assert_eq!(err.status_code(), 422);
        match err.metadata() {
            Some(ErrorMetadata::Validation(fields)) => assert_eq!(fields.len(), 2),
            _ => panic!("expected validation metadata"),
        }
    }

    #[test]
    fn rate_limited_carries_retry_after() {
        let err = NyxError::rate_limited(60);
        assert_eq!(err.status_code(), 429);
        match err.metadata() {
            Some(ErrorMetadata::RateLimit { retry_after_secs }) => {
                assert_eq!(*retry_after_secs, 60);
            }
            _ => panic!("expected rate limit metadata"),
        }
    }

    #[test]
    fn error_response_serializes_correctly() {
        let err = NyxError::not_found("user_not_found", "User not found");
        let resp = err.to_error_response(Some("req-123".to_string()));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["code"], "user_not_found");
        assert_eq!(json["error"], "User not found");
        assert_eq!(json["request_id"], "req-123");
        assert!(json.get("fields").is_none()); // skipped when None
        assert!(json.get("retry_after").is_none());
    }

    #[test]
    fn validation_response_includes_fields() {
        let err = NyxError::validation(vec![
            FieldError::new("email", "required", "Email is required"),
        ]);
        let resp = err.to_error_response(None);
        let json = serde_json::to_value(&resp).unwrap();
        let fields = json["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0]["field"], "email");
    }

    #[test]
    fn display_format() {
        let err = NyxError::not_found("post_not_found", "Post does not exist");
        let s = err.to_string();
        assert!(s.contains("not_found"));
        assert!(s.contains("post_not_found"));
        assert!(s.contains("Post does not exist"));
    }

    #[test]
    fn custom_status_code() {
        let err = NyxError::custom(418, "teapot", "I'm a teapot");
        assert_eq!(err.status_code(), 418);
        assert_eq!(err.kind(), ErrorKind::Custom(418));
    }

    #[test]
    fn with_source_chaining() {
        let source = std::io::Error::other("connection refused");
        let err = NyxError::service_unavailable("db_unreachable", "Database is unreachable")
            .with_source(source);
        assert_eq!(err.status_code(), 503);
        assert!(err.source().is_some());
    }

    #[test]
    fn from_serde_json_error() {
        let err: NyxError = serde_json::from_str::<serde_json::Value>("not json")
            .unwrap_err()
            .into();
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.code(), "invalid_json");
    }

    #[test]
    fn from_uuid_error() {
        let err: NyxError = uuid::Uuid::parse_str("not-a-uuid").unwrap_err().into();
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.code(), "invalid_id");
    }
}
