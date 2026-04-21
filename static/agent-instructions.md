# Vision API Agent Instructions

This document provides instructions for AI agents and automated systems on how to interact with the Vision API.

## Base URL
When exposed via Cloudflare tunnel, use the provided custom domain. Locally, the default is `http://localhost:8000`.

## Authentication
All API endpoints (except serving static files and Swagger UI) require authentication.

Provide the API Key in the headers of your request:
```http
x-api-key: <YOUR_API_KEY>
```
*Alternatively, you can use `Authorization: Bearer <YOUR_API_KEY>`.*

## Endpoints

### 1. Extract Text from Image
**POST** `/api/v1/vision/extract`

Extracts text from a base64 encoded image using `glm-ocr:latest`.

**Request Body (JSON):**
```json
{
  "image_base64": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
  "prompt": "Extract all text from this document."
}
```
*Note: `prompt` is optional. If omitted, it defaults to extracting all text.*

**Response (JSON):**
```json
{
  "text": "The extracted text goes here..."
}
```

### 2. Check Server Status
**GET** `/api/v1/status`

Checks if the Ollama server is reachable and if the `glm-ocr` model is available.

**Response (JSON):**
```json
{
  "status": "online",
  "message": "Ollama server is reachable",
  "model_available": true
}
```

## Error Handling
If you receive a `401 Unauthorized`, check your `x-api-key`.
If you receive a `500 Internal Server Error`, the backend Ollama service may be down or the model might not be installed on the host machine.
