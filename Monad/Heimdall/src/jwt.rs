//! JWT encoding and decoding for Heimdall.
//!
//! Heimdall issues JWTs to authenticated callers and validates them on every
//! protected route. All tokens use HS256 with the shared `JWT_SECRET`.
//!
//! # Example
//!
//! ```rust,no_run
//! use heimdall::jwt::{encode_jwt, decode_jwt};
//!
//! let secret = "at-least-32-character-secret-key!!";
//! let token = encode_jwt("identity-id-here", secret, 3600).unwrap();
//! let claims = decode_jwt(&token, secret).unwrap();
//! assert_eq!(claims.sub, "identity-id-here");
//! ```

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Claims ───────────────────────────────────────────────────────────────────

/// JWT payload claims issued and validated by Heimdall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the Nyx identity ID (`UUIDv7` string).
    pub sub: String,
    /// Issued-at timestamp (Unix seconds).
    pub iat: i64,
    /// Expiry timestamp (Unix seconds).
    pub exp: i64,
    /// JWT ID — unique per token, prevents replay within its window.
    pub jti: String,
}

// ── JwtError ─────────────────────────────────────────────────────────────────

/// Errors returned by [`decode_jwt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtError {
    /// The token's `exp` claim is in the past.
    Expired,
    /// The token is malformed, has an invalid signature, or uses an unsupported algorithm.
    Invalid,
}

// ── encode_jwt ───────────────────────────────────────────────────────────────

/// Encode a new JWT for the given identity.
///
/// The token uses HS256. `expiry_secs` determines how long the token remains
/// valid relative to the current system time. A unique [`jti`](Claims::jti)
/// is generated for each call.
///
/// # Errors
///
/// Returns an error if the JWT library fails to serialize the claims (extremely
/// unlikely for well-formed inputs).
pub fn encode_jwt(identity_id: &str, secret: &str, expiry_secs: u64) -> anyhow::Result<String> {
    let now = chrono::Utc::now().timestamp();
    let expiry_secs = i64::try_from(expiry_secs)
        .map_err(|_| anyhow::anyhow!("expiry_secs exceeds supported range"))?;
    let claims = Claims {
        sub: identity_id.to_owned(),
        iat: now,
        exp: now + expiry_secs,
        jti: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

// ── decode_jwt ───────────────────────────────────────────────────────────────

/// Decode and validate a JWT string.
///
/// Validates signature (HS256), expiry, and required claim presence.
///
/// # Errors
///
/// - [`JwtError::Expired`] if the `exp` claim is in the past.
/// - [`JwtError::Invalid`] for any other failure (bad signature, malformed
///   token, unsupported algorithm, missing claims).
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);
    // Validate exp automatically; no audience/issuer configured.
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| map_jwt_error(&err))?;

    Ok(token_data.claims)
}

/// Maps a `jsonwebtoken::errors::Error` to our [`JwtError`] enum.
fn map_jwt_error(err: &jsonwebtoken::errors::Error) -> JwtError {
    use jsonwebtoken::errors::ErrorKind;
    match err.kind() {
        ErrorKind::ExpiredSignature => JwtError::Expired,
        _ => JwtError::Invalid,
    }
}
