# CCO Private Distribution - Deployment Guide

## Overview

This guide covers deploying the CCO private binary distribution system, which includes:
- **Authentication API** - OIDC device flow for CLI authentication
- **Releases API** - Secure release distribution with access control
- **Storage Backend** - Cloudflare R2 for binary storage
- **Access Control** - Authentik OIDC provider integration

## Architecture

```
┌──────────────┐
│  CCO Client  │ (Rust binary on user's machine)
└──────┬───────┘
       │ HTTPS
       ▼
┌─────────────────────────────────────────┐
│     cco-api.visiquate.com              │
│  ┌─────────────────────────────────┐   │
│  │  Authentication API              │   │
│  │  - POST /auth/device/code        │   │
│  │  - POST /auth/device/token       │   │
│  │  - POST /auth/token/refresh      │   │
│  ├─────────────────────────────────┤   │
│  │  Releases API                    │   │
│  │  - GET /releases/latest          │   │
│  │  - GET /releases/{version}       │   │
│  │  - GET /download/{version}/{plat}│   │
│  └─────────────────────────────────┘   │
└──────┬───────────────┬──────────────────┘
       │               │
       │ OIDC          │ Presigned URLs
       ▼               ▼
┌────────────────┐  ┌─────────────────┐
│   Authentik    │  │  Cloudflare R2  │
│ auth.visiquate │  │  Binary Storage │
│      .com      │  │                 │
└────────────────┘  └─────────────────┘
```

## Prerequisites

### Infrastructure Requirements

1. **Server/VM** (for cco-api.visiquate.com)
   - Linux (Ubuntu 22.04 LTS or similar)
   - 2+ CPU cores
   - 2GB+ RAM
   - 20GB+ disk space
   - Public IP with reverse proxy (nginx/caddy)

2. **Cloudflare R2** (Storage)
   - R2 bucket created: `cco-releases`
   - API token with R2 permissions
   - Custom domain (optional): `releases.visiquate.com`

3. **Authentik OIDC Provider**
   - Instance running at `auth.visiquate.com`
   - OIDC provider configured
   - Group for CCO users: `cco-users`
   - Group for admins: `cco-admins`

4. **Domain DNS**
   - `cco-api.visiquate.com` → Server IP
   - `releases.visiquate.com` → R2 custom domain (optional)

### Software Dependencies

```bash
# Server
- Python 3.10+
- PostgreSQL 14+ (or SQLite for development)
- nginx or Caddy (reverse proxy)
- systemd (process management)

# Development
- Rust 1.70+ (for building CCO binary)
- Git
```

## Step-by-Step Deployment

### 1. Server Setup

#### 1.1 Initial Server Configuration

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y python3.10 python3-pip python3-venv \
    postgresql postgresql-contrib nginx certbot python3-certbot-nginx \
    git curl

# Create service user
sudo useradd -r -m -s /bin/bash cco-api
sudo su - cco-api
```

#### 1.2 Create Application Directory

```bash
# As cco-api user
mkdir -p ~/cco-api/{app,logs,data}
cd ~/cco-api
```

### 2. Deploy Backend API

#### 2.1 Create Python Application

Create `/home/cco-api/cco-api/app/main.py`:

```python
from fastapi import FastAPI, HTTPException, Depends, Header
from pydantic import BaseModel
from typing import Optional, List
import httpx
import os
import boto3
from botocore.config import Config

app = FastAPI(title="CCO API", version="1.0.0")

# Environment configuration
AUTHENTIK_URL = os.getenv("AUTHENTIK_URL", "https://auth.visiquate.com")
AUTHENTIK_CLIENT_ID = os.getenv("AUTHENTIK_CLIENT_ID")
AUTHENTIK_CLIENT_SECRET = os.getenv("AUTHENTIK_CLIENT_SECRET")
R2_ACCOUNT_ID = os.getenv("R2_ACCOUNT_ID")
R2_ACCESS_KEY_ID = os.getenv("R2_ACCESS_KEY_ID")
R2_SECRET_ACCESS_KEY = os.getenv("R2_SECRET_ACCESS_KEY")
R2_BUCKET_NAME = os.getenv("R2_BUCKET_NAME", "cco-releases")

# Configure R2 client
s3_client = boto3.client(
    's3',
    endpoint_url=f'https://{R2_ACCOUNT_ID}.r2.cloudflarestorage.com',
    aws_access_key_id=R2_ACCESS_KEY_ID,
    aws_secret_access_key=R2_SECRET_ACCESS_KEY,
    config=Config(signature_version='s3v4'),
)

# Models
class DeviceCodeResponse(BaseModel):
    device_code: str
    user_code: str
    verification_uri: str
    expires_in: int
    interval: int

class TokenRequest(BaseModel):
    device_code: str

class TokenResponse(BaseModel):
    access_token: str
    refresh_token: str
    expires_in: int
    token_type: str

class PlatformAsset(BaseModel):
    platform: str
    filename: str
    size: int
    checksum: str

class ReleaseResponse(BaseModel):
    version: str
    release_notes: str
    platforms: List[PlatformAsset]

class DownloadUrlResponse(BaseModel):
    url: str
    expires_in: int

# Authentication dependency
async def verify_token(authorization: str = Header(...)):
    if not authorization.startswith("Bearer "):
        raise HTTPException(status_code=401, detail="Invalid authorization header")

    token = authorization.replace("Bearer ", "")

    # Verify token with Authentik
    async with httpx.AsyncClient() as client:
        response = await client.get(
            f"{AUTHENTIK_URL}/application/o/userinfo/",
            headers={"Authorization": f"Bearer {token}"}
        )

        if response.status_code != 200:
            raise HTTPException(status_code=401, detail="Invalid or expired token")

        user_info = response.json()

        # Check group membership
        if "cco-users" not in user_info.get("groups", []):
            raise HTTPException(status_code=403, detail="Access denied: Not in cco-users group")

        return user_info

# Authentication endpoints
@app.post("/auth/device/code", response_model=DeviceCodeResponse)
async def device_code():
    """Initiate OIDC device flow"""
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{AUTHENTIK_URL}/application/o/device/",
            data={
                "client_id": AUTHENTIK_CLIENT_ID,
                "scope": "openid profile email groups",
            }
        )

        if response.status_code != 200:
            raise HTTPException(status_code=500, detail="Failed to initiate device flow")

        data = response.json()
        return DeviceCodeResponse(**data)

@app.post("/auth/device/token", response_model=TokenResponse)
async def device_token(request: TokenRequest):
    """Poll for device flow completion"""
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{AUTHENTIK_URL}/application/o/token/",
            data={
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": request.device_code,
                "client_id": AUTHENTIK_CLIENT_ID,
            }
        )

        if response.status_code == 400:
            error = response.json()
            if error.get("error") == "authorization_pending":
                raise HTTPException(status_code=400, detail="authorization_pending")
            raise HTTPException(status_code=400, detail=error.get("error", "unknown_error"))

        if response.status_code != 200:
            raise HTTPException(status_code=500, detail="Token request failed")

        data = response.json()
        return TokenResponse(**data)

@app.post("/auth/token/refresh", response_model=TokenResponse)
async def refresh_token(refresh_token: str):
    """Refresh access token"""
    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{AUTHENTIK_URL}/application/o/token/",
            data={
                "grant_type": "refresh_token",
                "refresh_token": refresh_token,
                "client_id": AUTHENTIK_CLIENT_ID,
                "client_secret": AUTHENTIK_CLIENT_SECRET,
            }
        )

        if response.status_code != 200:
            raise HTTPException(status_code=401, detail="Invalid refresh token")

        data = response.json()
        return TokenResponse(**data)

# Releases endpoints
@app.get("/releases/latest", response_model=ReleaseResponse)
async def get_latest_release(
    channel: str = "stable",
    user_info: dict = Depends(verify_token)
):
    """Get latest release for a channel"""
    # Query releases from database or R2 metadata
    # For simplicity, returning mock data - replace with actual database query
    return ReleaseResponse(
        version="2025.11.2",
        release_notes="## What's New\n- Feature improvements\n- Bug fixes",
        platforms=[
            PlatformAsset(
                platform="darwin-arm64",
                filename="cco-v2025.11.2-darwin-arm64.tar.gz",
                size=45000000,
                checksum="sha256:abc123..."
            ),
            # Add other platforms...
        ]
    )

@app.get("/releases/{version}", response_model=ReleaseResponse)
async def get_release_by_version(
    version: str,
    user_info: dict = Depends(verify_token)
):
    """Get specific release version"""
    # Similar to latest, but for specific version
    pass

@app.get("/download/{version}/{platform}", response_model=DownloadUrlResponse)
async def get_download_url(
    version: str,
    platform: str,
    user_info: dict = Depends(verify_token)
):
    """Get presigned download URL for release"""
    # Generate presigned URL for R2 object
    object_key = f"v{version}/cco-v{version}-{platform}.tar.gz"

    try:
        presigned_url = s3_client.generate_presigned_url(
            'get_object',
            Params={
                'Bucket': R2_BUCKET_NAME,
                'Key': object_key
            },
            ExpiresIn=300  # 5 minutes
        )

        return DownloadUrlResponse(
            url=presigned_url,
            expires_in=300
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to generate download URL: {str(e)}")

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "cco-api",
        "version": "1.0.0"
    }
```

#### 2.2 Create Requirements File

Create `/home/cco-api/cco-api/app/requirements.txt`:

```
fastapi==0.104.1
uvicorn[standard]==0.24.0
httpx==0.25.0
boto3==1.29.0
pydantic==2.5.0
python-multipart==0.0.6
```

#### 2.3 Install Dependencies

```bash
# As cco-api user
cd ~/cco-api/app
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 3. Configure Environment Variables

Create `/home/cco-api/cco-api/.env`:

```bash
# Authentik Configuration
AUTHENTIK_URL=https://auth.visiquate.com
AUTHENTIK_CLIENT_ID=your-client-id-here
AUTHENTIK_CLIENT_SECRET=your-client-secret-here

# Cloudflare R2 Configuration
R2_ACCOUNT_ID=your-r2-account-id
R2_ACCESS_KEY_ID=your-r2-access-key
R2_SECRET_ACCESS_KEY=your-r2-secret-key
R2_BUCKET_NAME=cco-releases

# Application Configuration
HOST=127.0.0.1
PORT=8000
LOG_LEVEL=info
```

**Security Note**: Set proper file permissions:
```bash
chmod 600 /home/cco-api/cco-api/.env
```

### 4. Create Systemd Service

Create `/etc/systemd/system/cco-api.service`:

```ini
[Unit]
Description=CCO API Service
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=cco-api
Group=cco-api
WorkingDirectory=/home/cco-api/cco-api/app
EnvironmentFile=/home/cco-api/cco-api/.env
ExecStart=/home/cco-api/cco-api/app/venv/bin/uvicorn main:app --host ${HOST} --port ${PORT}
Restart=always
RestartSec=10
StandardOutput=append:/home/cco-api/cco-api/logs/api.log
StandardError=append:/home/cco-api/cco-api/logs/api-error.log

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable cco-api
sudo systemctl start cco-api
sudo systemctl status cco-api
```

### 5. Configure Nginx Reverse Proxy

Create `/etc/nginx/sites-available/cco-api`:

```nginx
server {
    listen 80;
    server_name cco-api.visiquate.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name cco-api.visiquate.com;

    # SSL configuration (certbot will modify this)
    ssl_certificate /etc/letsencrypt/live/cco-api.visiquate.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/cco-api.visiquate.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Proxy to FastAPI
    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check
    location /health {
        access_log off;
        proxy_pass http://127.0.0.1:8000/health;
    }
}
```

Enable site and obtain SSL certificate:

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/cco-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Obtain SSL certificate
sudo certbot --nginx -d cco-api.visiquate.com
```

### 6. Configure Authentik OIDC Provider

#### 6.1 Create OIDC Application

1. Log into Authentik admin panel
2. Navigate to **Applications** → **Providers**
3. Create new **OAuth2/OIDC Provider**:
   - Name: `CCO CLI`
   - Client Type: `Public`
   - Client ID: (note this down)
   - Redirect URIs: `urn:ietf:wg:oauth:2.0:oob`
   - Scopes: `openid`, `profile`, `email`, `groups`
   - Token validity: `1 hour`
   - Refresh token validity: `30 days`

#### 6.2 Create Groups

1. Navigate to **Directory** → **Groups**
2. Create groups:
   - `cco-users` - Users with access to CCO binaries
   - `cco-admins` - Administrators with full access

#### 6.3 Assign Users to Groups

Add authorized users to `cco-users` group.

### 7. Configure Cloudflare R2

#### 7.1 Create R2 Bucket

```bash
# Using Wrangler CLI
npx wrangler r2 bucket create cco-releases

# Or via Cloudflare dashboard:
# 1. Navigate to R2
# 2. Create bucket: cco-releases
# 3. Configure CORS (optional)
```

#### 7.2 Create API Token

1. Go to Cloudflare dashboard → R2
2. Create API token with permissions:
   - `R2 Read` on `cco-releases`
   - `R2 Write` on `cco-releases`
3. Note down:
   - Account ID
   - Access Key ID
   - Secret Access Key

#### 7.3 Upload First Release

```bash
# Build CCO binary
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# Create release archive
VERSION="2025.11.1"
PLATFORM="darwin-arm64"
tar -czf cco-v${VERSION}-${PLATFORM}.tar.gz -C target/release cco

# Generate checksum
sha256sum cco-v${VERSION}-${PLATFORM}.tar.gz > checksums.txt

# Upload to R2
aws s3 cp \
  --endpoint-url https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  cco-v${VERSION}-${PLATFORM}.tar.gz \
  s3://cco-releases/v${VERSION}/

aws s3 cp \
  --endpoint-url https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  checksums.txt \
  s3://cco-releases/v${VERSION}/
```

### 8. Health Check Verification

```bash
# Test API health
curl https://cco-api.visiquate.com/health

# Expected response:
# {"status":"healthy","service":"cco-api","version":"1.0.0"}

# Test device flow initiation
curl -X POST https://cco-api.visiquate.com/auth/device/code

# Expected: device_code, user_code, verification_uri
```

## Environment Variables Reference

| Variable | Description | Required | Example |
|----------|-------------|----------|---------|
| `AUTHENTIK_URL` | Authentik instance URL | Yes | `https://auth.visiquate.com` |
| `AUTHENTIK_CLIENT_ID` | OIDC client ID | Yes | `abc123...` |
| `AUTHENTIK_CLIENT_SECRET` | OIDC client secret | Yes | `secret123...` |
| `R2_ACCOUNT_ID` | Cloudflare R2 account ID | Yes | `abc123...` |
| `R2_ACCESS_KEY_ID` | R2 API access key | Yes | `AKIAIOSFODNN7...` |
| `R2_SECRET_ACCESS_KEY` | R2 API secret key | Yes | `wJalrXUtnFEMI...` |
| `R2_BUCKET_NAME` | R2 bucket name | Yes | `cco-releases` |
| `HOST` | API bind host | No | `127.0.0.1` |
| `PORT` | API bind port | No | `8000` |
| `LOG_LEVEL` | Logging level | No | `info` |

## Secrets Management

### Development

Store secrets in `.env` file with `chmod 600` permissions.

### Production

Use one of these methods:

1. **Systemd EnvironmentFile** (current method)
   ```bash
   sudo chmod 600 /home/cco-api/cco-api/.env
   sudo chown cco-api:cco-api /home/cco-api/cco-api/.env
   ```

2. **HashiCorp Vault**
   ```bash
   vault kv put secret/cco-api \
     authentik_client_id=... \
     authentik_client_secret=...
   ```

3. **AWS Secrets Manager** (if on AWS)
   ```bash
   aws secretsmanager create-secret \
     --name cco-api/config \
     --secret-string file://secrets.json
   ```

## Troubleshooting

### API Not Responding

```bash
# Check service status
sudo systemctl status cco-api

# View logs
sudo journalctl -u cco-api -f

# Check if port is listening
sudo netstat -tlnp | grep 8000

# Test direct API access
curl http://127.0.0.1:8000/health
```

### SSL Certificate Issues

```bash
# Test SSL
openssl s_client -connect cco-api.visiquate.com:443

# Renew certificates
sudo certbot renew --dry-run
sudo certbot renew
```

### Authentik Connection Errors

```bash
# Test Authentik connectivity
curl https://auth.visiquate.com/.well-known/openid-configuration

# Verify client ID and secret
# Check Authentik logs
```

### R2 Access Issues

```bash
# Test R2 connectivity
aws s3 ls \
  --endpoint-url https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  s3://cco-releases/

# Verify credentials
echo $R2_ACCESS_KEY_ID
```

### Database Issues (if using PostgreSQL)

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Connect to database
sudo -u postgres psql

# View connections
SELECT * FROM pg_stat_activity;
```

## Monitoring

### Basic Monitoring

```bash
# Monitor API logs
tail -f /home/cco-api/cco-api/logs/api.log

# Monitor nginx access
sudo tail -f /var/log/nginx/access.log

# System resource usage
htop
```

### Production Monitoring

Consider implementing:

1. **Application Monitoring**
   - Prometheus metrics endpoint
   - Grafana dashboards
   - Alert rules for errors

2. **Infrastructure Monitoring**
   - Server metrics (CPU, RAM, disk)
   - Network metrics
   - SSL certificate expiration

3. **Audit Logging**
   - All authentication events
   - All download events
   - Failed access attempts

## Security Checklist

- [ ] SSL/TLS enabled with valid certificates
- [ ] Environment variables secured (chmod 600)
- [ ] Firewall configured (only 80/443 open)
- [ ] SSH key-only authentication
- [ ] Regular security updates scheduled
- [ ] Audit logging enabled
- [ ] Backup strategy in place
- [ ] Intrusion detection configured
- [ ] Rate limiting enabled
- [ ] CORS properly configured

## Backup and Recovery

### Database Backup

```bash
# Backup PostgreSQL database
pg_dump -U cco_api cco_db > backup-$(date +%Y%m%d).sql

# Restore
psql -U cco_api cco_db < backup-20251124.sql
```

### R2 Backup

```bash
# Sync R2 bucket to local storage
aws s3 sync \
  --endpoint-url https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  s3://cco-releases/ \
  /backup/cco-releases/
```

### Configuration Backup

```bash
# Backup configuration files
tar -czf cco-api-config-$(date +%Y%m%d).tar.gz \
  /home/cco-api/cco-api/.env \
  /etc/systemd/system/cco-api.service \
  /etc/nginx/sites-available/cco-api
```

## Rollback Plan

If deployment fails:

1. **Stop new service**
   ```bash
   sudo systemctl stop cco-api
   ```

2. **Restore previous version**
   ```bash
   cd /home/cco-api/cco-api/app
   git checkout previous-tag
   source venv/bin/activate
   pip install -r requirements.txt
   ```

3. **Restart service**
   ```bash
   sudo systemctl start cco-api
   ```

4. **Verify health**
   ```bash
   curl https://cco-api.visiquate.com/health
   ```

## Next Steps

After successful deployment:

1. **Test authentication flow** - See [USER_GUIDE_AUTHENTICATION.md](./USER_GUIDE_AUTHENTICATION.md)
2. **Configure user access** - See [ADMIN_GUIDE_ACCESS_CONTROL.md](./ADMIN_GUIDE_ACCESS_CONTROL.md)
3. **Upload releases** - See release upload scripts
4. **Monitor system** - Set up dashboards and alerts
5. **Document procedures** - Create runbooks for common operations

## Support

For deployment issues:
- Check logs: `/home/cco-api/cco-api/logs/`
- Review systemd status: `sudo systemctl status cco-api`
- Test components individually (API, Authentik, R2)
- Consult [TROUBLESHOOTING.md](../cco/TROUBLESHOOTING.md)
