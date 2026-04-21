# Vision API Gateway

A high-performance, secure Rust API Gateway that proxies document vision requests to a local Ollama instance running `glm-ocr:latest`. It features API Key authentication and a beautiful modern status dashboard.

## Deployment on Host Server

This project is designed to be cloned directly onto the server hosting your Ollama instance for zero-latency communication.

### Prerequisites
- Rust and Cargo (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Ollama running locally

### 1. Setup Environment
Copy the example environment file and configure your API key.

```bash
cp .env.example .env
```
Edit `.env` to set your custom `API_KEY`. If Ollama is running on the same server, `OLLAMA_BASE_URL=http://127.0.0.1:11434` will work perfectly.

### 2. Build and Run
Compile the application in release mode for maximum performance:

```bash
cargo build --release
./target/release/mtl-vision
```
*Alternatively, you can just run `cargo run --release`.*

The server will start on `http://0.0.0.0:8000` (or whatever `PORT` you specified).

## Exposing via Cloudflare Tunnels
To securely expose this API gateway to the internet without opening firewall ports:

1. Install `cloudflared` on the server.
2. Login to Cloudflare:
   ```bash
   cloudflared tunnel login
   ```
3. Create a tunnel and route traffic to the local port:
   ```bash
   cloudflared tunnel --url http://localhost:8000
   ```
*(For production, it is recommended to set up the tunnel as a background service via the Cloudflare Zero Trust dashboard).*

## Features
- **UI Dashboard**: Visit the root URL for a sleek, real-time server status page.
- **Swagger UI**: Automatically generated API reference available at `/swagger-ui`.
- **Security**: All endpoints protected by `X-API-Key` verification.
