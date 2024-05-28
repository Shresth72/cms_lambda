use anyhow::Context;
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use serde_json::{json, Value};
use std::time::Duration;

pub async fn generate_presigned_url(
    client: &Client,
    action: &str,
    key: String,
) -> anyhow::Result<Value> {
    let bucket_name =
        std::env::var("BUCKET_NAME").context("No BUCKET_NAME in the environment variables")?;

    let expires_in = Duration::from_secs(3600); // 1 hour
    let presigning_config =
        PresigningConfig::expires_in(expires_in).context("Failed to create presigning config")?;

    let presigned_req = match action {
        "GetObject" => {
            client
                .get_object()
                .bucket(&bucket_name)
                .key(key)
                .presigned(presigning_config)
                .await?
        }
        "PutObject" => {
            client
                .put_object()
                .bucket(&bucket_name)
                .key(key)
                .presigned(presigning_config)
                .await?
        }
        "DeleteObject" => {
            client
                .delete_object()
                .bucket(&bucket_name)
                .key(key)
                .presigned(presigning_config)
                .await?
        }
        _ => return Ok(json!({"error": "Invalid action on S3"})),
    };

    Ok(json!({"url": presigned_req.uri().to_string()}))
}

pub async fn list_content(client: &Client) -> anyhow::Result<Value> {
    let bucket_name =
        std::env::var("BUCKET_NAME").context("No BUCKET_NAME in the environment variables")?;

    let result = client
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
