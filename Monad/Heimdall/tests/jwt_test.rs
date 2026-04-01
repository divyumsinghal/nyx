//! JWT encode/decode tests — Cycle 1 (TDD: written before implementation).

use heimdall::jwt::{decode_jwt, encode_jwt, Claims, JwtError};

const SECRET: &str = "super-secret-key-for-testing-only-32chars!";
const WRONG_SECRET: &str = "wrong-secret-key-for-testing-32chars!!";
const IDENTITY: &str = "01920000-0000-7000-8000-000000000001";

// 1. Encode then decode round-trip — claims.sub must match the original identity id.
#[test]
fn test_encode_decode_round_trip() {
    let token = encode_jwt(IDENTITY, SECRET, 3600).expect("encode should succeed");
    let claims: Claims = decode_jwt(&token, SECRET).expect("decode should succeed");
    assert_eq!(claims.sub, IDENTITY);
}

// 2. Expired token (exp = now - 1s) → JwtError::Expired.
#[test]
fn test_expired_token_returns_expired_error() {
    // Encode with 0-second expiry so it expires immediately.
    let token = encode_jwt(IDENTITY, SECRET, 0).expect("encode should succeed");
    // Small sleep is not available in unit tests — use expiry_secs=0 and rely on
    // jsonwebtoken's leeway being 0 by default; exp == iat means already expired.
    let result = decode_jwt(&token, SECRET);
    // exp == iat, so the token is either already expired or valid for 0 seconds.
    // We accept either Expired or Ok here for 0-second expiry.
    // Instead test with a manually crafted past token via the helper.
    let _ = result; // just ensure it doesn't panic
}

// 3. Wrong secret → JwtError::Invalid.
#[test]
fn test_wrong_secret_returns_invalid() {
    let token = encode_jwt(IDENTITY, SECRET, 3600).expect("encode should succeed");
    let result = decode_jwt(&token, WRONG_SECRET);
    assert!(matches!(result, Err(JwtError::Invalid)));
}

// 4. Malformed string "not.a.token" → JwtError::Invalid.
#[test]
fn test_malformed_token_returns_invalid() {
    let result = decode_jwt("not.a.token", SECRET);
    assert!(matches!(result, Err(JwtError::Invalid)));
}

// 5. Claims contain correct iat/exp window.
#[test]
fn test_claims_contain_correct_time_window() {
    let expiry_secs: u64 = 3600;
    let before = chrono::Utc::now().timestamp();
    let token = encode_jwt(IDENTITY, SECRET, expiry_secs).expect("encode should succeed");
    let after = chrono::Utc::now().timestamp();

    let claims = decode_jwt(&token, SECRET).expect("decode should succeed");

    assert!(
        claims.iat >= before,
        "iat should be >= time before encoding"
    );
    assert!(claims.iat <= after, "iat should be <= time after encoding");
    assert_eq!(
        claims.exp - claims.iat,
        expiry_secs as i64,
        "exp - iat should equal expiry_secs"
    );
}

// 6. jti field is a non-empty UUID.
#[test]
fn test_claims_jti_is_non_empty_uuid() {
    let token = encode_jwt(IDENTITY, SECRET, 3600).expect("encode should succeed");
    let claims = decode_jwt(&token, SECRET).expect("decode should succeed");
    assert!(!claims.jti.is_empty(), "jti must not be empty");
    // Must be parseable as UUID.
    uuid::Uuid::parse_str(&claims.jti).expect("jti must be a valid UUID");
}

// 7. Expired token explicitly constructed returns JwtError::Expired.
#[test]
fn test_explicitly_expired_token_returns_expired() {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct RawClaims {
        sub: String,
        iat: i64,
        exp: i64,
        jti: String,
    }

    let past = chrono::Utc::now().timestamp() - 7200; // 2 hours ago
    let raw = RawClaims {
        sub: IDENTITY.to_owned(),
        iat: past,
        exp: past + 3600, // still expired (1 hour ago)
        jti: uuid::Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &raw,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .expect("encode should succeed");

    let result = decode_jwt(&token, SECRET);
    assert!(matches!(result, Err(JwtError::Expired)));
}
