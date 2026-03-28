use serde::{Deserialize, Serialize};

/// S3 storage service for Sora media
pub struct SoraS3Storage {
    config: S3Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("Upload failed: {0}")]
    UploadFailed(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
}

impl SoraS3Storage {
    pub fn new(config: S3Config) -> Self {
        Self { config }
    }

    /// Upload file to S3
    pub async fn upload(&self, key: &str, _data: &[u8]) -> Result<String, S3Error> {
        // In real implementation, would use AWS SDK
        Ok(format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.config.bucket, self.config.region, key
        ))
    }

    /// Generate presigned URL
    pub fn presign_url(&self, key: &str, expires_in: u64) -> String {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}?expires={}",
            self.config.bucket, self.config.region, key, expires_in
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config() {
        let config = S3Config {
            bucket: "test".to_string(),
            region: "us-east-1".to_string(),
            access_key: "key".to_string(),
            secret_key: "secret".to_string(),
            endpoint: None,
        };

        assert_eq!(config.bucket, "test");
    }
}
