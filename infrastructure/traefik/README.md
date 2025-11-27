# Traefik Reverse Proxy for CCO Releases API

This directory contains Traefik configuration for routing authenticated requests to the CCO Releases API service with Authentik forward authentication.

## Architecture

```
Internet
    │
    ├─ HTTP (port 80) ──────┐
    │                        ├──> Traefik ──> Authentik Forward Auth ──> cco-releases-api (port 8000)
    └─ HTTPS (port 443) ────┘                      │
                                                   ├─ Valid token?
                                                   │  ├─ YES: Forward to API
                                                   │  └─ NO: Redirect to Authentik
```

### Components

| Component | Role | Port | Notes |
|-----------|------|------|-------|
| **Traefik** | Reverse proxy, TLS termination | 80, 443 | Handles HTTPS, forwards auth |
| **Authentik Outpost** | Forward auth middleware | 4180 (internal) | Validates tokens from Authentik |
| **Releases API** | Backend service | 8000 (internal) | FastAPI service with R2 integration |
| **Let's Encrypt** | TLS certificates | N/A | Auto-renews cco-api.visiquate.com cert |

## Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Docker Compose configuration for Traefik + services |
| `traefik.yml` | Traefik static configuration (global settings) |
| `dynamic/cco-api.yml` | Traefik dynamic configuration (routing rules) |
| `README.md` | This file |
| `.env.example` | Example environment variables |

## Prerequisites

### 1. Authentik Configuration
Authentik must already be running with:
- **OAuth2 Provider**: `CCO CLI Provider` with client ID `cco-cli`
- **Forward Auth**: Enabled and working (test with other Traefik routes)
- **Outpost Token**: Generated and available

See `/Users/brent/git/cc-orchestra/infrastructure/authentik/` for setup.

### 2. R2 Configuration
CloudFlare R2 bucket must be created and populated:
- Bucket name: `cco-releases-private`
- Structure: `releases/{version}/{binary}` + `metadata/latest.json`

See `/Users/brent/git/cc-orchestra/infrastructure/r2/` for setup.

### 3. DNS Configuration
Point domain to the Traefik host:
```
cco-api.visiquate.com  A  <your-ip>
```

### 4. Docker Installation
- Docker Engine 20.10+
- Docker Compose 2.0+

## Environment Variables

Create `.env` file in this directory:

```bash
# Let's Encrypt
LETSENCRYPT_EMAIL=admin@visiquate.com

# Authentik Configuration
AUTHENTIK_BASE_URL=https://auth.visiquate.com
AUTHENTIK_OUTPOST_TOKEN=eyJ0eXAi...  # Generate from Authentik admin UI

# R2 Configuration
R2_ACCOUNT_ID=abc123xyz789  # CloudFlare account ID
R2_BUCKET_NAME=cco-releases-private
R2_ACCESS_KEY=v1.abc123...  # R2 API token
R2_SECRET_KEY=secret...     # R2 API secret

# API Configuration
ENVIRONMENT=production
LOG_LEVEL=info
CCO_API_IMAGE=ghcr.io/visiquate/cco-releases-api:latest
```

### Getting Authentik Outpost Token

1. Login to Authentik admin at `https://auth.visiquate.com/admin/`
2. Navigate to **Administration** > **System** > **Outposts**
3. Click on the CCO outpost (or create new if not exists)
4. Copy the **Service Token** to `AUTHENTIK_OUTPOST_TOKEN`

## Deployment

### 1. Configure Environment

```bash
cd /Users/brent/git/cc-orchestra/infrastructure/traefik

# Create .env with your configuration
cp .env.example .env
nano .env  # Edit with your settings
```

### 2. Start Services

```bash
# Pull images
docker-compose pull

# Start in background
docker-compose up -d

# Check status
docker-compose ps
```

**Expected output:**
```
NAME                        STATUS              PORTS
traefik-cco-api            Up (healthy)        0.0.0.0:80->80/tcp, 0.0.0.0:443->443/tcp
authentik-outpost-cco      Up (healthy)        4180/tcp
cco-releases-api           Up (healthy)
watchtower-cco             Up
autoheal-cco               Up
```

### 3. Verify TLS Certificate

```bash
# Check certificate details
docker exec traefik-cco-api cat /letsencrypt/acme.json | jq .

# Test HTTPS connection
curl -I https://cco-api.visiquate.com/

# Should get: HTTP/1.1 307 Temporary Redirect (redirecting to Authentik)
```

### 4. Test Authentication

```bash
# Without token: should redirect to Authentik
curl -L https://cco-api.visiquate.com/releases/latest

# With token: should return release metadata
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  https://cco-api.visiquate.com/releases/latest
```

## Monitoring

### View Logs

```bash
# Traefik logs
docker-compose logs -f traefik

# API logs
docker-compose logs -f cco-releases-api

# Authentik outpost logs
docker-compose logs -f authentik-outpost

# All services
docker-compose logs -f
```

### Health Checks

```bash
# Check Traefik health
curl https://traefik.visiquate.com/ping -k

# Check API health
docker exec cco-releases-api curl http://localhost:8000/health

# Check Authentik outpost
docker exec authentik-outpost-cco curl http://localhost:4180/
```

### Dashboard

Access Traefik dashboard (requires separate authentication setup):

```bash
# Dashboard at https://traefik.visiquate.com/dashboard/
# (Must configure dashboard authentication separately)
```

## Troubleshooting

### Certificate Generation Failed

**Error**: "acme: error: dns :: 400 :: urn:acme:errorType:dns :: DNS problem: NXDOMAIN looking up cco-api.visiquate.com"

**Solutions**:
1. Verify DNS record exists and resolves:
   ```bash
   nslookup cco-api.visiquate.com
   dig cco-api.visiquate.com
   ```
2. Check port 80 is open from the internet
3. Wait 30+ seconds after DNS change before testing
4. Check Let's Encrypt rate limits haven't been exceeded

### Authentication Always Redirects to Authentik

**Symptoms**: Even with valid token, redirects to login page.

**Debug steps**:
1. Check Authentik outpost is running:
   ```bash
   docker-compose ps authentik-outpost
   ```
2. Verify outpost token is correct:
   ```bash
   docker-compose logs authentik-outpost | grep -i error
   ```
3. Check Traefik can reach outpost:
   ```bash
   docker exec traefik-cco-api curl http://authentik-outpost:4180/
   ```
4. Verify token is valid:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" \
     https://auth.visiquate.com/application/o/userinfo/
   ```

### API Returns 500 Error

**Symptoms**: 500 Internal Server Error from releases API.

**Debug steps**:
1. Check API logs:
   ```bash
   docker-compose logs cco-releases-api | tail -50
   ```
2. Verify R2 credentials:
   ```bash
   docker exec cco-releases-api env | grep R2_
   ```
3. Check R2 bucket exists and is accessible:
   ```bash
   wrangler r2 bucket list
   ```
4. Verify metadata file exists:
   ```bash
   wrangler r2 object get cco-releases-private metadata/latest.json
   ```

### Port 80 or 443 Already in Use

**Error**: "Address already in use" when starting Traefik.

**Solutions**:
1. Find what's using the port:
   ```bash
   lsof -i :80
   lsof -i :443
   ```
2. Stop the service or change ports in `docker-compose.yml`
3. Or use a different host to run Traefik

### High Memory Usage

**Symptoms**: Docker containers consuming excessive memory.

**Solutions**:
1. Restart services:
   ```bash
   docker-compose restart
   ```
2. Check for stuck processes:
   ```bash
   docker-compose logs cco-releases-api | tail -100
   ```
3. Clear old certificate renewals:
   ```bash
   docker exec traefik-cco-api ls -lah /letsencrypt/
   ```

## Testing

### 1. Test Without Authentication (Should Fail)

```bash
# Try to access API without token
curl -v https://cco-api.visiquate.com/releases/latest 2>&1 | grep -E "HTTP|Location"

# Expected: 307 Temporary Redirect to Authentik login
```

### 2. Get Authenticated Token

```bash
# Use CCO CLI to login (if implemented)
cco login

# Or obtain token from Authentik manually:
curl -X POST https://auth.visiquate.com/application/o/token/ \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=refresh_token&refresh_token=YOUR_REFRESH_TOKEN&client_id=cco-cli"
```

### 3. Test With Authentication (Should Work)

```bash
# Request with token
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  https://cco-api.visiquate.com/releases/latest

# Expected: 200 OK with JSON response
# {
#   "version": "2025.11.1",
#   "name": "CCO",
#   ...
# }
```

### 4. Test Download Endpoint

```bash
# Get presigned URL for darwin-arm64 binary
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  https://cco-api.visiquate.com/download/2025.11.1/darwin-arm64

# Expected: 200 OK with presigned R2 URL
# {
#   "download_url": "https://abc123.r2.cloudflarestorage.com/...",
#   "expires_in": 900
# }

# Download from presigned URL
curl -L "$(curl -s -H 'Authorization: Bearer $ACCESS_TOKEN' https://cco-api.visiquate.com/download/2025.11.1/darwin-arm64 | jq -r '.download_url')" -o cco-binary.tar.gz
```

### 5. Test Root Path (Should Return 404)

```bash
# Request root path
curl -v https://cco-api.visiquate.com/ 2>&1 | grep -E "HTTP|error"

# Expected: 404 Not Found with error JSON
# {
#   "error": "not_found",
#   "message": "This endpoint is not publicly available"
# }
```

## Advanced Configuration

### Custom Rate Limiting

Edit `dynamic/cco-api.yml` middleware section:

```yaml
cco-rate-limit:
  rateLimit:
    average: 50    # Reduce from 100
    burst: 100     # Reduce from 200
    period: 60
```

### Multiple Traefik Instances (High Availability)

For production HA setup:
1. Deploy Traefik on multiple hosts
2. Use shared certificate storage (AWS S3, etc.)
3. Load balance between Traefik instances

### Custom Headers

Add headers to API responses in `dynamic/cco-api.yml`:

```yaml
cco-headers:
  headers:
    customResponseHeaders:
      X-Custom-Header: "Value"
      X-API-Version: "1.0"
```

## Maintenance

### Update Let's Encrypt Certificate

Certificates auto-renew, but to manually trigger:

```bash
# Remove cached certificate
docker exec traefik-cco-api rm -f /letsencrypt/acme.json

# Restart Traefik (will regenerate)
docker-compose restart traefik
```

### Update Traefik Version

```bash
# Edit docker-compose.yml image tag
# From: image: traefik:v3.0
# To:   image: traefik:v3.1

docker-compose pull traefik
docker-compose up -d traefik
```

### Backup Configuration

```bash
# Backup Traefik configuration
tar czf traefik-backup-$(date +%Y%m%d).tar.gz \
  docker-compose.yml traefik.yml dynamic/ .env

# Backup certificates
docker cp traefik-cco-api:/letsencrypt/acme.json ./acme.json.backup
```

## Security Considerations

### HTTPS Only
- All traffic is redirected from HTTP to HTTPS
- Let's Encrypt certificate auto-renewed

### Authentication
- All requests validated by Authentik forward auth
- Unauthenticated requests redirected to login
- Group-based access control (cco-users group)

### Rate Limiting
- Maximum 100 requests/second per client
- Burst limit of 200 requests
- Prevents abuse and DoS attacks

### Headers
- Security headers included by default
- XSS protection enabled
- Framing disabled

### Root Path Protection
- Root path (`/`) returns 404 with error message
- Prevents discovery of the API endpoint
- No robots.txt or sitemap served

## Integration with CI/CD

For GitHub Actions deployment:

```yaml
# .github/workflows/deploy-releases-api.yml
- name: Deploy Releases API
  env:
    SSH_KEY: ${{ secrets.DEPLOY_SSH_KEY }}
  run: |
    ssh -i $SSH_KEY deploy@prod.visiquate.com << 'EOF'
      cd /opt/cco/infrastructure/traefik
      docker-compose pull
      docker-compose up -d
      docker-compose logs -f --tail=20
    EOF
```

## References

- [Traefik Documentation](https://doc.traefik.io/)
- [Authentik Forward Auth](https://goauthentik.io/docs/providers/proxy/)
- [Let's Encrypt](https://letsencrypt.org/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)

## Support

For issues:
1. Check logs: `docker-compose logs -f`
2. Verify DNS resolution: `dig cco-api.visiquate.com`
3. Test connectivity: `curl -v https://cco-api.visiquate.com/`
4. Check Authentik status: Visit https://auth.visiquate.com/admin/
5. Review Traefik dashboard: https://traefik.visiquate.com/dashboard/ (if auth configured)
