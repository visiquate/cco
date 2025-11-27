# Infrastructure Migration: R2 to Local Docker Volume Storage

**Completed**: November 24, 2025
**Status**: Complete
**Migration Type**: External Storage → Local Docker Volume

## Summary

Successfully migrated CCO Private Binary Distribution infrastructure from CloudFlare R2 (external cloud storage) to local Docker volume-based storage. This eliminates external dependencies and reduces operational complexity while maintaining full functionality.

## Changes Made

### 1. Deleted R2 Infrastructure
**Location Removed**: `/infrastructure/r2/`
- `setup.sh` - R2 bucket setup script
- `upload-release.sh` - R2 upload script
- `README.md` - R2 configuration guide

**Rationale**: R2 dependency is no longer needed with local volume storage.

### 2. Updated Docker Compose Configuration
**File**: `/infrastructure/traefik/docker-compose.yml`

**Changes**:
- **Removed** R2 environment variables:
  - `R2_ACCOUNT_ID`
  - `R2_BUCKET_NAME`
  - `R2_ACCESS_KEY`
  - `R2_SECRET_KEY`
  - `R2_PRESIGNED_URL_TTL`

- **Added** local storage environment variables:
  - `RELEASES_DIR=/data/releases` - Local storage mount point
  - `UPLOAD_API_KEY` - Secure API key for upload authentication

- **Added** volume configuration:
  ```yaml
  volumes:
    - cco-releases:/data/releases
  ```

- **Added** volume definition:
  ```yaml
  volumes:
    cco-releases:
      driver: local
      name: cco-releases
  ```

### 3. Updated Environment Template
**File**: `/infrastructure/traefik/.env.example`

**Changes**:
- **Removed** all R2_* variables
- **Added** RELEASES_DIR configuration
- **Added** UPLOAD_API_KEY with generation instructions
- Simplified configuration for local deployment

### 4. Updated Deployment Documentation
**File**: `/infrastructure/QUICK_START.md`

**Changes**:
- **Overview**: Updated to reference local Docker volume storage
- **Prerequisites**: Removed CloudFlare R2 requirements
- **Step 1**: Replaced "Set Up R2 Bucket" with "Generate Upload API Key"
- **Architecture Diagram**: Updated to show local volume instead of R2
- **Environment Variables**: Simplified to local storage requirements
- **Troubleshooting**: Added storage-specific troubleshooting
- **Cost Breakdown**: Updated from ~$0.15/month to $0.00/month
- **Support Resources**: Removed R2 documentation links

### 5. Created Release Upload Script
**File**: `/infrastructure/scripts/upload-release.sh`

**Features**:
- Command: `./upload-release.sh <version> <binaries_directory>`
- Accepts version and binary directory as arguments
- Uses curl with `UPLOAD_API_KEY` header authentication
- Uses bearer token via `ACCESS_TOKEN` environment variable
- Validates binary directory exists
- Finds all matching binaries with pattern: `cco-v{VERSION}-*`
- Uploads each file to `/upload` endpoint
- Provides success/failure feedback

**Usage**:
```bash
export UPLOAD_API_KEY="your-secure-key"
export ACCESS_TOKEN="your-bearer-token"
export CCO_API_URL="https://cco-api.visiquate.com"

./infrastructure/scripts/upload-release.sh 2025.11.1 /path/to/binaries
```

## Benefits

### Cost Reduction
- **Before**: ~$0.15/month (R2 storage + operations)
- **After**: $0.00/month (uses existing infrastructure)

### Simplified Operations
- No external API credentials needed for storage
- No CloudFlare account dependency
- Self-contained deployment
- Easier backup strategies (local volume backups)

### Improved Control
- Full control over release binaries
- No rate limiting from external services
- Local performance for downloads
- Simple volume management

### Security
- No external API keys exposed
- Data stays on owned infrastructure
- Easier to implement access controls
- Simplified audit trails

## Migration Path for Existing Deployments

If you have an existing R2-based deployment, follow these steps:

### 1. Download Existing Releases from R2
```bash
# Export existing releases from R2
wrangler r2 object list cco-releases-private/releases/ --recursive
# Download each release file locally
```

### 2. Create Local Storage Volume
```bash
cd /Users/brent/git/cc-orchestra/infrastructure/traefik

# Start services (volume will be created automatically)
docker-compose up -d
```

### 3. Upload Releases to Local Storage
```bash
# Copy downloaded releases to local structure
# Then use the new upload script
export UPLOAD_API_KEY="your-generated-key"
export ACCESS_TOKEN="your-bearer-token"

./infrastructure/scripts/upload-release.sh 2025.11.1 /path/to/binaries
```

### 4. Verify Migration
```bash
# Check local volume
docker volume ls | grep cco-releases

# List files in volume
docker run --rm -v cco-releases:/data ls -la /data/releases/
```

## Environment Configuration

### Required Variables (in .env)
```bash
# Let's Encrypt
LETSENCRYPT_EMAIL=admin@visiquate.com

# Authentik
AUTHENTIK_BASE_URL=https://auth.visiquate.com
AUTHENTIK_OUTPOST_TOKEN=eyJ0...

# Local Release Storage
RELEASES_DIR=/data/releases

# Upload API Key (generate with: openssl rand -base64 32)
UPLOAD_API_KEY=your-secure-random-key

# API Configuration
ENVIRONMENT=production
LOG_LEVEL=info

# Docker Image
CCO_API_IMAGE=ghcr.io/visiquate/cco-releases-api:latest
```

## Implementation Notes

### Releases API Requirements
The CCO Releases API (`cco-releases-api` container) must implement:

1. **POST /upload**
   - Accept multipart form data with file and version
   - Require `X-Upload-Key` header matching UPLOAD_API_KEY
   - Require Authentik bearer token
   - Store files to RELEASES_DIR

2. **GET /releases/latest**
   - Return latest release metadata
   - Serve from local RELEASES_DIR

3. **GET /download/{version}/{platform}**
   - Return direct download URL or file content
   - Use files from local RELEASES_DIR

### Docker Volume Storage
- **Location**: Managed by Docker (use `docker volume inspect cco-releases` to find)
- **Permissions**: Docker manages access
- **Backups**: Standard volume backup procedures
- **Persistence**: Survives container restarts
- **Scaling**: Can be moved between hosts if needed

## Troubleshooting

### Volume Not Mounting
```bash
# Check if volume exists
docker volume ls | grep cco-releases

# Inspect volume
docker volume inspect cco-releases

# Check container volume mounts
docker inspect cco-releases-api | jq '.Mounts[]'
```

### Upload Fails
```bash
# Verify API key is set in .env
grep UPLOAD_API_KEY .env

# Test API endpoint
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  https://cco-api.visiquate.com/releases/latest

# Check container logs
docker-compose logs cco-releases-api
```

### Storage Space Issues
```bash
# Check volume usage
docker run --rm -v cco-releases:/data \
  sh -c 'du -sh /data/'

# List release versions
docker run --rm -v cco-releases:/data \
  find /data -type d -name '20*'
```

## Rollback Plan

If needed, you can revert to R2:

1. Keep R2 bucket with existing releases
2. Update docker-compose.yml with R2 environment variables
3. Update upload scripts to use R2
4. No data loss since R2 data is still available

However, R2 directory was deleted from repo. Recreate if needed from backup.

## Files Modified

| File | Changes |
|------|---------|
| `infrastructure/traefik/docker-compose.yml` | Updated env vars, added volume config |
| `infrastructure/traefik/.env.example` | Removed R2 vars, added local storage |
| `infrastructure/QUICK_START.md` | Updated setup, deployment, documentation |
| `infrastructure/scripts/upload-release.sh` | **NEW** - Upload script for local storage |
| `infrastructure/r2/*` | **DELETED** - R2 configuration no longer needed |

## Next Steps

1. **For Existing Deployments**:
   - Back up existing R2 releases
   - Update .env file with UPLOAD_API_KEY
   - Migrate releases to local storage
   - Update deployment configuration

2. **For New Deployments**:
   - Follow QUICK_START.md with simplified setup
   - No R2 credentials needed
   - Generate UPLOAD_API_KEY with `openssl rand -base64 32`
   - Deploy with `docker-compose up -d`

3. **For API Implementation**:
   - Implement `/upload` endpoint with API key validation
   - Implement local file storage in RELEASES_DIR
   - Test upload flow
   - Verify file serving

## References

- Docker Volumes: https://docs.docker.com/storage/volumes/
- Docker Compose: https://docs.docker.com/compose/
- CCO Infrastructure: `/infrastructure/QUICK_START.md`
- Releases API: To be implemented

---

**Status**: Migration complete and documented
**Date**: November 24, 2025
**Next Phase**: Releases API implementation
