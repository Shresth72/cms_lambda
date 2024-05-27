#[allow(unused)]
mod get;
mod post;
mod utils;

use aws_lambda_events::{
    event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use aws_sdk_s3::Client;
use get::handle_get;
use lambda_runtime::{run, service_fn, tracing, LambdaEvent};

trait ApiResponse {
    fn new(status_code: i64, body: &str) -> Self;
}

impl ApiResponse for ApiGatewayProxyResponse {
    fn new(status_code: i64, body: &str) -> Self {
        ApiGatewayProxyResponse {
            status_code,
            headers: HeaderMap::default(),
            multi_value_headers: HeaderMap::default(),
            body: Some(aws_lambda_events::encodings::Body::Text(body.to_string())),
            is_base64_encoded: false,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    tracing::init_default_subscriber();

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
    match event.payload.http_method.as_str() {
        "GET" => handle_get(event, client).await,

        "POST" => {
            let _k = 3;
            return Ok(ApiGatewayProxyResponse::new(
                200,
                "Method not implemented yet",
            ));
        }

        _ => return Ok(ApiGatewayProxyResponse::new(405, "Method not allowed")),
    }
}
