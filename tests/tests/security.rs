#[path = "security/feed_security.rs"]
mod feed_security;

#[path = "security/payload_sweep.rs"]
mod payload_sweep;

#[path = "security/sql_injection.rs"]
mod sql_injection;

#[path = "security/xss.rs"]
mod xss;

#[path = "security/authz_bypass.rs"]
mod authz_bypass;

#[path = "security/privacy_violations.rs"]
mod privacy_violations;

#[path = "security/edge_cases.rs"]
mod edge_cases;
