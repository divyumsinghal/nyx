//! JWT creation and validation for Nyx access tokens.
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nun::{NyxApp, NyxError, Result};

/// Claims embedded in Nyx JWT access tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the Kratos identity ID.
    pub sub: Uuid,
    /// Expiry (Unix timestamp).
    pub exp: i64,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Which app this token is scoped to.
    pub app: NyxApp,
}

/// Validate a JWT and return its claims.
///
/// `secret` is the HMAC-SHA256 signing key (raw bytes).
pub fn validate_jwt(token: &str, secret: &[u8]) -> Result<Claims> {
    let key = DecodingKey::from_secret(secret);
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| NyxError::unauthorized("invalid_token", e.to_string()))
}

/// Create a new JWT for a given identity and app, valid for 24 hours.
///
/// `secret` is the HMAC-SHA256 signing key (raw bytes).
pub fn create_jwt(identity_id: Uuid, app: NyxApp, secret: &[u8]) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: identity_id,
        exp: (now + Duration::hours(24)).timestamp(),
        iat: now.timestamp(),
        app,
    };
    let key = EncodingKey::from_secret(secret);
    encode(&Header::default(), &claims, &key).map_err(NyxError::internal)
}
