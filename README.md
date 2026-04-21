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

## Exposing via Cloudflare Tunnels (Production Setup)
To securely expose this API gateway to the internet as an "always-on" service without opening firewall ports:

1. **Install `cloudflared`:**
   ```bash
   curl -L --output cloudflared.deb https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb
   sudo dpkg -i cloudflared.deb
   ```

2. **Login and Create Tunnel:**
   ```bash
   cloudflared tunnel login
   cloudflared tunnel create vision-api
   ```
   *Note the Tunnel ID that is outputted.*

3. **Route Traffic:**
   ```bash
   # Replace with your domain
   cloudflared tunnel route dns vision-api vision.yourdomain.com
   ```

4. **Configure System Service Paths:**
   Because `cloudflared service install` requires root access, it expects the config and credentials in `/etc/cloudflared/`, not your user directory.
   ```bash
   sudo mkdir -p /etc/cloudflared
   sudo cp ~/.cloudflared/*.json /etc/cloudflared/
   sudo nano /etc/cloudflared/config.yml
   ```

5. **Create the Configuration:**
   Paste the following into `/etc/cloudflared/config.yml` (replace `<Tunnel-ID>`):
   ```yaml
   tunnel: <Tunnel-ID>
   credentials-file: /etc/cloudflared/<Tunnel-ID>.json

   ingress:
     - hostname: vision.yourdomain.com
       service: http://localhost:8000
     - service: http_status:404
   ```

6. **Install and Start the Background Service:**
   ```bash
   sudo cloudflared service install
   sudo systemctl enable cloudflared
   sudo systemctl start cloudflared
   ```
   *Your API and Status Dashboard are now persistently available and will auto-restart on server reboot.*

## Features
- **UI Dashboard**: Visit the root URL for a sleek, real-time server status page.
- **Swagger UI**: Automatically generated API reference available at `/swagger-ui`.
- **Security**: All endpoints protected by `X-API-Key` verification.
