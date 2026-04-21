use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::env;

pub struct ApiKey;

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let configured_key = env::var("API_KEY").unwrap_or_else(|_| "".to_string());

        if configured_key.is_empty() {
            tracing::warn!("API_KEY environment variable is not set. All requests will be rejected.");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server Configuration Error"));
        }

        // Check X-API-Key header
        if let Some(api_key_header) = parts.headers.get("x-api-key") {
            if let Ok(api_key_str) = api_key_header.to_str() {
                if api_key_str == configured_key {
                    return Ok(ApiKey);
                }
            }
        }

        // Check Authorization: Bearer
        if let Some(auth_header) = parts.headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if token == configured_key {
                        return Ok(ApiKey);
                    }
                }
            }
        }

        Err((StatusCode::UNAUTHORIZED, "Invalid or missing API Key"))
    }
}
