# Configuration Reference - CCO Releases API

## Environment Variables

### Required

| Variable | Type | Example | Description |
|----------|------|---------|-------------|
| `R2_ACCOUNT_ID` | string | `abc123def456` | CloudFlare R2 Account ID |
| `R2_ACCESS_KEY` | string | `xxxxxxxxxxxx` | R2 API Access Key |
| `R2_SECRET_KEY` | string | `yyyyyyyyyyyy` | R2 API Secret Key |

### Optional

| Variable | Default | Example | Description |
|----------|---------|---------|-------------|
| `R2_BUCKET` | `cco-releases-private` | `cco-releases-private` | R2 bucket name |
| `VERSION` | `unknown` | `2025.11.24` | Application version |
| `ENVIRONMENT` | `production` | `development` | Environment mode |

## Environment Variable Details

### R2_ACCOUNT_ID

**Where to find**: CloudFlare Dashboard → R2 → Settings → Account Details

**Format**: 32-character hexadecimal string

**Example**:
```
a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

**How to get**:
1. Log in to CloudFlare dashboard
2. Navigate to R2 section
3. Click "Settings" in the left sidebar
4. Find "Account Details" section
5. Copy "Account ID"

### R2_ACCESS_KEY

**Where to find**: CloudFlare Dashboard → R2 → Settings → API Tokens

**Format**: Random alphanumeric string (~32 characters)

**Example**:
```
1234567890abcdefghijklmnopqrstuv
```

**How to get**:
1. Go to CloudFlare Dashboard → R2 → Settings → API Tokens
2. Click "Create API token"
3. Choose "R2 Restricted"
4. Set permissions:
   - Permissions: `s3:GetObject`, `s3:ListBucket`, `s3:GetBucketLocation`
   - Bucket: Select `cco-releases-private`
   - TTL: Never (or your preference)
5. Copy "Access Key ID"

### R2_SECRET_KEY

**Where to find**: CloudFlare Dashboard → R2 → Settings → API Tokens (shown once)

**Format**: Random alphanumeric string (~64 characters)

**Important**: This is shown only ONCE when created. Store it immediately.

**Example**:
```
abcdefghijklmnopqrstuvwxyz123456ABCDEFGHIJKLMNOPQRSTUVWXYZ7890
```

**How to get**:
1. From API token creation (same process as Access Key)
2. Copy "Secret Access Key" immediately (won't be shown again)
3. Store securely

### R2_BUCKET

**Type**: `string`
**Default**: `cco-releases-private`
**Purpose**: S3 bucket name (must exist in R2)

**Typical values**:
```
cco-releases-private       # Production
cco-releases-staging       # Staging
cco-releases-test          # Testing
```

**How to verify**:
```bash
aws s3 ls s3://cco-releases-private/ \
  --endpoint-url https://{ACCOUNT_ID}.r2.cloudflarestorage.com
```

### VERSION

**Type**: `string`
**Default**: `unknown`
**Purpose**: Application version string (for logging and health checks)

**Format**: YYYY.MM.N (date-based versioning)

**Examples**:
```
2025.11.24     # Release from Nov 24, 2025
2025.11.1      # First release in November 2025
2025.11.15     # Fifteenth release in November 2025
```

**Exposed via**:
```bash
# Health check endpoint
curl https://cco-api.visiquate.com/health

# Response includes:
# {"version": "2025.11.24", ...}
```

### ENVIRONMENT

**Type**: `string`
**Default**: `production`
**Purpose**: Controls application behavior (logging, debug mode, etc.)

**Valid values**:
- `production` - Production behavior (minimal logging)
- `development` - Development behavior (verbose logging, hot-reload)
- `staging` - Staging/test environment

**Behavior**:
```python
# In main.py
if ENVIRONMENT == "development":
    # Enable hot-reload
    uvicorn.run(..., reload=True)
else:
    # Production (no reload)
    uvicorn.run(..., reload=False)
```

## Configuration Methods

### 1. Environment Variables (Simplest)

```bash
export R2_ACCOUNT_ID=abc123...
export R2_ACCESS_KEY=xxxx...
export R2_SECRET_KEY=yyyy...

python -m uvicorn main:app --host 0.0.0.0 --port 8000
```

### 2. .env File (Local Development)

Create `.env`:
```bash
R2_ACCOUNT_ID=abc123...
R2_ACCESS_KEY=xxxx...
R2_SECRET_KEY=yyyy...
VERSION=2025.11.24
ENVIRONMENT=development
```

Load automatically:
```bash
docker compose up
```

### 3. Docker Environment (Recommended)

```bash
docker run -e R2_ACCOUNT_ID=$R2_ACCOUNT_ID \
           -e R2_ACCESS_KEY=$R2_ACCESS_KEY \
           -e R2_SECRET_KEY=$R2_SECRET_KEY \
           cco-releases-api:latest
```

### 4. Docker Compose

```yaml
services:
  releases-api:
    environment:
      R2_ACCOUNT_ID: ${R2_ACCOUNT_ID}
      R2_ACCESS_KEY: ${R2_ACCESS_KEY}
      R2_SECRET_KEY: ${R2_SECRET_KEY}
      VERSION: 2025.11.24
```

### 5. Kubernetes Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: releases-api-secrets
stringData:
  R2_ACCOUNT_ID: abc123...
  R2_ACCESS_KEY: xxxx...
  R2_SECRET_KEY: yyyy...

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco-releases-api
spec:
  template:
    spec:
      containers:
      - name: releases-api
        envFrom:
        - secretRef:
            name: releases-api-secrets
```

## Configuration Validation

### Check Configuration at Startup

```bash
# The application validates all required variables on startup
# If missing, you'll see:
# RuntimeError: Missing required R2 configuration: R2_ACCOUNT_ID, R2_ACCESS_KEY, R2_SECRET_KEY
```

### Verify R2 Connection

```bash
# Health check confirms R2 connectivity
curl https://cco-api.visiquate.com/health

# Response when configured correctly:
{
  "status": "healthy",
  "service": "cco-releases-api",
  "version": "2025.11.24",
  "r2_connected": true,
  "timestamp": "2025-11-24T10:30:00Z"
}

# If R2 fails:
{
  "status": "degraded",
  "r2_connected": false,
  ...
}
```

## Configuration Best Practices

### 1. Never Commit Secrets

```bash
# ✓ DO
echo ".env" >> .gitignore
cp .env.example .env  # (edit locally)

# ✗ DON'T
git add .env
git commit -m "Add R2 credentials"  # NEVER!
```

### 2. Rotate Credentials Regularly

**Schedule**: Every 90 days

**Process**:
1. Generate new API token in CloudFlare
2. Update environment variable
3. Redeploy service
4. Revoke old token

### 3. Use Minimal Permissions

**Recommended permissions**:
```
- s3:GetObject        (read files)
- s3:ListBucket       (list contents)
- s3:GetBucketLocation (health checks)
```

**Never grant**:
```
- s3:DeleteObject     (accidental deletion)
- s3:PutObject        (unauthorized writes)
- s3:ListAllBuckets   (information disclosure)
```

### 4. Monitor Access

**CloudFlare R2 Access Logs**:
```bash
# View in CloudFlare dashboard
# R2 → Settings → Request logging
```

**Application Logs**:
```bash
# See all API requests
docker compose logs releases-api
```

## Configuration Examples

### Local Development with MinIO

```bash
# .env.local
R2_ACCOUNT_ID=localhost
R2_ACCESS_KEY=minioadmin
R2_SECRET_KEY=minioadmin
R2_BUCKET=cco-releases-private
VERSION=dev
ENVIRONMENT=development

# Start MinIO
docker compose --profile dev up
```

### Production with CloudFlare

```bash
# .env.prod
R2_ACCOUNT_ID=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
R2_ACCESS_KEY=1234567890abcdefghijklmnopqrstuv
R2_SECRET_KEY=abcdefghijklmnopqrstuvwxyz123456ABCDEFGHIJKLMNOPQRSTUVWXYZ7890
R2_BUCKET=cco-releases-private
VERSION=2025.11.24
ENVIRONMENT=production

# Deploy
docker compose -f docker-compose.prod.yml up -d
```

### Kubernetes Secret

```bash
# Create secret
kubectl create secret generic releases-api-secrets \
  --from-literal=R2_ACCOUNT_ID=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6 \
  --from-literal=R2_ACCESS_KEY=1234567890abcdefghijklmnopqrstuv \
  --from-literal=R2_SECRET_KEY=abcdefghijklmnopqrstuvwxyz123456ABCDEFGHIJKLMNOPQRSTUVWXYZ7890 \
  -n cco

# Deploy
kubectl apply -f k8s-deployment.yml
```

## Troubleshooting Configuration

### "Missing required R2 configuration"

```bash
# Check environment variables
echo $R2_ACCOUNT_ID
echo $R2_ACCESS_KEY
echo $R2_SECRET_KEY

# If empty, set them
export R2_ACCOUNT_ID=...
export R2_ACCESS_KEY=...
export R2_SECRET_KEY=...

# Or use .env file
# docker compose up (will auto-load .env)
```

### "R2 connection failed"

```bash
# Verify credentials are correct
aws s3 ls s3://cco-releases-private/ \
  --endpoint-url https://{R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  --access-key-id $R2_ACCESS_KEY \
  --secret-access-key $R2_SECRET_KEY

# Check health endpoint
curl https://cco-api.visiquate.com/health

# If degraded, check logs
docker compose logs releases-api
```

### "Metadata not found"

```bash
# Verify bucket path structure
aws s3 ls s3://cco-releases-private/ \
  --endpoint-url https://{R2_ACCOUNT_ID}.r2.cloudflarestorage.com \
  --recursive

# Upload test metadata
aws s3 cp metadata/latest-stable.json \
  s3://cco-releases-private/metadata/latest-stable.json \
  --endpoint-url https://{R2_ACCOUNT_ID}.r2.cloudflarestorage.com
```

## Related Documentation

- **README.md** - API endpoint documentation
- **SECURITY.md** - Credential security best practices
- **DEPLOYMENT.md** - Production deployment guide
- **QUICKSTART.md** - Get started in 5 minutes

---

**Last Updated**: 2025-11-24
**Version**: 1.0
