use aws_sdk_s3::{
    operation::create_multipart_upload::CreateMultipartUploadOutput,
    presigning::PresigningConfig,
    types::{CompletedMultipartUpload, CompletedPart},
    Client,
};
use core::panic;
use lambda_http::{
    http::Method, service_fn, tower::ServiceBuilder, tracing, Body, Error, Request, Response,
};
use serde::Serialize;
use std::{collections::HashMap, time::Duration};
use tower_http::cors::{Any, CorsLayer};

const CHUNK_SIZE: u64 = 1024 * 1024 * 5;
const MAX_CHUNKS: u64 = 10000;

#[tokio::main]
async fn main() -> anyhow::Result<(), Error> {
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

async fn func(event: Request) -> anyhow::Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    match event.uri().path() {
        "/createMultiPartUpload" => create_multi_part_upload(event, &s3_client).await,
        "/completeMultiPartUpload" => complete_multipart_upload(event, &s3_client).await,
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))?),
    }
}

async fn create_multi_part_upload(
    event: Request,
    client: &Client,
) -> anyhow::Result<Response<Body>, Error> {
    let body_str = String::from_utf8(event.body().to_vec())?;
    let event_body: HashMap<String, String> = serde_json::from_str(&body_str)?;

    let key = event_body.get("key").unwrap();
    let file_size = event_body.get("size").unwrap().parse::<u64>().unwrap();

    let bucket_name =
        std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables");

    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(&bucket_name)
        .key(key)
        .send()
        .await?;

    let upload_id = multipart_upload_res.upload_id().unwrap();

    let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
    let mut _size_of_last_chunk = file_size % CHUNK_SIZE;
    if _size_of_last_chunk == 0 {
        _size_of_last_chunk = CHUNK_SIZE;
        chunk_count -= 1;
    }

    if file_size == 0 {
        panic!("Bad file size.");
    }
    if chunk_count > MAX_CHUNKS {
        panic!("Too many chunks! Try increasing your chunk size.")
    }

    let mut signed_urls = Vec::new();

    for chunk_index in 0..chunk_count {
        // Client side logic
        // let this_chunk = if chunk_count - 1 == chunk_index {
        //     size_of_last_chunk
        // } else {
        //     CHUNK_SIZE
        // };

        let part_number = (chunk_index as i32) + 1;
        let expires_in = Duration::from_secs(600);
        let presigning_config = PresigningConfig::expires_in(expires_in)?;

        let signed_url = client
            .upload_part()
            .key(key)
            .bucket(&bucket_name)
            .upload_id(upload_id)
            .part_number(part_number)
            .presigned(presigning_config)
            .await?;

        signed_urls.push(signed_url.uri().to_string());
    }

    let response = MultiPartUploadResponse {
        upload_id: upload_id.to_string(),
        signed_urls,
    };

    let json_response = serde_json::to_string(&response)?;
    Ok(Response::builder()
        .status(200)
        .body(Body::from(json_response))?)
}

#[derive(Serialize)]
struct MultiPartUploadResponse {
    upload_id: String,
    signed_urls: Vec<String>,
}

// Recieved Completed Part Response should have etag and part_number
// from the client side
async fn complete_multipart_upload(
    event: Request,
    client: &Client,
) -> anyhow::Result<Response<Body>, Error> {
    let body_str = String::from_utf8(event.body().to_vec())?;
    let event_body: HashMap<String, String> = serde_json::from_str(&body_str)?;

    let key = event_body.get("key").unwrap();
    let upload_id = event_body.get("upload_id").unwrap();
    let upload_parts: Vec<HashMap<String, String>> =
        serde_json::from_str(event_body.get("parts").unwrap())?;

    let bucket_name =
        std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables");

    let mut completed_parts = Vec::new();
    for part in upload_parts {
        completed_parts.push(
            CompletedPart::builder()
                .e_tag(part.get("etag").unwrap())
                .part_number(part.get("part_number").unwrap().parse::<i32>().unwrap())
                .build(),
        );
    }

    let completed_multipart_upload = CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts))
        .build();

    let _complete_multipart_upload_res = client
        .complete_multipart_upload()
        .bucket(&bucket_name)
        .key(key)
        .multipart_upload(completed_multipart_upload)
        .upload_id(upload_id)
        .send()
        .await?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from("Multipart upload completed"))?)
}
