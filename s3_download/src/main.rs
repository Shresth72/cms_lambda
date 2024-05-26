use anyhow::anyhow;
use lambda_runtime::{service_fn, tracing, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    key: String,
}

#[derive(Serialize)]
struct S3Response {
    req_id: String,
    msg: String,
}

// async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
//     // Extract some useful information from the request
//     let who = event
//         .query_string_parameters_ref()
//         .and_then(|params| params.first("name"))
//         .unwrap_or("world");
//     let message = format!("Hello {who}, this is an AWS Lambda HTTP request");
//
//     // Return something that implements IntoResponse.
//     // It will be serialized to the right response event automatically by the runtime
//     let resp = Response::builder()
//         .status(200)
//         .header("content-type", "text/html")
//         .body(message.into())
//         .map_err(Box::new)?;
//     Ok(resp)
// }

async fn function_handler(event: LambdaEvent<Request>) -> anyhow::Result<S3Response> {
    // let (event, _context) = event.into_parts();
    let bucket = std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables");
    let key = &event.payload.key;

    let started_at = std::time::Instant::now();

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);

    let object = client.get_object().bucket(bucket).key(key).send().await?;
    let Ok(Some(mut body)) = object.body.try_into() else {
        return Err(anyhow!("No body found in S3 response"));
    };

    // For each line in the ByteStream of S3 data
    // Parse the JSON object
    let mut byte_count = 0_usize;
    let mut num_log_events = 0;
    while let Some(bytes) = body.try_next().await? {
        let bytes_len = bytes.len();

        byte_count += bytes_len;
        num_log_events += 1;

        if num_log_events % 1000 == 0 {
            println!("num_log_events={}", num_log_events);
        }
    }

    let msg = format!(
        "elapsed={:?} | num_log_events={} | bytes_count={}",
        started_at.elapsed(),
        num_log_events,
        byte_count
    );

    let resp = S3Response {
        req_id: event.context.request_id,
        msg,
    };

    Ok(resp)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::init_default_subscriber();

    let func = service_fn(function_handler);
    lambda_runtime::run(func).await.unwrap();

    Ok(())
}
