use anyhow::Context;
use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use std::time::Duration;

use crate::{ApiResponse, ListContentResponse, PresignedUrlResponse};

pub async fn generate_presigned_url(
    client: &Client,
    action: &str,
    key: &str,
    request_id: &str,
) -> anyhow::Result<ApiGatewayProxyResponse> {
    let bucket_name =
        std::env::var("BUCKET_NAME").context("No BUCKET_NAME in the environment variables")?;

    let expires_in = Duration::from_secs(3600); // 1 hour
    let presigned_config =
        PresigningConfig::expires_in(expires_in).context("Failed to create presigning config")?;

    let presigned_req = match action {
        "GetObject" => {
            client
                .get_object()
                .bucket(&bucket_name)
                .key(key)
                .presigned(presigned_config)
                .await?
        }
        "PutObject" => {
            client
                .put_object()
                .bucket(&bucket_name)
                .key(key)
                .presigned(presigned_config)
                .await?
        }
        "DeleteObject" => {
            client
                .delete_object()
                .bucket(&bucket_name)
                .key(key)
                .presigned(presigned_config)
                .await?
        }
        _ => return Ok(ApiGatewayProxyResponse::new(400, "Invalid action on S3")),
    };

    let response = PresignedUrlResponse {
        req_id: request_id.to_string(),
        url: presigned_req.uri().to_string(),
    };

    Ok(ApiGatewayProxyResponse::new(
        200,
        &serde_json::to_string(&response)?,
    ))
}

pub async fn list_content(
    client: &Client,
    request_id: &str,
) -> anyhow::Result<ApiGatewayProxyResponse> {
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

    let response = ListContentResponse {
        req_id: request_id.to_string(),
        keys,
    };

    Ok(ApiGatewayProxyResponse::new(
        200,
        &serde_json::to_string(&response)?,
    ))
}
