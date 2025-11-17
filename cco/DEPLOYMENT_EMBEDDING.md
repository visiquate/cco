# Deployment Guide: Compile-Time Agent Embedding

## Overview

CCO binaries include agent definitions compiled at build time. This guide explains how to deploy, distribute, and manage CCO in production environments.

## Key Deployment Benefits

| Aspect | Benefit |
|--------|---------|
| **Single Binary** | Everything in one executable - no config files needed |
| **Offline Capable** | Works without internet, no file I/O required |
| **Version Locked** | Agent definitions match the binary version exactly |
| **Cross-Platform** | macOS, Linux, Windows - same binary format |
| **Zero Setup** | No configuration directories to create |
| **No Dependencies** | No Rust installation, no cargo, no build tools |

## Distribution

### Before Distribution

**Build the binary:**

```bash
cd cco
cargo build --release
# Binary at: target/release/cco (~15-20MB)
```

**Verify the build:**

```bash
# Check version
./target/release/cco --version
# Output: cco 2025.11.2

# Test agents load
./target/release/cco run --port 3000 &
sleep 2
curl http://localhost:3000/api/agents | jq '.agents | length'
# Output: 119
```

### Distribution Methods

#### Method 1: GitHub Releases

```bash
# Create release tag
git tag v2025.11.2
git push origin v2025.11.2

# GitHub Actions builds and uploads:
# - cco-macos-arm64
# - cco-macos-x86_64
# - cco-linux-x86_64
# - cco-linux-arm64
# - cco-windows-x86_64.exe
```

#### Method 2: Direct Upload

```bash
# Upload to your server
scp target/release/cco user@server.com:/opt/cco/

# Or upload to artifact storage
aws s3 cp target/release/cco s3://releases/cco-2025.11.2-linux-x86_64
```

#### Method 3: Package Manager

```bash
# Create Homebrew formula
brew install visiquate/tools/cco

# Or distribute via other package managers
apt install cco              # Ubuntu/Debian
yum install cco             # RHEL/CentOS
pacman -S cco               # Arch Linux
```

### File Distribution Checklist

- [ ] Binary built and tested locally
- [ ] Version number set correctly
- [ ] Agent definitions verified in binary
- [ ] Release notes prepared
- [ ] Binary signed (optional but recommended)
- [ ] Checksum generated for integrity
- [ ] Upload to distribution platform
- [ ] Documentation updated
- [ ] Announcement prepared

## Installation

### macOS Installation

```bash
# Download binary
curl -L -o cco https://releases.visiquate.com/cco-latest-macos-arm64
chmod +x cco

# Option A: Use directly
./cco run --port 3000

# Option B: Install to PATH
mv cco /usr/local/bin/
cco run --port 3000

# Option C: Use with Homebrew
brew install visiquate/tools/cco
cco run --port 3000
```

### Linux Installation

```bash
# Download binary
curl -L -o cco https://releases.visiquate.com/cco-latest-linux-x86_64
chmod +x cco

# Option A: Use directly
./cco run --port 3000

# Option B: Install to PATH
sudo mv cco /usr/local/bin/
cco run --port 3000

# Option C: Install via package manager
sudo apt update
sudo apt install cco
cco run --port 3000
```

### Windows Installation

```powershell
# Download binary
Invoke-WebRequest -Uri "https://releases.visiquate.com/cco-latest-windows-x86_64.exe" -OutFile "cco.exe"

# Run directly
.\cco.exe run --port 3000

# Or add to PATH and run from anywhere
cco run --port 3000
```

## Running CCO

### Basic Usage

```bash
# Start CCO on port 3000
cco run --port 3000

# Start on custom port
cco run --port 8080

# Start on custom host
cco run --host 0.0.0.0 --port 3000

# With custom cache settings
cco run --port 3000 --cache-size 2147483648 --cache-ttl 7200
```

### Environment Variables

```bash
# Set version at build time (not at runtime)
CCO_VERSION=2025.11.2 cargo build --release

# Enable debug logging at runtime
RUST_LOG=debug cco run --port 3000

# Set debug level
RUST_LOG=warn,cco=debug cco run --port 3000
```

### Command-Line Options

```
cco run
  --port PORT                Default: 3000
  --host HOST                Default: 127.0.0.1
  --cache-size BYTES         Default: 1GB (1073741824)
  --cache-ttl SECONDS        Default: 1 hour (3600)
  --database-url URL         Default: sqlite://analytics.db
```

## Accessing Agent Definitions

### Via HTTP API

Agent definitions are available at runtime via HTTP:

```bash
# Start CCO
cco run --port 3000

# List all agents
curl http://localhost:3000/api/agents

# Get specific agent
curl http://localhost:3000/api/agents/chief-architect

# Get health check
curl http://localhost:3000/health
```

### Example Response

```json
{
  "agents": [
    {
      "name": "chief-architect",
      "model": "opus",
      "description": "Strategic architecture leadership and orchestra coordination",
      "tools": ["Read", "Write", "Edit", "TodoWrite", "Bash"]
    },
    {
      "name": "python-specialist",
      "model": "haiku",
      "description": "Python implementation specialist",
      "tools": ["Read", "Write", "Edit", "Bash"]
    }
    // ... 117 total agents
  ]
}
```

### No Filesystem Access Needed

The key advantage: Agent definitions are served from memory, not from disk:

```bash
# Works offline (no file I/O)
cco run --port 3000

# Definitions available immediately
curl http://localhost:3000/api/agents
# Fast response - data already in memory

# Works even if ~/.claude/agents/ doesn't exist
# Binary includes all definitions
```

## Version Management

### Version Format

CCO uses date-based versioning: `YYYY.MM.N`

```
2025.11.1       First release in November 2025
2025.11.2       Second release in November 2025
2025.12.1       First release in December 2025 (counter resets)
2026.1.1        First release in January 2026
```

### Setting Version

**At build time:**

```bash
# Use environment variable
CCO_VERSION=2025.11.2 cargo build --release

# Default if not set
cargo build --release  # Uses 2025.11.2 from build.rs
```

**At runtime:**

```bash
# Check version
cco --version
# Output: cco 2025.11.2

# Check embedded build info
curl http://localhost:3000/health
# Shows version in response
```

### Version in Binary

The version is a compile-time constant:

```rust
// In build.rs
let version = env::var("CCO_VERSION").unwrap_or_else(|_| "2025.11.2".to_string());
println!("cargo:rustc-env=CCO_VERSION={}", version);

// In code - available as constant
let version = env!("CCO_VERSION");  // "2025.11.2"
```

This means:
- Version is fixed in binary
- Cannot be changed at runtime
- Each binary has its version "baked in"
- No risk of version mismatches

### Deployment Workflow

```
1. Build binary with version
   CCO_VERSION=2025.11.2 cargo build --release

2. Test binary
   ./target/release/cco --version
   ./target/release/cco run --port 3000

3. Tag release
   git tag v2025.11.2
   git push origin v2025.11.2

4. Upload binary
   Upload to releases page, package manager, etc.

5. Deploy to production
   Download binary
   Make executable
   Run with: cco run --port 3000

6. Verify deployment
   curl http://localhost:3000/health
   curl http://localhost:3000/api/agents | wc -l
```

## Docker Deployment

### Dockerfile Example

```dockerfile
FROM debian:bookworm-slim

# Copy pre-built CCO binary
COPY cco /usr/local/bin/cco
RUN chmod +x /usr/local/bin/cco

# Create data directory for logs and analytics
RUN mkdir -p /data && chmod 777 /data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Expose port
EXPOSE 3000

# Run CCO
CMD ["cco", "run", "--host", "0.0.0.0", "--port", "3000"]
```

### Building Docker Image

```bash
# Build CCO first
cargo build --release

# Create Docker image
docker build -t cco:2025.11.2 .

# Push to registry
docker push myregistry.com/cco:2025.11.2

# Run container
docker run -p 3000:3000 cco:2025.11.2
```

### Docker Compose Example

```yaml
version: '3'

services:
  cco:
    image: cco:2025.11.2
    ports:
      - "3000:3000"
    volumes:
      - cco-data:/data
    environment:
      RUST_LOG: info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  cco-data:
```

## Kubernetes Deployment

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cco
  labels:
    app: cco
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cco
  template:
    metadata:
      labels:
        app: cco
    spec:
      containers:
      - name: cco
        image: myregistry.com/cco:2025.11.2
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: info
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: cco-service
spec:
  selector:
    app: cco
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

### Deploying to Kubernetes

```bash
# Apply manifest
kubectl apply -f cco-deployment.yaml

# Check deployment
kubectl get deployments
kubectl get pods
kubectl get svc cco-service

# View logs
kubectl logs -f deployment/cco

# Update deployment with new version
kubectl set image deployment/cco cco=myregistry.com/cco:2025.11.3 --record
kubectl rollout status deployment/cco
```

## Production Considerations

### Resource Requirements

**Minimum:**
- CPU: 256m (0.25 cores)
- RAM: 256MB
- Storage: 50MB (for logs/analytics)

**Recommended:**
- CPU: 500m (0.5 cores)
- RAM: 512MB
- Storage: 1GB

**High-Load:**
- CPU: 2-4 cores
- RAM: 2-4GB
- Storage: 10-50GB

### Monitoring

Monitor these endpoints:

```bash
# Health check
curl http://localhost:3000/health
# Returns: {"status": "ok", "version": "2025.11.2", ...}

# Cache statistics
curl http://localhost:3000/api/stats
# Returns: Cache hit rate, memory usage, cost savings

# Agent status
curl http://localhost:3000/api/agents
# Returns: All 119 agents with current definitions
```

### Log Management

Logs are written to:

```bash
# Log location (configured in server.rs)
~/.local/share/cco/logs/cco-3000.log

# Enable debug logging
RUST_LOG=debug cco run --port 3000

# Filter by module
RUST_LOG=warn,cco=debug cco run --port 3000
```

### Database

Analytics are stored in SQLite:

```bash
# Default location
./analytics.db

# Custom location
cco run --port 3000 --database-url sqlite:///var/lib/cco/analytics.db
```

## Updates and Rollback

### Updating CCO

```bash
# Stop current instance
pkill cco

# Download new binary
curl -L -o cco-new https://releases.visiquate.com/cco-2025.11.3-linux-x86_64
chmod +x cco-new

# Test new version
./cco-new --version
./cco-new run --port 3001 &
sleep 2
curl http://localhost:3001/health
pkill cco

# Replace old binary
mv cco cco.bak
mv cco-new cco

# Start new version
./cco run --port 3000
```

### Rollback

```bash
# Stop current instance
pkill cco

# Restore previous binary
mv cco cco.new
mv cco.bak cco

# Restart with previous version
./cco run --port 3000

# Verify rollback
curl http://localhost:3000/health
```

## Performance Optimization

### Caching

Agent definitions are cached in memory after loading:

```bash
# Cache is pre-populated at startup
cco run --port 3000
# All 119 agent definitions loaded into RAM

# Access is O(1) HashMap lookup
curl http://localhost:3000/api/agents/chief-architect
# Returns instantly from memory
```

### Multi-Instance Setup

For high availability:

```bash
# Start multiple instances on different ports
cco run --port 3000 &
cco run --port 3001 &
cco run --port 3002 &

# Load balance across instances
# Use haproxy, nginx, or cloud load balancer

# Each instance has full agent definitions
curl http://localhost:3000/api/agents | wc -l   # 119
curl http://localhost:3001/api/agents | wc -l   # 119
curl http://localhost:3002/api/agents | wc -l   # 119
```

## Security

### Agent Definitions Are Public

Agent definitions are served via HTTP without authentication:

```bash
# Public endpoint - no auth required
curl http://localhost:3000/api/agents

# If you need to restrict access, use:
# - Firewall rules
# - Network policies
# - API gateway with authentication
# - Reverse proxy (nginx, haproxy)
```

### Audit Trail

Enable debug logging to track access:

```bash
RUST_LOG=debug cco run --port 3000 2>&1 | tee cco.log
# Logs show which agents are queried
# Timestamps for audit trail
```

## See Also

- [EMBEDDING_ARCHITECTURE.md](EMBEDDING_ARCHITECTURE.md) - System design
- [BUILD_PROCESS.md](BUILD_PROCESS.md) - Build process details
- [EMBEDDING_IMPLEMENTATION.md](EMBEDDING_IMPLEMENTATION.md) - Code details
- [EMBEDDING_TROUBLESHOOTING.md](EMBEDDING_TROUBLESHOOTING.md) - Troubleshooting
- [config/agents/README.md](config/agents/README.md) - Agent development guide

## Summary

CCO's compile-time agent embedding enables:

1. **Simple Distribution**: Single standalone binary
2. **Easy Installation**: No configuration needed
3. **Reliable Deployment**: Version-locked definitions
4. **Cross-Platform**: Works on macOS, Linux, Windows
5. **Production Ready**: Docker, Kubernetes ready
6. **Scalable**: Multi-instance load balancing
7. **Offline Operation**: No filesystem access needed

Just download the binary, make it executable, and run!
