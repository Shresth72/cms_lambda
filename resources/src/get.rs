use aws_lambda_events::{
    event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use aws_sdk_s3::Client;
use flate2::bufread::GzDecoder;
use lambda_runtime::{run, service_fn, tracing, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::io::Read;

use crate::ApiResponse;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Request {
    key: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
    byte_count: usize,
}

pub async fn handle_get(
    event: LambdaEvent<ApiGatewayProxyRequest>,
    client: &Client,
) -> anyhow::Result<ApiGatewayProxyResponse> {
    let request: Request = match serde_json::from_str(event.payload.body.as_deref().unwrap_or("{}"))
    {
        Ok(req) => req,
        Err(_) => return Ok(ApiGatewayProxyResponse::new(400, "Invalid request body")),
    };

    let key = &request.key;

    let resp = match client
        .get_object()
        .bucket(std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables"))
        .key(key)
        .response_content_type("application/json")
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return Ok(ApiGatewayProxyResponse::new(
                500,
                "Failed to fetch object from S3",
            ))
        }
    };

    let compressed_response = resp.body.collect().await?.to_vec();

    let mut decoder = GzDecoder::new(&compressed_response[..]);
    let mut response_as_str = String::new();
    decoder.read_to_string(&mut response_as_str)?;

    let byte_count = response_as_str.len();

    let response = Response {
        req_id: event.context.request_id,
        msg: "Success".to_string(),
        byte_count,
    };

    Ok(ApiGatewayProxyResponse::new(
        200,
        &serde_json::to_string(&response)?,
    ))
}
