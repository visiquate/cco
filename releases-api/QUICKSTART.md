# Quick Start Guide - CCO Releases API

## 5-Minute Setup

### 1. Prerequisites

```bash
# Install Docker
docker --version
docker-compose --version

# Get R2 credentials from https://dash.cloudflare.com/
# Note: Account ID, Access Key, Secret Key
```

### 2. Configure Environment

```bash
# Copy template
cp .env.example .env

# Edit with your R2 credentials
nano .env
```

### 3. Run Locally

```bash
# Start services
docker compose --profile dev up

# In another terminal, test
curl http://localhost:8000/health

# Response:
# {"status":"healthy","service":"cco-releases-api","version":"0.0.0","r2_connected":true,"timestamp":"..."}
```

That's it! Service is running.

## Common Tasks

### Add Test Release to R2

```bash
# 1. Create metadata file
cat > /tmp/latest-stable.json <<'EOF'
{
  "version": "2025.11.24",
  "channel": "stable",
  "released_at": "2025-11-24T10:30:00Z",
  "platforms": {
    "darwin-arm64": "abc123",
    "darwin-x86_64": "def456",
    "linux-x86_64": "ghi789",
    "linux-aarch64": "jkl012",
    "windows-x86_64": "mno345"
  }
}
EOF

# 2. Upload to R2 (if using MinIO, use MinIO console on :9001)
aws s3 cp /tmp/latest-stable.json \
  s3://cco-releases-private/metadata/latest-stable.json \
  --endpoint-url https://{YOUR_ACCOUNT_ID}.r2.cloudflarestorage.com
```

### Get Latest Release

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://cco-api.visiquate.com/releases/latest?channel=stable

# Response:
# {"version":"2025.11.24","channel":"stable",...}
```

### Generate Download URL

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://cco-api.visiquate.com/download/2025.11.24/darwin-arm64

# Response:
# {
#   "platform": "darwin-arm64",
#   "version": "2025.11.24",
#   "download_url": "https://...",
#   "expires_in_seconds": 900
# }
```

### Run Tests

```bash
# With Docker
docker compose exec releases-api pytest test_main.py -v

# Or locally
python -m venv venv
. venv/bin/activate
pip install -r requirements-dev.txt
pytest test_main.py -v
```

### View Logs

```bash
# Docker
docker compose logs -f releases-api

# Specific errors
docker compose logs releases-api | grep ERROR
```

### Lint & Format

```bash
make lint    # Check code quality
make format  # Auto-format code
```

## Development

### Project Structure

```
releases-api/
├── main.py              # FastAPI application
├── test_main.py         # Tests
├── requirements.txt     # Dependencies
├── requirements-dev.txt # Dev tools
├── Dockerfile          # Container image
├── docker-compose.yml  # Local development
├── Makefile            # Common tasks
├── README.md           # Full documentation
├── SECURITY.md         # Security guide
├── DEPLOYMENT.md       # Production deployment
└── .env.example        # Configuration template
```

### Key Endpoints

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/health` | GET | Traefik | Service health |
| `/releases/latest` | GET | Traefik | Latest release metadata |
| `/releases/{version}` | GET | Traefik | Specific version metadata |
| `/download/{version}/{platform}` | GET | Traefik | Presigned download URL |

### Valid Platforms

- `darwin-arm64` → `cco-v{version}-darwin-arm64.tar.gz`
- `darwin-x86_64` → `cco-v{version}-darwin-x86_64.tar.gz`
- `linux-x86_64` → `cco-v{version}-linux-x86_64.tar.gz`
- `linux-aarch64` → `cco-v{version}-linux-aarch64.tar.gz`
- `windows-x86_64` → `cco-v{version}-windows-x86_64.zip`

### Channels

- `stable` → Latest stable release (default)
- `beta` → Latest beta/pre-release

## Deployment

### Docker Production

```bash
# Build
docker build -t cco-releases-api:latest .

# Run
docker run -d \
  -e R2_ACCOUNT_ID=$R2_ACCOUNT_ID \
  -e R2_ACCESS_KEY=$R2_ACCESS_KEY \
  -e R2_SECRET_KEY=$R2_SECRET_KEY \
  -p 8000:8000 \
  cco-releases-api:latest
```

### Docker Compose Production

```bash
docker compose -f docker-compose.prod.yml up -d
```

### Kubernetes

See DEPLOYMENT.md for complete guide.

## Troubleshooting

### "R2 connection failed"

```bash
# Check environment
docker compose exec releases-api env | grep R2_

# Verify R2 credentials are correct
# Test with AWS CLI
aws s3 ls s3://cco-releases-private/ \
  --endpoint-url https://{YOUR_ACCOUNT_ID}.r2.cloudflarestorage.com \
  --access-key-id YOUR_ACCESS_KEY \
  --secret-access-key YOUR_SECRET_KEY
```

### "Metadata not found (404)"

```bash
# Check if metadata files exist in R2
aws s3 ls s3://cco-releases-private/metadata/ \
  --endpoint-url https://{YOUR_ACCOUNT_ID}.r2.cloudflarestorage.com

# Upload test metadata
aws s3 cp metadata/latest-stable.json \
  s3://cco-releases-private/metadata/latest-stable.json \
  --endpoint-url https://{YOUR_ACCOUNT_ID}.r2.cloudflarestorage.com
```

### "Invalid platform" (400)

```bash
# Valid platforms only:
# darwin-arm64, darwin-x86_64, linux-x86_64, linux-aarch64, windows-x86_64

# ✓ Correct
curl https://cco-api.visiquate.com/download/2025.11.24/darwin-arm64

# ✗ Wrong
curl https://cco-api.visiquate.com/download/2025.11.24/mac-arm64
```

## Next Steps

1. Read **README.md** for full API documentation
2. Review **SECURITY.md** for credential best practices
3. Check **DEPLOYMENT.md** for production setup
4. Run `make help` for available commands

## Questions?

See README.md or SECURITY.md, or create an issue in the repository.

---

**Last Updated**: 2025-11-24
**Version**: 2025.11.24
