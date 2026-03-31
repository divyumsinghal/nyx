//! Transaction helpers.

/// A PostgreSQL transaction borrowed from the pool.
pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

/// Begin a new transaction on `pool`.
///
/// # Errors
///
/// Returns [`Nun::NyxError`] if the transaction cannot be started.
pub async fn begin(pool: &sqlx::PgPool) -> Nun::Result<Transaction<'_>> {
    pool.begin().await.map_err(Nun::NyxError::internal)
}
