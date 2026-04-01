#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]

pub mod auth_layer;
pub mod config;
pub mod health;
pub mod jwt;
pub mod proxy;
pub mod routes;
pub mod state;
pub mod websocket;
