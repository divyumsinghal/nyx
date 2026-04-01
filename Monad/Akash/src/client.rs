use std::{collections::BTreeMap, time::Duration};

use nun::{NyxError, Result, config::StorageConfig};
use s3::{Bucket, Region, creds::Credentials};

use crate::{ObjectKey, PresignedDownload, PresignedUpload, UploadMetadata};

#[derive(Clone)]
pub struct StorageClient {
    bucket: Box<Bucket>,
}

pub fn connect(config: &StorageConfig) -> Result<StorageClient> {
    StorageClient::new(config)
}

impl std::fmt::Debug for StorageClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageClient").finish_non_exhaustive()
    }
}

impl StorageClient {
    pub fn new(config: &StorageConfig) -> Result<Self> {
        let bucket_name = config.bucket.trim();
        if bucket_name.is_empty() {
            return Err(NyxError::bad_request(
                "storage_bucket_required",
                "Storage bucket must not be empty",
            ));
        }

        let endpoint = config.endpoint.trim().trim_end_matches('/').to_string();
        if endpoint.is_empty() {
            return Err(NyxError::bad_request(
                "storage_endpoint_required",
                "Storage endpoint must not be empty",
            ));
        }

        let region_name = config.region.trim();
        if region_name.is_empty() {
            return Err(NyxError::bad_request(
                "storage_region_required",
                "Storage region must not be empty",
            ));
        }

        let region = Region::Custom {
            region: region_name.to_string(),
            endpoint,
        };

        let credentials = Credentials::new(
            Some(config.access_key.expose()),
            Some(config.secret_key.expose()),
            None,
            None,
            None,
        )
        .map_err(|err| {
            NyxError::service_unavailable(
                "storage_credentials_invalid",
                format!("Failed to create storage credentials: {err}"),
            )
        })?;

        let bucket = Bucket::new(bucket_name, region, credentials)
            .map_err(|err| {
                NyxError::service_unavailable(
                    "storage_bucket_init_failed",
                    format!("Failed to initialize storage bucket: {err}"),
                )
            })?
            .with_path_style();

        Ok(Self { bucket })
    }

    pub async fn presign_put(
        &self,
        object_key: &ObjectKey,
        expires_in: Duration,
        metadata: UploadMetadata,
    ) -> Result<PresignedUpload> {
        let ttl = duration_to_expiry(expires_in)?;

        let url = self
            .bucket
            .presign_put(object_key.as_str(), ttl, None, None)
            .await
            .map_err(map_storage_error)?;

        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_string(), metadata.content_type().to_string());
        headers.insert(
            "content-length".to_string(),
            metadata.content_length().to_string(),
        );
        headers.insert("x-content-type-options".to_string(), "nosniff".to_string());
        headers.insert("cache-control".to_string(), "no-store".to_string());

        Ok(PresignedUpload::new(url, "PUT", headers, metadata))
    }

    pub async fn presign_get(
        &self,
        object_key: &ObjectKey,
        expires_in: Duration,
    ) -> Result<PresignedDownload> {
        let ttl = duration_to_expiry(expires_in)?;
        let url = self
            .bucket
            .presign_get(object_key.as_str(), ttl, None)
            .await
            .map_err(map_storage_error)?;

        let mut headers = BTreeMap::new();
        headers.insert("x-content-type-options".to_string(), "nosniff".to_string());
        headers.insert("cache-control".to_string(), "private, max-age=60".to_string());

        Ok(PresignedDownload::new(url, "GET", headers))
    }

    pub async fn upload(
        &self,
        object_key: &ObjectKey,
        data: &[u8],
        content_type: &str,
    ) -> Result<()> {
        self.bucket
            .put_object_with_content_type(object_key.as_str(), data, content_type)
            .await
            .map_err(map_storage_error)?;
        Ok(())
    }

    pub async fn delete(&self, object_key: &ObjectKey) -> Result<()> {
        self.bucket
            .delete_object(object_key.as_str())
            .await
            .map_err(map_storage_error)?;
        Ok(())
    }

    pub async fn exists(&self, object_key: &ObjectKey) -> Result<bool> {
        match self.bucket.head_object(object_key.as_str()).await {
            Ok(_) => Ok(true),
            Err(err) => {
                let message = err.to_string();
                if message.contains("404") || message.contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(map_storage_error(err))
                }
            }
        }
    }
}

fn duration_to_expiry(expires_in: Duration) -> Result<u32> {
    let secs = expires_in.as_secs();
    if secs == 0 {
        return Err(NyxError::bad_request(
            "presign_expiry_invalid",
            "Presigned URL expiry must be at least 1 second",
        ));
    }

    u32::try_from(secs).map_err(|_| {
        NyxError::bad_request(
            "presign_expiry_invalid",
            "Presigned URL expiry exceeds supported range",
        )
    })
}

fn map_storage_error(err: s3::error::S3Error) -> NyxError {
    NyxError::service_unavailable(
        "storage_presign_failed",
        format!("Failed to generate presigned storage URL: {err}"),
    )
}
