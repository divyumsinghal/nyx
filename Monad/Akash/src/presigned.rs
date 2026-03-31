//! Presigned URL generation.
use Nun::Result;
use Nun::NyxError;

use crate::client::StorageClient;

impl StorageClient {
    /// Generate a presigned GET URL valid for the given number of seconds.
    pub fn presigned_get(&self, path: &str, expires_in: u32) -> Result<String> {
        self.bucket
            .presign_get(path, expires_in, None)
            .map_err(NyxError::internal)
    }

    /// Generate a presigned PUT URL for direct client uploads.
    pub fn presigned_put(&self, path: &str, expires_in: u32) -> Result<String> {
        self.bucket
            .presign_put(path, expires_in, None)
            .map_err(NyxError::internal)
    }
}
