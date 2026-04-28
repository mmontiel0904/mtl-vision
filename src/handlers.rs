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

const MAX_PAGES: usize = 50;

async fn call_ollama_ocr(
    state: &Arc<AppState>,
    base64_image: String,
    prompt: &str,
) -> Result<String, (StatusCode, String)> {
    let url = format!("{}/api/generate", state.ollama_url);

    let ollama_req = OllamaGenerateRequest {
        model: "glm-ocr:latest".to_string(),
        prompt: prompt.to_string(),
        stream: false,
        images: vec![base64_image],
    };

    let res = state
        .http_client
        .post(&url)
        .json(&ollama_req)
        .send()
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to connect to Ollama: {}", e),
        ))?;

    if !res.status().is_success() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ollama returned an error: {}", res.status()),
        ));
    }

    let ollama_resp: OllamaGenerateResponse = res.json().await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to parse Ollama generation: {}", e),
    ))?;

    Ok(ollama_resp.response)
}

#[utoipa::path(
    get,
    path = "/api/v1/status",
    responses(
        (status = 200, description = "Status of the API and Ollama Server", body = StatusResponse)
    )
)]
pub async fn status_handler(
    State(state): State<Arc<AppState>>,
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
    let prompt = payload.prompt.unwrap_or_else(|| "Extract all text from this image.".to_string());
    let text = call_ollama_ocr(&state, payload.image_base64, &prompt).await?;
    Ok(Json(ExtractResponse { text }))
}

#[utoipa::path(
    post,
    path = "/api/v1/vision/extract/pdf",
    request_body(
        content = inline(crate::models::PdfExtractRequest),
        description = "Multipart form with 'file' (PDF binary) and optional 'prompt' text field",
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "Extracted text from all PDF pages with --- Page N --- separators", body = ExtractResponse),
        (status = 400, description = "Missing 'file' field or malformed multipart"),
        (status = 500, description = "Ollama error or libpdfium not found"),
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn pdf_extract_handler(
    State(state): State<Arc<AppState>>,
    _api_key: crate::security::ApiKey,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<ExtractResponse>, (StatusCode, String)> {
    let mut pdf_bytes: Option<Vec<u8>> = None;
    let mut prompt_text: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        // Copy name before consuming field
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read PDF bytes: {}", e)))?;
                pdf_bytes = Some(bytes.to_vec());
            }
            "prompt" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read prompt field: {}", e)))?;
                prompt_text = Some(text);
            }
            _ => {}
        }
    }

    let pdf_bytes = pdf_bytes
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'file' field in multipart".to_string()))?;

    let prompt = prompt_text.unwrap_or_else(|| "Extract all text from this image.".to_string());

    // PDF rendering is CPU-bound and pdfium types are !Send — must run in a blocking thread
    let base64_pages: Vec<String> = tokio::task::spawn_blocking(move || -> Result<Vec<String>, String> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        use image::ImageFormat;
        use pdfium_render::prelude::*;
        use std::io::Cursor;

        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .map_err(|e| format!(
                    "libpdfium not found: {}. Install from https://github.com/bblanchon/pdfium-binaries and run ldconfig.",
                    e
                ))?,
        );

        let document = pdfium
            .load_pdf_from_byte_slice(&pdf_bytes, None)
            .map_err(|e| format!("Failed to parse PDF: {}", e))?;

        let total_pages = document.pages().len() as usize;
        let pages_to_render = total_pages.min(MAX_PAGES);

        if total_pages > MAX_PAGES {
            tracing::warn!(
                total_pages,
                cap = MAX_PAGES,
                "PDF exceeds page cap; truncating to {} pages",
                MAX_PAGES
            );
        }

        let mut result: Vec<String> = Vec::with_capacity(pages_to_render);

        for i in 0..pages_to_render {
            let page = document
                .pages()
                .get(i as u16)
                .map_err(|e| format!("Failed to get page {}: {}", i + 1, e))?;

            let render_config = PdfRenderConfig::new().set_target_width(2048);

            let bitmap = page
                .render_with_config(&render_config)
                .map_err(|e| format!("Failed to render page {}: {}", i + 1, e))?;

            let img = bitmap.as_image();

            let mut png_buffer: Vec<u8> = Vec::new();
            img.write_to(&mut Cursor::new(&mut png_buffer), ImageFormat::Png)
                .map_err(|e| format!("Failed to PNG-encode page {}: {}", i + 1, e))?;

            result.push(STANDARD.encode(&png_buffer));
        }

        Ok(result)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("PDF rendering thread panicked: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut page_texts: Vec<String> = Vec::with_capacity(base64_pages.len());

    for (i, page_b64) in base64_pages.into_iter().enumerate() {
        let page_text = call_ollama_ocr(&state, page_b64, &prompt).await?;
        page_texts.push(format!("--- Page {} ---\n{}", i + 1, page_text));
    }

    Ok(Json(ExtractResponse {
        text: page_texts.join("\n\n"),
    }))
}
