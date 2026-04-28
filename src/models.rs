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

/// Schema for the multipart PDF upload endpoint (used only for OpenAPI docs).
#[allow(dead_code)]
#[derive(ToSchema)]
pub struct PdfExtractRequest {
    /// PDF file bytes (binary upload)
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
    /// Optional OCR prompt. Defaults to "Extract all text from this image."
    pub prompt: Option<String>,
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
