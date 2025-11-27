# CCO Infrastructure Quick Start

Complete infrastructure configuration for CCO Private Binary Distribution with local Docker volume storage and Traefik reverse proxy.

## What Was Created

```
infrastructure/
├── authentik/              (Existing - OAuth2/OIDC setup)
├── scripts/                (NEW - Deployment utilities)
│   └── upload-release.sh  - Upload release binaries to API
└── traefik/               (NEW - Reverse proxy)
    ├── docker-compose.yml - Traefik + API + Auth
    ├── traefik.yml        - Static config
    ├── dynamic/cco-api.yml - Dynamic routing
    ├── .env.example       - Environment template
    └── README.md          - Complete deployment guide
```

## 5-Minute Setup Overview

### 1. Prerequisites (Before Starting)

You need:
- Authentik running at https://auth.visiquate.com
- Authentik Outpost token (get from Authentik admin UI)
- Docker & Docker Compose installed
- DNS pointing `cco-api.visiquate.com` to your host
- Sufficient disk space for release binaries (recommend 50GB+)

### 2. Step 1: Generate Upload API Key (1 minute)

```bash
# Generate a secure API key for release uploads
UPLOAD_API_KEY=$(openssl rand -base64 32)
echo "UPLOAD_API_KEY=${UPLOAD_API_KEY}"

# Save this value - you'll need it for .env file
```

### 3. Step 2: Configure Traefik (5 minutes)

```bash
cd /Users/brent/git/cc-orchestra/infrastructure/traefik

# Create environment file
cp .env.example .env

# Edit with your values
nano .env
# Required: AUTHENTIK_OUTPOST_TOKEN, LETSENCRYPT_EMAIL, UPLOAD_API_KEY
```

### 4. Step 3: Create Storage Directories (1 minute)

```bash
# Create local storage volume (Docker will handle this automatically, but you can pre-create if desired)
# The cco-releases volume will be created by Docker when services start
```

### 5. Step 4: Deploy Services (2 minutes)

```bash
cd /Users/brent/git/cc-orchestra/infrastructure/traefik

# Start all services
docker-compose up -d

# Verify status
docker-compose ps
```

### 6. Step 5: Test It Works (3 minutes)

```bash
# Get authentication token (via Authentik device flow or manually)
export ACCESS_TOKEN="your-bearer-token"

# Test unauthenticated request (should fail)
curl https://cco-api.visiquate.com/releases/latest

# Test authenticated request (should work)
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  https://cco-api.visiquate.com/releases/latest

# Should return: {"version": "...", "platforms": {...}}
```

## Common Commands

### Upload a Release

```bash
cd /Users/brent/git/cc-orchestra/infrastructure/scripts

# Set environment variables
export UPLOAD_API_KEY="your-api-key-from-env"
export ACCESS_TOKEN="your-authentik-bearer-token"
export CCO_API_URL="https://cco-api.visiquate.com"

# Upload release binaries
./upload-release.sh 2025.11.1 /path/to/binaries/

# Binaries should be named: cco-v2025.11.1-{platform}.{tar.gz|zip}
```

### Check Services

```bash
cd /Users/brent/git/cc-orchestra/infrastructure/traefik

# Status
docker-compose ps

# Logs
docker-compose logs -f traefik
docker-compose logs -f cco-releases-api
docker-compose logs -f authentik-outpost
```

### View TLS Certificate

```bash
cd /Users/brent/git/cc-orchestra/infrastructure/traefik

# View certificate details
docker exec traefik-cco-api cat /letsencrypt/acme.json | jq .
```

## Architecture at a Glance

```
User with Token
    │
    ├─ HTTPS (443)
    │
    ├─ Traefik (Reverse Proxy)
    │  ├─ TLS Termination
    │  ├─ Let's Encrypt Certificate
    │  └─ Docker Socket for service discovery
    │
    ├─ Authentik Outpost (Forward Auth)
    │  ├─ Validates Bearer token
    │  ├─ Checks cco-users group membership
    │  └─ Rejects unauthenticated requests
    │
    ├─ CCO Releases API (FastAPI)
    │  ├─ GET /releases/latest → Returns metadata
    │  ├─ GET /download/{version}/{platform} → Returns download URL
    │  ├─ POST /upload → Receives binary uploads (API key auth)
    │  └─ GET /health → Returns service status
    │
    └─ Local Docker Volume Storage
       ├─ cco-releases volume
       ├─ Mounted at /data/releases
       ├─ Persistent across container restarts
       └─ Binary storage and metadata
```

## Environment Variables Checklist

Before deploying, ensure you have:

```bash
# Let's Encrypt
LETSENCRYPT_EMAIL=admin@visiquate.com

# Authentik
AUTHENTIK_BASE_URL=https://auth.visiquate.com
AUTHENTIK_OUTPOST_TOKEN=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...

# Release Storage
RELEASES_DIR=/data/releases
UPLOAD_API_KEY=$(openssl rand -base64 32)

# API
ENVIRONMENT=production
LOG_LEVEL=info
CCO_API_IMAGE=ghcr.io/visiquate/cco-releases-api:latest
```

## Troubleshooting Quick Links

| Issue | Solution |
|-------|----------|
| "Certificate generation failed" | Check DNS resolution: `dig cco-api.visiquate.com` |
| "Access denied" error | Verify token is valid: See Authentik endpoints in `/authentik/endpoints.md` |
| "Connection refused" | Check services running: `docker-compose ps` |
| "Storage volume not mounted" | Check volume exists: `docker volume ls \| grep cco-releases` |
| "Upload fails" | Verify UPLOAD_API_KEY is set correctly in .env file |
| "No outpost token" | Generate from Authentik admin: Administration > System > Outposts |

## Next: Releases API Implementation

The `releases-api` FastAPI service needs to be implemented. See:
- Plan: `/Users/brent/.claude/plans/wild-churning-trinket.md` (Phase 2)
- Location: `/Users/brent/git/cc-orchestra/releases-api/` (to be created)

## Detailed Documentation

For complete documentation, see:
- **R2 Storage**: `/Users/brent/git/cc-orchestra/infrastructure/r2/README.md`
- **Traefik Proxy**: `/Users/brent/git/cc-orchestra/infrastructure/traefik/README.md`
- **Authentik Auth**: `/Users/brent/git/cc-orchestra/infrastructure/authentik/README.md`
- **Plan**: `/Users/brent/.claude/plans/wild-churning-trinket.md`

## Cost Breakdown

| Component | Monthly Cost | Notes |
|-----------|--------------|-------|
| Local Storage | $0.00 | On existing infrastructure |
| Disk Space (50GB) | Included | Part of server cost |
| Traefik | $0.00 | Existing infra |
| Authentik | $0.00 | Existing infra |
| **TOTAL** | **$0.00/month** | Zero external dependencies! |

## Next Steps

1. Generate UPLOAD_API_KEY: `openssl rand -base64 32`
2. Gather Authentik outpost token
3. Create `.env` file from `.env.example` with your values
4. Run `docker-compose up -d` in traefik directory
5. Verify certificate generation with `curl -I https://cco-api.visiquate.com/`
6. Test authentication flow
7. Upload first release with `infrastructure/scripts/upload-release.sh`

## Support Resources

- Traefik Docs: https://doc.traefik.io/
- Authentik Docs: https://goauthentik.io/docs/
- Docker Volumes: https://docs.docker.com/storage/volumes/
- Docker Compose: https://docs.docker.com/compose/

---

**Status**: Infrastructure configuration complete. Ready for deployment.
**Next Phase**: Implement FastAPI Releases API service.
