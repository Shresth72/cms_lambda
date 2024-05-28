use core::panic;
use std::collections::HashMap;

use aws_sdk_s3::{
    operation::create_multipart_upload::CreateMultipartUploadOutput, types::CompletedPart, Client,
};
use lambda_http::{
    http::Method, service_fn, tower::ServiceBuilder, tracing, Body, Error, Request, Response,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

const CHUNK_SIZE: u64 = 1024 * 1024 * 5;
const MAX_CHUNKS: u64 = 10000;

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    tracing::init_default_subscriber();

    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_origin(Any); // TODO: Add Cors for our APiGateway Endpoint only

    let handler = ServiceBuilder::new()
        .layer(cors_layer)
        .service(service_fn(func));

    lambda_http::run(handler).await
}

async fn func(event: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    match event.uri().path() {
        "/createMultiPartUpload" => create_multi_part_upload(event, &s3_client).await,
        "/completeMultiPartUpload" => complete_multipart_upload(event, &s3_client).await,
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

async fn create_multi_part_upload(
    event: Request,
    client: &Client,
) -> Result<Response<Body>, Error> {
    let body_str = String::from_utf8(event.body().to_vec()).unwrap();
    let event_body: HashMap<String, String> = serde_json::from_str(&body_str)?;

    let bucket_name =
        std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables");
    let key = event_body.get("key").unwrap();
    let file_size = event_body.get("size").unwrap().parse::<u64>().unwrap();

    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(&bucket_name)
        .key(key)
        .send()
        .await?;

    let upload_id = multipart_upload_res.upload_id().unwrap();

    let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
    let mut size_of_last_chunk = file_size % CHUNK_SIZE;
    if size_of_last_chunk == 0 {
        size_of_last_chunk = CHUNK_SIZE;
        chunk_count -= 1;
    }

    if file_size == 0 {
        panic!("Bad file size.");
    }
    if chunk_count > MAX_CHUNKS {
        panic!("Too many chunks! Try increasing your chunk size.")
    }

    let mut upload_parts: Vec<CompletedPart> = Vec::new();

    todo!()
}

async fn complete_multipart_upload(
    event: Request,
    client: &Client,
) -> Result<Response<Body>, Error> {
    todo!()
}
