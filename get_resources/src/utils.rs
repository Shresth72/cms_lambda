use aws_sdk_s3::{
    primitives::{ByteStream, SdkBody},
    Client,
};
use serde_json::{json, Value};

pub fn convert_aws_to_teamscale(aws_object: Vec<Value>) -> Vec<Value> {
    aws_object
        .into_iter()
        .map(|obj| json!({"teamscale": obj}))
        .collect()
}

pub async fn upload_json_to_bucket(
    client: &Client,
    json_object: Vec<Value>,
    key: String,
) -> anyhow::Result<()> {
    let json_string = serde_json::to_string(&json_object)?;

    let byte_stream = ByteStream::new(SdkBody::from(json_string.as_str()));
    let target_bucket = "bucket_name";

    client
        .put_object()
        .bucket(target_bucket)
        .key(key)
        .body(byte_stream)
        .content_type("application/json")
        .send()
        .await?;

    Ok(())
}
