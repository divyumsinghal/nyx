//! # events — Typed Nyx event contracts and lifecycle subjects
//!
//! NATS JetStream typed pub/sub for the Nyx platform.
//!
//! ## Core types
//!
//! - [`NyxEvent<T>`](envelope::NyxEvent) — Event envelope with id, subject, app, timestamp, payload
//! - [`NatsClient`](client::NatsClient) — Connection and stream management
//! - [`Publisher`](publisher::Publisher) — Typed event publisher
//! - [`Subscriber`](subscriber::Subscriber) — Typed event subscriber
//!
//! ## Subject convention
//!
//! `{app_or_nyx}.{entity}.{action}` — see [`subjects`] for all constants.

pub mod client;
pub mod compat;
pub mod envelope;
pub mod publisher;
pub mod subjects;
pub mod subscriber;

pub use client::{NatsClient, NatsError};
pub use compat::{EventEnvelope, EventPublisher, NoopEventPublisher};
pub use envelope::NyxEvent;
pub use publisher::Publisher;
pub use subscriber::Subscriber;
