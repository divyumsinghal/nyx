//! # nyx-events — NATS JetStream typed pub/sub
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod envelope;
pub mod publisher;
pub mod subjects;
pub mod subscriber;

pub use client::{connect, NatsClient};
pub use envelope::NyxEvent;
pub use publisher::Publisher;
pub use subscriber::Subscriber;
