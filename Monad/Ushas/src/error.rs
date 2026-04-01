//! Error type for the Ushas notification crate.

/// All errors that can occur within the Ushas notification system.
#[derive(Debug, thiserror::Error)]
pub enum UshasError {
    /// The Gorush push gateway returned an error or an unexpected response.
    #[error("Gorush error: {0}")]
    GorushError(String),

    /// A PostgreSQL database operation failed.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// JSON serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
