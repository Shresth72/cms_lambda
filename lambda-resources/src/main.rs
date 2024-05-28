mod s3;
use s3::{generate_presigned_url, list_content};

mod utils;
use utils::{empty_string_as_none, handle_response};

use aws_sdk_s3::Client;
use axum::{extract::Query, response::IntoResponse, routing::get, Extension, Json, Router};
use lambda_http::{http::Method, tracing, Error};
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
        generate_presigned_url(&client, "GetObject", key).await
    } else {
        list_content(&client).await
    };
    handle_response(response)
}

async fn put_handler(
    Query(params): Query<Params>,
    Extension(client): Extension<Client>,
) -> impl IntoResponse {
    if let Some(key) = params.key {
        match generate_presigned_url(&client, "PutObject", key).await {
            Ok(response) => Json(response),
            Err(err) => Json(json!({"error": format!("Internal Server Error: {}", err)})),
        }
    } else {
        Json(json!({"error": "Missing key parameter"}))
    }
}

async fn delete_handler(
    Query(params): Query<Params>,
    Extension(client): Extension<Client>,
) -> impl IntoResponse {
    if let Some(key) = params.key {
        match generate_presigned_url(&client, "DeleteObject", key).await {
            Ok(response) => Json(response),
            Err(err) => Json(json!({"error": format!("Internal Server Error: {}", err)})),
        }
    } else {
        Json(json!({"error": "Missing key parameter"}))
    }
}
