//! S3/MinIO bucket client.
use s3::creds::Credentials;
use s3::{Bucket, Region};
use tracing::instrument;

use Nun::config::StorageConfig;
use Nun::{NyxError, Result};

/// Thin wrapper around an S3 `Bucket` that maps errors to [`NyxError`].
pub struct StorageClient {
    pub(crate) bucket: Box<Bucket>,
}

/// Connect to the configured S3/MinIO bucket.
pub fn connect(config: &StorageConfig) -> Result<StorageClient> {
    let region = Region::Custom {
        region: config.region.clone(),
        endpoint: config.endpoint.clone(),
    };

    let credentials = Credentials::new(
        Some(config.access_key.expose()),
        Some(config.secret_key.expose()),
        None,
        None,
        None,
    )
    .map_err(|e| NyxError::internal(e.to_string()))?;

    let bucket =
        Bucket::new(&config.bucket, region, credentials).map_err(NyxError::internal)?;

    Ok(StorageClient { bucket })
}

impl StorageClient {
    /// Upload `data` to `path` with the given MIME content type.
    #[instrument(skip(self, data), fields(path, content_type))]
    pub async fn upload(&self, path: &str, data: Vec<u8>, content_type: &str) -> Result<()> {
        self.bucket
            .put_object_with_content_type(path, &data, content_type)
            .await
            .map_err(NyxError::internal)?;
        Ok(())
    }

    /// Delete the object at `path`.
    #[instrument(skip(self), fields(path))]
    pub async fn delete(&self, path: &str) -> Result<()> {
        self.bucket
            .delete_object(path)
            .await
            .map_err(NyxError::internal)?;
        Ok(())
    }

    /// Return `true` if an object exists at `path`.
    #[instrument(skip(self), fields(path))]
    pub async fn exists(&self, path: &str) -> Result<bool> {
        match self.bucket.head_object(path).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("404") || msg.contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(NyxError::internal(e))
                }
            }
        }
    }
}
