# Phase 1C Quick Start Guide

## What Was Built

Phase 1C adds the `/api/classify` endpoint to the CCO daemon for CRUD classification of shell commands.

## Quick Test

```bash
# 1. Build the project
cd /Users/brent/git/cc-orchestra/cco
cargo build --release

# 2. Start the daemon
./target/release/cco daemon start

# 3. Wait for initialization (3 seconds)
sleep 3

# 4. Test the endpoint
curl -X POST http://127.0.0.1:3000/api/classify \
  -H "Content-Type: application/json" \
  -d '{"command": "ls -la"}' | jq

# Expected output:
# {
#   "classification": "Read",
#   "confidence": 0.95,
#   "reasoning": "LLM response: READ",
#   "timestamp": "2025-11-17T..."
# }

# 5. Check health
curl http://127.0.0.1:3000/health | jq

# 6. Run full test suite
./test-classify-endpoint.sh

# 7. Stop the daemon
./target/release/cco daemon stop
```

## Files Changed

1. **cco/src/daemon/server.rs** (NEW) - Daemon HTTP server
2. **cco/src/daemon/mod.rs** - Export server module
3. **cco/src/daemon/hooks/mod.rs** - Export CrudClassifier
4. **cco/src/main.rs** - Wire daemon run command
5. **cco/src/api_client.rs** - Add hooks status to health
6. **cco/test-classify-endpoint.sh** (NEW) - Test script

## API Endpoints

### POST /api/classify

**Request:**
```json
{
  "command": "rm -rf /tmp/test",
  "context": "user-initiated"  // optional
}
```

**Response:**
```json
{
  "classification": "Delete",
  "confidence": 0.92,
  "reasoning": "Command removes files/directories",
  "timestamp": "2025-11-17T10:30:00Z"
}
```

**Classification Types:**
- `Read` - Safe read-only operations
- `Create` - Creates new resources  
- `Update` - Modifies existing resources
- `Delete` - Removes resources

### GET /health

**Response:**
```json
{
  "status": "ok",
  "version": "2025.11.2",
  "uptime_seconds": 3600,
  "port": 3000,
  "hooks": {
    "enabled": true,
    "classifier_available": true,
    "model_loaded": true,
    "model_name": "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
    "classification_latency_ms": null
  }
}
```

## Troubleshooting

**Q: Daemon fails to start?**
```bash
# Check if port 3000 is in use
lsof -i :3000

# Try different port
cco daemon start --port 3001
```

**Q: Classifier returns 503 Service Unavailable?**

Hooks system is disabled. Enable in config:
```toml
# ~/.cco/config.toml
[hooks]
enabled = true
```

**Q: Model download slow?**

First classification triggers model download (~600MB). Subsequent calls use cached model.

**Q: How to check daemon status?**
```bash
cco daemon status
```

## Next Steps

Phase 2 will use this `/api/classify` endpoint for permission gating in the TUI.

---

**Status:** ✅ Phase 1C Complete
**Date:** November 17, 2025
