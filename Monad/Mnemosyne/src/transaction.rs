use std::{future::Future, pin::Pin};

use nun::Result;
use sqlx::{PgPool, Postgres, Transaction};

pub async fn with_transaction<T, F>(
    pool: &PgPool,
    f: F,
) -> Result<T>
where
    F: for<'a> FnOnce(
        &'a mut Transaction<'_, Postgres>,
    ) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>,
{
    let mut tx = pool.begin().await.map_err(nun::NyxError::from)?;
    let result = f(&mut tx).await;

    match result {
        Ok(value) => {
            tx.commit().await.map_err(nun::NyxError::from)?;
            Ok(value)
        }
        Err(err) => {
            tx.rollback().await.map_err(nun::NyxError::from)?;
            Err(err)
        }
    }
}
