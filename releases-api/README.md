# CCO Releases API

FastAPI service for private binary distribution with CloudFlare R2 backend and Authentik authentication via Traefik.

## Overview

This service provides authenticated endpoints for downloading CCO binaries:
- `GET /releases/latest` - Get latest release metadata
- `GET /releases/{version}` - Get specific version metadata
- `GET /download/{version}/{platform}` - Generate 15-minute presigned R2 URLs
- `GET /health` - Service health check

**Authentication**: Handled by Traefik Forward Auth middleware. No auth logic needed in this service.

## Architecture

```
CCO Client (with Bearer token)
    ↓
Traefik (cco-api.visiquate.com)
    ↓
Authentik Forward Auth Middleware
    ↓
Releases API (FastAPI)
    ↓
CloudFlare R2 (presigned URLs)
```

## Setup

### 1. CloudFlare R2 Configuration

```bash
# Create R2 bucket
wrangler r2 bucket create cco-releases-private

# Generate API token
# - Go to https://dash.cloudflare.com/profile/api-tokens
# - Create token with R2 permissions
# - Note: Account ID, Access Key ID, Secret Access Key
```

### 2. Environment Variables

Create `.env` file:

```bash
cp .env.example .env
```

Then update with your R2 credentials:

```bash
R2_ACCOUNT_ID=your-account-id
R2_ACCESS_KEY=your-access-key
R2_SECRET_KEY=your-secret-key
R2_BUCKET=cco-releases-private
VERSION=2025.11.24
ENVIRONMENT=production
```

### 3. Docker Build & Run

```bash
# Build image
docker build -t cco-releases-api:latest .

# Run locally
docker compose up

# Run with MinIO (local R2 simulation)
docker compose --profile dev up

# Run in production
docker run -d \
  --name cco-releases-api \
  -p 8000:8000 \
  -e R2_ACCOUNT_ID=$R2_ACCOUNT_ID \
  -e R2_ACCESS_KEY=$R2_ACCESS_KEY \
  -e R2_SECRET_KEY=$R2_SECRET_KEY \
  cco-releases-api:latest
```

## Development

### Local Setup with MinIO

MinIO simulates R2 locally:

```bash
# Start services (includes MinIO on :9001)
docker compose --profile dev up

# MinIO console: http://localhost:9001
# Default credentials: minioadmin/minioadmin

# Access API: http://localhost:8000/health
```

### R2 Structure

```
cco-releases-private/
├── releases/
│   ├── 2025.11.24/
│   │   ├── cco-v2025.11.24-darwin-arm64.tar.gz
│   │   ├── cco-v2025.11.24-darwin-x86_64.tar.gz
│   │   ├── cco-v2025.11.24-linux-x86_64.tar.gz
│   │   ├── cco-v2025.11.24-linux-aarch64.tar.gz
│   │   ├── cco-v2025.11.24-windows-x86_64.zip
│   │   └── checksums.sha256
│   └── 2025.11.25/
│       └── ...
└── metadata/
    ├── latest-stable.json
    ├── latest-beta.json
    ├── 2025.11.24.json
    └── 2025.11.25.json
```

### Metadata Format

`metadata/latest-stable.json`:

```json
{
  "version": "2025.11.24",
  "channel": "stable",
  "released_at": "2025-11-24T10:30:00Z",
  "platforms": {
    "darwin-arm64": "abc123...",
    "darwin-x86_64": "def456...",
    "linux-x86_64": "ghi789...",
    "linux-aarch64": "jkl012...",
    "windows-x86_64": "mno345..."
  }
}
```

## API Endpoints

### GET /health

Health check with R2 connectivity verification.

**Response** (200 OK):
```json
{
  "status": "healthy",
  "service": "cco-releases-api",
  "version": "2025.11.24",
  "r2_connected": true,
  "timestamp": "2025-11-24T10:30:00Z"
}
```

### GET /releases/latest

Get latest release metadata for a channel.

**Query Parameters**:
- `channel` (string, default="stable"): "stable" or "beta"

**Response** (200 OK):
```json
{
  "version": "2025.11.24",
  "channel": "stable",
  "released_at": "2025-11-24T10:30:00Z",
  "platforms": {
    "darwin-arm64": "abc123...",
    "darwin-x86_64": "def456...",
    "linux-x86_64": "ghi789...",
    "linux-aarch64": "jkl012...",
    "windows-x86_64": "mno345..."
  }
}
```

### GET /releases/{version}

Get specific release metadata.

**Path Parameters**:
- `version` (string): Release version (e.g., "2025.11.24")

**Response** (200 OK):
```json
{
  "version": "2025.11.24",
  "channel": "stable",
  "released_at": "2025-11-24T10:30:00Z",
  "platforms": { ... }
}
```

### GET /download/{version}/{platform}

Generate presigned R2 download URL.

**Path Parameters**:
- `version` (string): Release version
- `platform` (string): One of darwin-arm64, darwin-x86_64, linux-x86_64, linux-aarch64, windows-x86_64

**Response** (200 OK):
```json
{
  "platform": "darwin-arm64",
  "version": "2025.11.24",
  "download_url": "https://<account>.r2.cloudflarestorage.com/releases/...",
  "expires_in_seconds": 900
}
```

**Error Responses**:
- 400: Invalid platform
- 404: Version not found
- 500: R2 connection error

## Security Considerations

1. **Authentication**: Handled by Traefik Forward Auth
   - API only accepts requests from Traefik (already authenticated)
   - No token validation needed in service

2. **R2 Credentials**:
   - Never commit `.env` file
   - Use IAM tokens with limited permissions (R2 access only)
   - Rotate credentials regularly

3. **Presigned URLs**:
   - 15-minute TTL prevents long-term leaks
   - R2 enforces IP/signature validation
   - Direct download from R2 (no proxy traffic)

4. **API Endpoints**:
   - Root path (/) returns 404 - no index disclosure
   - No public OpenAPI docs (docs_url=None)
   - No Swagger/ReDoc endpoints

## Traefik Integration

Docker labels for docker-compose.yml:

```yaml
services:
  cco-releases-api:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.cco-api.rule=Host(`cco-api.visiquate.com`)"
      - "traefik.http.routers.cco-api.entrypoints=websecure"
      - "traefik.http.routers.cco-api.tls.certresolver=letsencrypt"
      - "traefik.http.routers.cco-api.middlewares=authentik@docker"
      - "traefik.http.services.cco-api.loadbalancer.server.port=8000"
```

## Cost Estimate

| Item | Cost |
|------|------|
| R2 Storage (10GB) | ~$0.15/month |
| R2 Operations | ~$0.00 |
| R2 Egress | **$0.00** (FREE!) |
| Releases API | $0.00 (shared infra) |
| **Total** | **~$0.15/month** |

## Performance

- **Health check latency**: <100ms (with R2 head_bucket)
- **Metadata retrieval**: <200ms (from R2)
- **Presigned URL generation**: <50ms (local)
- **Throughput**: 1000+ requests/sec (on modest hardware)

## Monitoring

Health check runs every 30 seconds (Docker HEALTHCHECK).

**Alerts**:
- R2 connectivity failure → status="degraded"
- Service restart on 3 consecutive failures

## License

Part of CCO project. See main repository LICENSE.
