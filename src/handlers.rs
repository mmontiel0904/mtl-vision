use axum::{extract::State, http::StatusCode, Json};
use reqwest::Client;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::models::{
    ExtractRequest, ExtractResponse, OllamaGenerateRequest, OllamaGenerateResponse, OllamaModelList,
    StatusResponse,
};

pub struct AppState {
    pub http_client: Client,
    pub ollama_url: String,
    pub rate_limits: RwLock<HashMap<String, Vec<SystemTime>>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/status",
    responses(
        (status = 200, description = "Status of the API and Ollama Server", body = StatusResponse)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn status_handler(
    State(state): State<Arc<AppState>>,
    _api_key: crate::security::ApiKey,
) -> Result<Json<StatusResponse>, (StatusCode, String)> {
    let url = format!("{}/api/tags", state.ollama_url);
    
    let res = state.http_client.get(&url).send().await;

    match res {
        Ok(response) if response.status().is_success() => {
            let body: OllamaModelList = response.json().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse Ollama response: {}", e),
                )
            })?;

            let has_model = body.models.iter().any(|m| m.name.contains("glm-ocr"));

            Ok(Json(StatusResponse {
                status: "online".to_string(),
                message: "Ollama server is reachable".to_string(),
                model_available: has_model,
            }))
        }
        _ => Ok(Json(StatusResponse {
            status: "offline".to_string(),
            message: "Cannot reach Ollama server".to_string(),
            model_available: false,
        })),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/vision/extract",
    request_body = ExtractRequest,
    responses(
        (status = 200, description = "Extracted text from the image", body = ExtractResponse),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn extract_handler(
    State(state): State<Arc<AppState>>,
    _api_key: crate::security::ApiKey,
    Json(payload): Json<ExtractRequest>,
) -> Result<Json<ExtractResponse>, (StatusCode, String)> {
    let url = format!("{}/api/generate", state.ollama_url);
    
    // Default prompt for glm-ocr
    let prompt = payload.prompt.unwrap_or_else(|| "Extract all text from this image.".to_string());

    let ollama_req = OllamaGenerateRequest {
        model: "glm-ocr:latest".to_string(), // Or configure via env
        prompt,
        stream: false,
        images: vec![payload.image_base64],
    };

    let res = state
        .http_client
        .post(&url)
        .json(&ollama_req)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to connect to Ollama: {}", e),
            )
        })?;

    if !res.status().is_success() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ollama returned an error: {}", res.status()),
        ));
    }

    let ollama_resp: OllamaGenerateResponse = res.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse Ollama generation: {}", e),
        )
    })?;

    Ok(Json(ExtractResponse {
        text: ollama_resp.response,
    }))
}
