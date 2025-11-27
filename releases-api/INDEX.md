# CCO Releases API - Complete Documentation Index

## Overview

The CCO Releases API is a FastAPI service that provides authenticated access to CCO binary distributions through CloudFlare R2 storage. It's designed to be lightweight, secure, and easy to deploy.

**Location**: `/Users/brent/git/cc-orchestra/releases-api/`

## Getting Started

### For First-Time Users

1. **QUICKSTART.md** - Get running in 5 minutes
2. **README.md** - Full API documentation and examples
3. **CONFIG_REFERENCE.md** - Environment configuration details

### For Developers

1. **README.md** - Architecture and project structure
2. **SECURITY.md** - Credential handling best practices
3. **test_main.py** - Test suite with examples

### For DevOps/Operations

1. **DEPLOYMENT.md** - Production deployment guide
2. **SECURITY.md** - Security considerations
3. **CONFIG_REFERENCE.md** - Environment configuration

## Documentation Files

### Core Documentation

| File | Purpose | Audience | Read Time |
|------|---------|----------|-----------|
| **QUICKSTART.md** | Get started in 5 minutes | Everyone | 5 min |
| **README.md** | Complete API documentation | Developers, DevOps | 15 min |
| **SECURITY.md** | Security best practices | DevOps, Security | 10 min |
| **DEPLOYMENT.md** | Production deployment guide | DevOps, Ops | 20 min |
| **CONFIG_REFERENCE.md** | Configuration details | Developers, DevOps | 15 min |
| **INDEX.md** | This file - documentation index | Everyone | 5 min |

### Code Files

| File | Purpose | Lines | Type |
|------|---------|-------|------|
| **main.py** | FastAPI application | ~250 | Python |
| **test_main.py** | Unit tests | ~300 | Python |
| **Dockerfile** | Container image definition | ~50 | Docker |
| **docker-compose.yml** | Local development setup | ~50 | YAML |
| **Makefile** | Common development tasks | ~60 | Makefile |

### Configuration Files

| File | Purpose | Notes |
|------|---------|-------|
| **.env.example** | Environment template | Copy to .env and edit |
| **.gitignore** | Git ignore rules | Prevents committing secrets |
| **requirements.txt** | Production dependencies | 6 packages |
| **requirements-dev.txt** | Development dependencies | Testing, linting, formatting |

### GitHub Actions

| File | Purpose | Trigger |
|------|---------|---------|
| **.github/workflows/test-and-build.yml** | CI/CD pipeline | Push to main, PRs |

## Key Features

### API Endpoints

```
GET /health                          Health check with R2 connectivity
GET /releases/latest?channel=stable  Get latest release metadata
GET /releases/{version}              Get specific version metadata
GET /download/{version}/{platform}   Generate 15-minute presigned URL
GET /                                Root (returns 404)
```

### Security Features

- Traefik Forward Auth integration (no auth logic in service)
- Presigned R2 URLs with 15-minute TTL
- Input validation for version, platform, channel
- No public API documentation (docs_url=None)
- Non-root Docker user
- Health checks for R2 connectivity

### Deployment Options

- Local development with MinIO
- Docker Compose (production)
- Kubernetes with manifests
- Traefik integration with labels

## Quick Reference

### Common Tasks

```bash
# Local development
docker compose --profile dev up
curl http://localhost:8000/health

# Run tests
docker compose exec releases-api pytest test_main.py -v

# Production deployment
docker compose -f docker-compose.prod.yml up -d

# View logs
docker compose logs -f releases-api

# Check R2 connection
curl -H "Authorization: Bearer TOKEN" \
  https://cco-api.visiquate.com/health
```

### Environment Variables Required

```bash
R2_ACCOUNT_ID=your-account-id
R2_ACCESS_KEY=your-access-key
R2_SECRET_KEY=your-secret-key
```

### Valid Platforms

- `darwin-arm64` (macOS Apple Silicon)
- `darwin-x86_64` (macOS Intel)
- `linux-x86_64` (Linux x86-64)
- `linux-aarch64` (Linux ARM64)
- `windows-x86_64` (Windows x86-64)

## Architecture

```
┌─────────────────┐
│  CCO Client     │
│  (with token)   │
└────────┬────────┘
         │
    ┌────▼──────────────┐
    │ Traefik Proxy    │
    │ + Forward Auth   │
    │ (Auth validates) │
    └────┬─────────────┘
         │
    ┌────▼──────────────┐
    │ Releases API     │
    │ (no auth logic)  │
    └────┬─────────────┘
         │
    ┌────▼──────────────┐
    │ CloudFlare R2    │
    │ (presigned URLs) │
    └──────────────────┘
```

## Cost Analysis

| Component | Monthly Cost |
|-----------|-------------|
| R2 Storage (10GB) | ~$0.15 |
| R2 Operations | ~$0.00 |
| R2 Egress | **$0.00** (FREE!) |
| API Service | $0.00 (shared infra) |
| **Total** | **~$0.15** |

## Compliance & Standards

- ✓ OWASP API Security Top 10
- ✓ AWS S3 best practices
- ✓ Docker security scanning (Trivy)
- ✓ Python code quality (pylint, black, mypy)
- ✓ Unit test coverage (pytest with fixtures)
- ✓ Security scanning (bandit, safety)

## File Structure

```
releases-api/
├── Documentation
│   ├── README.md                    # Main documentation
│   ├── QUICKSTART.md                # 5-minute guide
│   ├── SECURITY.md                  # Security best practices
│   ├── DEPLOYMENT.md                # Production deployment
│   ├── CONFIG_REFERENCE.md          # Configuration guide
│   └── INDEX.md                     # This file
│
├── Application Code
│   ├── main.py                      # FastAPI app
│   ├── test_main.py                 # Tests
│   ├── requirements.txt             # Dependencies
│   └── requirements-dev.txt         # Dev tools
│
├── Docker
│   ├── Dockerfile                   # Container image
│   ├── docker-compose.yml           # Local dev setup
│   └── .dockerignore                # Docker ignore rules
│
├── Configuration
│   ├── .env.example                 # Environment template
│   ├── .gitignore                   # Git ignore rules
│   └── Makefile                     # Development tasks
│
├── CI/CD
│   └── .github/workflows/
│       └── test-and-build.yml       # GitHub Actions pipeline
│
└── Root
    └── releases-api/                # This directory
```

## Version Information

| Item | Version |
|------|---------|
| Python | 3.11 |
| FastAPI | 0.104.1 |
| boto3 | 1.28.85 |
| Docker | 24.x |
| Docker Compose | 2.x |

## Related Documentation

- **CCO Main Repository**: `/Users/brent/git/cc-orchestra/`
- **Plan Document**: `/Users/brent/.claude/plans/wild-churning-trinket.md`
- **Global CLAUDE.md**: `/Users/brent/.claude/CLAUDE.md`
- **Orchestrator Rules**: `/Users/brent/git/cc-orchestra/ORCHESTRATOR_RULES.md`

## Important Links

### CloudFlare Resources
- [R2 Dashboard](https://dash.cloudflare.com/)
- [R2 API Documentation](https://developers.cloudflare.com/r2/)
- [boto3 S3 API Reference](https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/s3.html)

### FastAPI Resources
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Pydantic Documentation](https://docs.pydantic.dev/)
- [boto3 S3 Examples](https://boto3.amazonaws.com/v1/documentation/api/latest/guide/s3.html)

### Security Resources
- [OWASP Top 10](https://owasp.org/Top10/)
- [OWASP API Security](https://owasp.org/www-project-api-security/)
- [AWS S3 Security](https://docs.aws.amazon.com/AmazonS3/latest/userguide/security.html)

## Troubleshooting

### Health Check Failing

```bash
# Check if service is running
docker ps | grep releases-api

# View logs
docker compose logs releases-api

# Test R2 connection
aws s3 ls s3://cco-releases-private/ \
  --endpoint-url https://{ACCOUNT_ID}.r2.cloudflarestorage.com
```

See **DEPLOYMENT.md** section "Troubleshooting" for more.

### Tests Not Running

```bash
# Install dependencies
pip install -r requirements-dev.txt

# Run tests
pytest test_main.py -v

# View specific test
pytest test_main.py::TestHealthCheck -v
```

See **README.md** section "Development" for more.

### Deployment Issues

See **DEPLOYMENT.md** section "Troubleshooting" for comprehensive debugging guide.

## Support & Feedback

For issues, questions, or feedback:

1. Check the relevant documentation file
2. Review troubleshooting sections
3. Check GitHub issues
4. Create new issue with details

## Checklist for New Users

- [ ] Read QUICKSTART.md (5 minutes)
- [ ] Copy .env.example to .env
- [ ] Add R2 credentials to .env
- [ ] Run `docker compose --profile dev up`
- [ ] Test health check: `curl http://localhost:8000/health`
- [ ] Read README.md for full API documentation
- [ ] Review SECURITY.md for credential best practices
- [ ] Check DEPLOYMENT.md for production setup
- [ ] Run tests: `docker compose exec releases-api pytest test_main.py -v`

## Checklist for DevOps/Operations

- [ ] Review SECURITY.md for credential handling
- [ ] Review DEPLOYMENT.md for production setup
- [ ] Review CONFIG_REFERENCE.md for environment variables
- [ ] Set up R2 bucket and API credentials
- [ ] Configure Traefik forward auth middleware
- [ ] Deploy service (Docker Compose or Kubernetes)
- [ ] Monitor health checks
- [ ] Set up log rotation
- [ ] Plan credential rotation schedule (every 90 days)

## Next Steps

1. **Immediate**: Read QUICKSTART.md
2. **Next**: Deploy locally and test API
3. **Production**: Review DEPLOYMENT.md and SECURITY.md
4. **Ongoing**: Monitor health, rotate credentials, update dependencies

---

**Last Updated**: 2025-11-24
**Service Version**: 2025.11.24
**Documentation Version**: 1.0
**Status**: Production Ready
