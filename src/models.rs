use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    pub status: String,
    pub message: String,
    pub model_available: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ExtractRequest {
    /// Base64 encoded image string
    pub image_base64: String,
    /// Optional prompt, defaults to extracting all text
    pub prompt: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ExtractResponse {
    pub text: String,
}

// Ollama API Request Models
#[derive(Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub images: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct OllamaGenerateResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Deserialize)]
pub struct OllamaModelList {
    pub models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
pub struct OllamaModel {
    pub name: String,
}
