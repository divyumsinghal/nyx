//! # Ushas — Push + in-app notification dispatch for the Nyx platform
//!
//! Gorush HTTP client for APNs/FCM. In-app notification storage.
//! Notification grouping ("X and 42 others liked your post").
//! User preference management per app/event-type.
//!
//! ## Planned modules
//! - `gorush` — Gorush HTTP client
//! - `in_app` — PostgreSQL storage + WebSocket push
//! - `grouping` — notification aggregation logic
//! - `preferences` — per-app, per-event-type mute settings
//! - `bin/worker` — NATS subscriber → dispatch

// TODO: implement Ushas modules
