mod s3;
use s3::S3ClientTrait;

mod utils;
use utils::{empty_string_as_none, handle_response};

mod test;

use aws_sdk_s3::Client;
use axum::{extract::Query, response::IntoResponse, routing::get, Extension, Json, Router};
use lambda_http::{
    http::{Method, StatusCode},
    tracing, Error,
};
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
#[allow(dead_code)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    tracing::init_default_subscriber();

    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::PUT, Method::DELETE])
        .allow_origin(Any); // TODO: Add Cors for our APiGateway Endpoint only

    let app = Router::new()
        .route(
            "/",
            get(get_handler).put(put_handler).delete(delete_handler),
        )
        .layer(cors_layer)
        .layer(Extension(s3_client));

    lambda_http::run(app).await
}

async fn get_handler(
    Query(params): Query<Params>,
    Extension(client): Extension<Client>,
) -> impl IntoResponse {
    let response = if let Some(key) = params.key {
        client.generate_presigned_url("GetObject", key).await
    } else {
        client.list_content().await
    };
    handle_response(response)
}

async fn put_handler(
    Query(params): Query<Params>,
    Extension(client): Extension<Client>,
) -> impl IntoResponse {
    if let Some(key) = params.key {
        match client.generate_presigned_url("PutObject", key).await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Internal Server Error: {}", err)})),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing key parameter"})),
        )
            .into_response()
    }
}

async fn delete_handler(
    Query(params): Query<Params>,
    Extension(client): Extension<Client>,
) -> impl IntoResponse {
    if let Some(key) = params.key {
        match client.generate_presigned_url("DeleteObject", key).await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Internal Server Error: {}", err)})),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing key parameter"})),
        )
            .into_response()
    }
}
