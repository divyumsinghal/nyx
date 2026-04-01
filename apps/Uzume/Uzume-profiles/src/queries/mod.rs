//! Raw SQL query functions organised by table.
//!
//! Nothing in this module contains business logic. Each function maps 1-to-1
//! to a SQL statement and returns raw database rows. All error translation
//! from `sqlx::Error` to `NyxError` happens here via the `From` impl in Nun.

pub mod follow;
pub mod profiles;
