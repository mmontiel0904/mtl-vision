use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::handlers::AppState;

pub struct ApiKey;

impl FromRequestParts<Arc<AppState>> for ApiKey {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let configured_key = env::var("API_KEY").unwrap_or_else(|_| "".to_string());

        let mut has_valid_key = false;

        if !configured_key.is_empty() {
            // Check X-API-Key header
            if let Some(api_key_header) = parts.headers.get("x-api-key") {
                if let Ok(api_key_str) = api_key_header.to_str() {
                    if api_key_str == configured_key {
                        has_valid_key = true;
                    }
                }
            }

            // Check Authorization: Bearer
            if let Some(auth_header) = parts.headers.get("authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        if token == configured_key {
                            has_valid_key = true;
                        }
                    }
                }
            }
        }

        if has_valid_key {
            return Ok(ApiKey);
        }

        // If API key is not valid or not provided, check IP rate limit (3 per day)
        let ip = if let Some(cf_ip) = parts.headers.get("cf-connecting-ip") {
            cf_ip.to_str().unwrap_or("unknown_ip").to_string()
        } else if let Some(forwarded) = parts.headers.get("x-forwarded-for") {
            forwarded.to_str().unwrap_or("unknown_ip").split(',').next().unwrap_or("unknown_ip").trim().to_string()
        } else if let Some(ConnectInfo(addr)) = parts.extensions.get::<ConnectInfo<SocketAddr>>() {
            addr.ip().to_string()
        } else {
            "unknown_ip".to_string()
        };

        let now = SystemTime::now();
        let one_day = Duration::from_secs(24 * 60 * 60);

        let mut limits = state.rate_limits.write().unwrap();
        let timestamps = limits.entry(ip.clone()).or_insert_with(Vec::new);

        // Remove timestamps older than 24 hours
        timestamps.retain(|&ts| {
            if let Ok(duration) = now.duration_since(ts) {
                duration < one_day
            } else {
                false
            }
        });

        if timestamps.len() < 3 {
            timestamps.push(now);
            Ok(ApiKey)
        } else {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded: 3 requests per day without API key",
            ))
        }
    }
}
