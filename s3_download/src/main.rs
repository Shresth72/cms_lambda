use lambda_runtime::{service_fn, tracing, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    key: String,
}

#[derive(Serialize)]
struct S3Response {}

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
    let _bucket =
        std::env::var("BUCKET_NAME").expect("No BUCKET_NAME in the environment variables");
    let _key = &event.payload.key;

    // let client = S3Client::ne

    let resp = S3Response {};
    Ok(resp)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::init_default_subscriber();

    let func = service_fn(function_handler);
    lambda_runtime::run(func).await.unwrap();

    Ok(())
}
