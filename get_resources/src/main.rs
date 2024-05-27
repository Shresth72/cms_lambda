#[allow(unused)]
mod utils;

use aws_sdk_s3::Client;
use lambda_runtime::{run, service_fn, tracing, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Request {
    // records: Vec<Record>,
    key: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
    byte_count: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);

    tracing::init_default_subscriber();

    run(service_fn(|event: LambdaEvent<Value>| {
        function_handler(event, &s3_client)
    }))
    .await
    .unwrap();

    Ok(())
}

async fn function_handler(event: LambdaEvent<Value>, client: &Client) -> anyhow::Result<Response> {
    let request: Request = serde_json::from_value(event.payload)?;
    // let record = &request.records[0];
    let key = &request.key;

    // send a request to S3 bucket to get the new bucket entry
    let resp = client
        .get_object()
        .bucket(std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables"))
        .key(key)
        .response_content_type("application/json")
        .send()
        .await?;

    // get the gzip compressed content
    let compressed_response = resp.body.collect().await?.to_vec();

    // decompress the content to a readable string
    let mut decoder = flate2::bufread::GzDecoder::new(&compressed_response[..]);
    let mut response_as_str = String::new();
    decoder.read_to_string(&mut response_as_str)?;

    let byte_count = response_as_str.len();

    /*
        let object = serde_json::from_str(&response_as_str);

        // upload converted json to other bucket
        if object.is_err() {
            return Err(anyhow!("error: Object from bucket is not valid JSON"));
        } else {
            let json_object: Vec<Value> = convert_aws_to_teamscale(object.unwrap());

            if json_object.len() > 0 {
            let mut new_key = key.to_string();
            new_key.push_str(".json");
            upload_json_to_bucket(&client, json_object, new_key).await?;
            }
        }
    */

    Ok(Response {
        req_id: event.context.request_id,
        msg: "Success".to_string(),
        byte_count,
    })
}

/*
#[derive(Deserialize)]
struct S3Object {
    key: String,
}

#[derive(Deserialize)]
struct S3Bucket {
    name: String,
}

#[derive(Deserialize)]
struct S3 {
    object: S3Object,
    bucket: S3Bucket,
}

#[derive(Deserialize)]
struct Record {
    s3: S3,
}
*/
