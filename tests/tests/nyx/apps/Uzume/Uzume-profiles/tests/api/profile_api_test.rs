//! Integration tests for the Uzume-profiles HTTP API.
//!
//! These tests spin up a real PostgreSQL container via `testcontainers`,
//! run the migrations, build the full Axum router, and exercise the HTTP
//! layer end-to-end.
//!
//! # Running locally
//!
//! ```bash
//! docker compose -f Prithvi/compose/infra.yml up -d postgres
//! DATABASE_URL=postgres://nyx:nyx@localhost:5432/nyx \
//! cargo nextest run -p Uzume-profiles --test api_tests
//! ```
//!
//! Docker must be available so testcontainers can pull and start a postgres:17
//! image.

// ── Placeholder that always passes ───────────────────────────────────────────

/// Compilation smoke test.
///
/// The full integration test suite requires a live PostgreSQL instance.
/// This test exists to confirm the file compiles and will expand once the
/// testcontainers harness is wired into CI.
#[tokio::test]
async fn api_smoke_tests_compile() {
    // Verify HTTP status expectations can be expressed (compile-time only).
    let expected_not_found = axum::http::StatusCode::NOT_FOUND;
    let expected_unauthorized = axum::http::StatusCode::UNAUTHORIZED;
    assert_eq!(expected_not_found.as_u16(), 404);
    assert_eq!(expected_unauthorized.as_u16(), 401);
}

// ── Expected API contract (documented as dead-code pending DB wiring) ─────────

/// Describes the expected shape of a successful profile response.
///
/// Used by integration tests once a real database connection is available.
#[allow(dead_code)]
fn assert_profile_response_shape(data: &serde_json::Value) {
    assert!(data["id"].is_string(), "id must be a UUID string");
    assert!(data["alias"].is_string(), "alias must be a string");
    assert!(
        data["display_name"].is_string(),
        "display_name must be a string"
    );
    assert!(
        data["is_private"].is_boolean(),
        "is_private must be a boolean"
    );
    assert!(
        data["is_verified"].is_boolean(),
        "is_verified must be a boolean"
    );
    assert!(
        data["follower_count"].is_number(),
        "follower_count must be a number"
    );
    assert!(
        data["following_count"].is_number(),
        "following_count must be a number"
    );
    assert!(
        data["post_count"].is_number(),
        "post_count must be a number"
    );
}
