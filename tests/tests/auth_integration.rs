//! Auth integration tests — full end-to-end flows against a real Kratos instance.
//!
//! # Prerequisites
//!
//! The auth stack must be running before these tests execute:
//!
//! ```bash
//! docker compose -f Prithvi/compose/auth-test.yml up -d --wait
//! ```
//!
//! Then run the tests:
//!
//! ```bash
//! KRATOS_PUBLIC_URL=http://localhost:4433 \
//! KRATOS_ADMIN_URL=http://localhost:4434  \
//! MAILPIT_API_URL=http://localhost:8025   \
//! cargo test --test auth_integration -- --test-threads=4
//! ```
//!
//! Or use the convenience script:
//! ```bash
//! ./tools/scripts/test-auth.sh
//! ```
//!
//! Tests will be **skipped** (not failed) if the stack is not running.
//! This allows `cargo test` to work normally without the stack.

#[path = "auth/helpers.rs"]
mod helpers;

#[path = "auth/email_password.rs"]
mod email_password;

#[path = "auth/email_otp.rs"]
mod email_otp;

#[path = "auth/nyx_id_tests.rs"]
mod nyx_id_tests;

#[path = "auth/session_tests.rs"]
mod session_tests;

#[path = "auth/recovery.rs"]
mod recovery;
