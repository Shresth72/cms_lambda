use anyhow::Context;
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use serde_json::{json, Value};
use std::time::Duration;

#[async_trait::async_trait]
pub trait S3ClientTrait {
    async fn generate_presigned_url(&self, action: &str, key: String) -> anyhow::Result<Value>;
    async fn list_content(&self) -> anyhow::Result<Value>;
}

#[async_trait::async_trait]
impl S3ClientTrait for Client {
    async fn generate_presigned_url(&self, action: &str, key: String) -> anyhow::Result<Value> {
        let bucket_name =
            std::env::var("BUCKET_NAME").context("No BUCKET_NAME in the environment variables")?;

        let expires_in = Duration::from_secs(3600); // 1 hour
        let presigning_config = PresigningConfig::expires_in(expires_in)
            .context("Failed to create presigning config")?;

        let presigned_req = match action {
            "GetObject" => {
                self.get_object()
                    .bucket(&bucket_name)
                    .key(key)
                    .presigned(presigning_config)
                    .await?
            }
            "PutObject" => {
                self.put_object()
                    .bucket(&bucket_name)
                    .key(key)
                    .presigned(presigning_config)
                    .await?
            }
            "DeleteObject" => {
                self.delete_object()
                    .bucket(&bucket_name)
                    .key(key)
                    .presigned(presigning_config)
                    .await?
            }
            _ => return Ok(json!({"error": "Invalid action on S3"})),
        };

        Ok(json!({"url": presigned_req.uri().to_string()}))
    }

    async fn list_content(&self) -> anyhow::Result<Value> {
        let bucket_name =
            std::env::var("BUCKET_NAME").context("No BUCKET_NAME in the environment variables")?;

        let result = self
            .list_objects_v2()
            .bucket(&bucket_name)
            .send()
            .await
            .context("Failed to list objects in S3")?;

        let keys: Vec<String> = result
            .contents
            .unwrap_or_default()
            .into_iter()
            .filter_map(|object| object.key)
            .collect();

        Ok(json!({"keys": keys}))
    }
}

#[allow(unused)]
#[cfg(test)]
mod test {
    use super::*;
    use mockall::{mock, predicate::*};

    mock! {
        pub S3Client {}

        #[async_trait::async_trait]
        impl S3ClientTrait for S3Client {
            async fn generate_presigned_url(&self, action: &str, key: String) -> anyhow::Result<Value>;
            async fn list_content(&self) -> anyhow::Result<Value>;
        }
    }

    #[tokio::test]
    async fn test_generate_presigned_url_get() {
        let mut mock_client = MockS3Client::new();
        let key = "test-key".to_string();

        mock_client
            .expect_generate_presigned_url()
            .with(eq("GetObject"), eq(key.clone()))
            .returning(move |_, _| Ok(json!({"url": ""})));

        let result = mock_client
            .generate_presigned_url("GetObject", key)
            .await
            .unwrap();

        assert!(result.get("url").is_some());
    }
}
