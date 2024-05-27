use std::time::Duration;

use anyhow::Context;
#[allow(unused)]
use aws_lambda_events::{
    event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
    query_map::QueryMap,
};
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use lambda_runtime::{run, service_fn, tracing, LambdaEvent};
use serde::Serialize;

#[derive(Serialize)]
struct PresignedUrlResponse {
    req_id: String,
    url: String,
}

trait ApiResponse {
    fn new(status_code: i64, body: &str) -> Self;
}

impl ApiResponse for ApiGatewayProxyResponse {
    fn new(status_code: i64, body: &str) -> Self {
        // TODO: fix headers
        let headers = HeaderMap::default();
        // headers.insert("content-type", "text/html".parse().unwrap());

        ApiGatewayProxyResponse {
            status_code,
            multi_value_headers: headers.clone(),
            is_base64_encoded: false,
            body: Some(aws_lambda_events::encodings::Body::Text(body.to_string())),
            headers,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    tracing::init_default_subscriber();

    // TODO: Add cors (tower_http::cors)
    // let cors_layer = CorsLayer::new()
    //     .allow_methods(vec![Method::GET, Method::POST])
    //     .allow_origin(Any);
    //
    // let handler = lambda_runtime::tower::ServiceBuilder::new().layer(cors_layer).service(service_fn(
    //     |event: LambdaEvent<ApiGatewayProxyRequest>| function_handler(event, &s3_client),
    // ));
    //
    // run(handler).await?;

    run(service_fn(|event: LambdaEvent<ApiGatewayProxyRequest>| {
        function_handler(event, &s3_client)
    }))
    .await
    .unwrap();

    Ok(())
}

async fn function_handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
    client: &Client,
) -> anyhow::Result<ApiGatewayProxyResponse> {
    match (
        event.payload.http_method.as_str(),
        event.payload.query_string_parameters,
    ) {
        ("GET", query_params) => {
            if let Some(key) = query_params.first("key") {
                generate_presigned_url(client, "GetObject", key, &event.context.request_id).await
            } else {
                Ok(ApiGatewayProxyResponse::new(
                    400,
                    "Missing 'key' query parameter",
                ))
            }
        }
        ("PUT", query_params) => {
            if let Some(key) = query_params.first("key") {
                generate_presigned_url(client, "PutObject", key, &event.context.request_id).await
            } else {
                Ok(ApiGatewayProxyResponse::new(
                    400,
                    "Missing 'key' query parameter",
                ))
            }
        }
        _ => Ok(ApiGatewayProxyResponse::new(405, "Method not allowed")),
    }
}

async fn generate_presigned_url(
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
