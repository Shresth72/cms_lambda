use axum::{response::IntoResponse, Json};
use lambda_http::http::StatusCode;
use serde::{de, Deserialize, Deserializer};
use serde_json::{json, Value};
use std::{fmt, str::FromStr};

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

pub fn handle_response(response: anyhow::Result<Value>) -> impl IntoResponse {
    match response {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Internal Server Error: {}", err)})),
        )
            .into_response(),
    }
}
