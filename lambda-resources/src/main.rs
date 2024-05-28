mod utils;
use utils::{generate_presigned_url, list_content};

use aws_lambda_events::{
    event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use aws_sdk_s3::Client;
use lambda_runtime::{run, service_fn, tracing, LambdaEvent};
use serde::Serialize;

#[derive(Serialize)]
struct PresignedUrlResponse {
    req_id: String,
    url: String,
}

#[derive(Serialize)]
struct ListContentResponse {
    req_id: String,
    keys: Vec<String>,
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
    // Add cors

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
                list_content(client, &event.context.request_id).await
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
        ("DELETE", query_params) => {
            if let Some(key) = query_params.first("key") {
                generate_presigned_url(client, "DeleteObject", key, &event.context.request_id).await
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
