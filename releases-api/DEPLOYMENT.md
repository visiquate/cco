# Deployment Guide for CCO Releases API

## Overview

This guide covers deploying the Releases API to production with Traefik integration.

## Prerequisites

- Docker and Docker Compose
- CloudFlare R2 account with bucket created
- R2 API credentials (Account ID, Access Key, Secret Key)
- Traefik configured with Authentik forward auth
- Domain: `cco-api.visiquate.com`

## Environment Setup

### 1. CloudFlare R2 Configuration

```bash
# Create bucket
wrangler r2 bucket create cco-releases-private

# Generate API token
# 1. Go to https://dash.cloudflare.com/profile/api-tokens
# 2. Create Custom Token:
#    - Name: CCO Releases API
#    - Permissions: Account > R2 > Edit
#    - Include: Specific buckets only > cco-releases-private
# 3. Copy: Account ID, Access Key ID, Secret Access Key
```

### 2. Credential Storage

**Option A: Environment Variables (Simple)**

```bash
export R2_ACCOUNT_ID=abc123def456
export R2_ACCESS_KEY=xxxxxxxxxxxxxxxxxxxx
export R2_SECRET_KEY=yyyyyyyyyyyyyyyyyyyy
```

**Option B: Docker Secrets (Recommended)**

```bash
# Create secrets
echo "abc123def456" | docker secret create r2_account_id -
echo "xxxxxxxxxxxxxxxxxxxx" | docker secret create r2_access_key -
echo "yyyyyyyyyyyyyyyyyyyy" | docker secret create r2_secret_key -
```

**Option C: Kubernetes Secrets**

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: releases-api-secrets
type: Opaque
stringData:
  R2_ACCOUNT_ID: abc123def456
  R2_ACCESS_KEY: xxxxxxxxxxxxxxxxxxxx
  R2_SECRET_KEY: yyyyyyyyyyyyyyyyyyyy
```

### 3. Build Docker Image

```bash
# Build
docker build -t cco-releases-api:latest \
  --build-arg VERSION=2025.11.24 \
  --build-arg GIT_COMMIT=$(git rev-parse --short HEAD) \
  --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
  .

# Tag for registry
docker tag cco-releases-api:latest ghcr.io/YOUR_ORG/releases-api:2025.11.24

# Push to registry
docker push ghcr.io/YOUR_ORG/releases-api:2025.11.24
```

## Docker Compose Deployment

### 1. Production Configuration

Create `docker-compose.prod.yml`:

```yaml
services:
  releases-api:
    image: ghcr.io/YOUR_ORG/releases-api:2025.11.24
    container_name: cco-releases-api
    restart: always
    environment:
      ENVIRONMENT: production
      VERSION: 2025.11.24
      R2_ACCOUNT_ID: ${R2_ACCOUNT_ID}
      R2_ACCESS_KEY: ${R2_ACCESS_KEY}
      R2_SECRET_KEY: ${R2_SECRET_KEY}
      R2_BUCKET: cco-releases-private
    ports:
      - "8000:8000"
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.cco-api.rule=Host(`cco-api.visiquate.com`)"
      - "traefik.http.routers.cco-api.entrypoints=websecure"
      - "traefik.http.routers.cco-api.tls.certresolver=letsencrypt"
      - "traefik.http.routers.cco-api.middlewares=authentik@docker"
      - "traefik.http.services.cco-api.loadbalancer.server.port=8000"
      - "com.centurylinklabs.watchtower.enable=true"
    networks:
      - web
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s

networks:
  web:
    external: true
```

### 2. Deploy

```bash
# Create environment file
cat > .env.prod <<EOF
R2_ACCOUNT_ID=your-value
R2_ACCESS_KEY=your-value
R2_SECRET_KEY=your-value
VERSION=2025.11.24
ENVIRONMENT=production
EOF

# Deploy
docker compose -f docker-compose.prod.yml up -d

# Verify
docker compose -f docker-compose.prod.yml logs -f releases-api
curl -H "Authorization: Bearer TOKEN" https://cco-api.visiquate.com/health
```

## Kubernetes Deployment

### 1. Create Deployment Manifest

Create `k8s-deployment.yml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco-releases-api
  namespace: cco
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: cco-releases-api
  template:
    metadata:
      labels:
        app: cco-releases-api
    spec:
      containers:
      - name: releases-api
        image: ghcr.io/YOUR_ORG/releases-api:2025.11.24
        imagePullPolicy: Always
        ports:
        - containerPort: 8000
          name: http
        env:
        - name: ENVIRONMENT
          value: production
        - name: VERSION
          value: "2025.11.24"
        - name: R2_ACCOUNT_ID
          valueFrom:
            secretKeyRef:
              name: releases-api-secrets
              key: account_id
        - name: R2_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: releases-api-secrets
              key: access_key
        - name: R2_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: releases-api-secrets
              key: secret_key
        - name: R2_BUCKET
          value: cco-releases-private
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL

---
apiVersion: v1
kind: Service
metadata:
  name: cco-releases-api
  namespace: cco
spec:
  selector:
    app: cco-releases-api
  ports:
  - port: 80
    targetPort: 8000
    name: http
  type: ClusterIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cco-releases-api
  namespace: cco
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    traefik.ingress.kubernetes.io/router.middlewares: cco-authentik@kubernetescrd
spec:
  ingressClassName: traefik
  tls:
  - hosts:
    - cco-api.visiquate.com
    secretName: cco-api-tls
  rules:
  - host: cco-api.visiquate.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cco-releases-api
            port:
              number: 80
```

### 2. Deploy to Kubernetes

```bash
# Create namespace
kubectl create namespace cco

# Create secrets
kubectl create secret generic releases-api-secrets \
  --from-literal=account_id=YOUR_ACCOUNT_ID \
  --from-literal=access_key=YOUR_ACCESS_KEY \
  --from-literal=secret_key=YOUR_SECRET_KEY \
  -n cco

# Deploy
kubectl apply -f k8s-deployment.yml

# Verify
kubectl get pods -n cco
kubectl logs -n cco -l app=cco-releases-api -f
```

## Traefik Configuration

### 1. Add Service to Traefik

In your main `docker-compose.yml`:

```yaml
services:
  traefik:
    # ... existing traefik config ...
    command:
      - "--entrypoints.websecure.address=:443"
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@visiquate.com"
      # ... other config ...
```

### 2. Service Labels

Already included in deployment examples above.

### 3. Authentik Forward Auth Middleware

```yaml
# In traefik config or docker-compose
middleware:
  authentik@docker:
    forwardAuth:
      address: http://authentik:9000/outpost.goauthk.io/auth/traefik
      trustForwardHeader: true
      authResponseHeaders:
        - Remote-User
        - Remote-Groups
```

## Monitoring

### Health Checks

```bash
# Check service health
curl -H "Authorization: Bearer TOKEN" https://cco-api.visiquate.com/health

# Expected response
{
  "status": "healthy",
  "service": "cco-releases-api",
  "version": "2025.11.24",
  "r2_connected": true,
  "timestamp": "2025-11-24T10:30:00Z"
}
```

### Logs

```bash
# Docker Compose
docker compose logs -f releases-api

# Kubernetes
kubectl logs -n cco -l app=cco-releases-api -f

# Follow errors
docker compose logs releases-api | grep ERROR
```

### Metrics

Application provides health endpoint for monitoring:

```bash
# Prometheus scrape config (optional future enhancement)
scrape_configs:
  - job_name: 'cco-releases-api'
    static_configs:
      - targets: ['cco-api.visiquate.com:443']
```

## Updates & Rollbacks

### Rolling Update

```bash
# Update image
docker pull ghcr.io/YOUR_ORG/releases-api:2025.11.25

# Update docker-compose.prod.yml with new version
# Then redeploy
docker compose -f docker-compose.prod.yml up -d releases-api
```

### Automated Updates with Watchtower

```yaml
watchtower:
  image: containrrr/watchtower:latest
  volumes:
    - /var/run/docker.sock:/var/run/docker.sock
  command:
    - "--schedule"
    - "0 2 * * *"  # 2 AM daily
    - "--cleanup"
    - "--scope"
    - "cco"  # Only update CCO services
```

### Rollback

```bash
# Kubernetes
kubectl rollout undo deployment/cco-releases-api -n cco

# Docker Compose (revert docker-compose.prod.yml to previous version)
git checkout HEAD~1 docker-compose.prod.yml
docker compose -f docker-compose.prod.yml up -d releases-api
```

## Troubleshooting

### Service Not Responding

```bash
# Check if container is running
docker ps | grep releases-api

# Check logs
docker logs releases-api

# Check health
curl http://localhost:8000/health
```

### R2 Connection Errors

```bash
# Verify credentials in environment
docker exec releases-api env | grep R2_

# Test R2 access
aws s3 ls s3://cco-releases-private/ \
  --endpoint-url https://{R2_ACCOUNT_ID}.r2.cloudflarestorage.com
```

### Traefik Routing Issues

```bash
# Check Traefik logs
docker logs traefik

# Verify service is registered
curl http://localhost:8080/api/providers/docker/services
```

## Cleanup

```bash
# Stop service
docker compose -f docker-compose.prod.yml down

# Remove image
docker rmi ghcr.io/YOUR_ORG/releases-api:2025.11.24

# Remove secrets
docker secret rm r2_account_id r2_access_key r2_secret_key
```

## Next Steps

1. Deploy release binaries to R2
2. Create metadata files in R2
3. Configure Authentik OIDC application
4. Test end-to-end authentication flow
5. Deploy CCO client with update logic
6. Monitor logs and health checks

## Support

For issues or questions, see SECURITY.md or create an issue in the main repository.
