//! JWT encoding and decoding for Heimdall.
//!
//! Heimdall issues JWTs to authenticated callers and validates them on every
//! protected route. Heimdall prefers RS256 when PEM keys are configured, and
//! falls back to HS256 only for legacy/dev compatibility.
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

use jsonwebtoken::{decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
/// The token uses RS256 when the RSA private key is configured, otherwise it
/// falls back to HS256 for legacy/dev compatibility.
///
/// # Errors
///
/// Returns an error if the JWT library fails to serialize the claims (extremely
/// unlikely for well-formed inputs).
pub fn encode_jwt(
    identity_id: &str,
    secret: &str,
    private_key_pem: Option<&str>,
    expiry_secs: u64,
) -> anyhow::Result<String> {
    let now = chrono::Utc::now().timestamp();
    let expiry_secs = i64::try_from(expiry_secs)
        .map_err(|_| anyhow::anyhow!("expiry_secs exceeds supported range"))?;
    let claims = Claims {
        sub: identity_id.to_owned(),
        iat: now,
        exp: now + expiry_secs,
        jti: Uuid::new_v4().to_string(),
    };

    let token = if let Some(private_key_pem) = private_key_pem {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("nyx-rs256-1".to_owned());
        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?,
        )?
    } else {
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?
    };

    Ok(token)
}

// ── decode_jwt ───────────────────────────────────────────────────────────────

/// Decode and validate a JWT string.
///
/// Validates signature, expiry, and required claim presence. RS256 is used
/// when the RSA public key is configured; HS256 is only accepted for legacy/dev
/// compatibility.
///
/// # Errors
///
/// - [`JwtError::Expired`] if the `exp` claim is in the past.
/// - [`JwtError::Invalid`] for any other failure (bad signature, malformed
///   token, unsupported algorithm, missing claims).
pub fn decode_jwt(token: &str, secret: &str, public_key_pem: Option<&str>) -> Result<Claims, JwtError> {
    let header = decode_header(token).map_err(|err| map_jwt_error(&err.into()))?;
    let alg = header.alg;

    let token_data = match (alg, public_key_pem) {
        (Algorithm::RS256, Some(public_key_pem)) => decode_with_validation(
            token,
            &DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).map_err(|_| JwtError::Invalid)?,
            Algorithm::RS256,
        )?,
        (Algorithm::HS256, _) => decode_with_validation(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            Algorithm::HS256,
        )?,
        _ => return Err(JwtError::Invalid),
    };

    Ok(token_data.claims)
}

fn decode_with_validation(
    token: &str,
    key: &DecodingKey,
    algorithm: Algorithm,
) -> Result<jsonwebtoken::TokenData<Claims>, JwtError> {
    let mut validation = Validation::new(algorithm);
    validation.validate_exp = true;

    decode::<Claims>(token, key, &validation).map_err(|err| map_jwt_error(&err))
}

/// Check whether a JWT ID has been revoked.
pub async fn is_jti_revoked(db: &PgPool, jti: &str) -> anyhow::Result<bool> {
    let revoked = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM nyx.revoked_jwts
            WHERE jti = $1
              AND expires_at > NOW()
        )
        "#,
    )
    .bind(jti)
    .fetch_one(db)
    .await?;

    Ok(revoked)
}

/// Revoke a JWT ID until its expiry.
pub async fn revoke_jti(db: &PgPool, jti: &str, subject: &str, expires_at: i64) -> anyhow::Result<()> {
    let expires_at = chrono::DateTime::from_timestamp(expires_at, 0)
        .ok_or_else(|| anyhow::anyhow!("invalid JWT expiry timestamp"))?;

    sqlx::query(
        r#"
        INSERT INTO nyx.revoked_jwts (jti, subject, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (jti) DO UPDATE
        SET revoked_at = NOW(),
            subject = EXCLUDED.subject,
            expires_at = EXCLUDED.expires_at
        "#,
    )
    .bind(jti)
    .bind(subject)
    .bind(expires_at)
    .execute(db)
    .await?;

    Ok(())
}

/// Maps a `jsonwebtoken::errors::Error` to our [`JwtError`] enum.
fn map_jwt_error(err: &jsonwebtoken::errors::Error) -> JwtError {
    use jsonwebtoken::errors::ErrorKind;
    match err.kind() {
        ErrorKind::ExpiredSignature => JwtError::Expired,
        _ => JwtError::Invalid,
    }
}
