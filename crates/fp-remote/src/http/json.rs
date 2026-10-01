//! Request bodies: JSON only, within the body limit, with errors in the
//! API's format.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, header};
use serde::de::DeserializeOwned;

use crate::api::ApiError;

pub struct JsonBody<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for JsonBody<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let is_json = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"));
        if !is_json {
            return Err(ApiError::UnsupportedMediaType);
        }
        let bytes = Bytes::from_request(req, state).await.map_err(|r| {
            if r.status() == StatusCode::PAYLOAD_TOO_LARGE {
                ApiError::PayloadTooLarge
            } else {
                ApiError::BadRequest(r.body_text())
            }
        })?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| ApiError::BadRequest(e.to_string()))?;
        // serde also reads a struct from an array; the API only takes objects.
        if !value.is_object() {
            return Err(ApiError::BadRequest(
                "the body must be a JSON object".to_owned(),
            ));
        }
        T::deserialize(value)
            .map(JsonBody)
            .map_err(|e| ApiError::BadRequest(e.to_string()))
    }
}
