# Web UI Backend Integration - Testing Guide

## Changes Made

### Modified Files
- `/Users/brent/git/cc-orchestra/cco/src/server.rs`

### Changes Summary

#### 1. Added Import for HTML Responses
```rust
use axum::{
    extract::{Json, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},  // Added Html
    routing::{get, post},
    Router,
};
```

#### 2. Enhanced Data Structures
Added comprehensive response structures:
- `HealthResponse` - Enhanced with analytics data (cache_hit_rate, total_requests, total_savings)
- `StatsResponse` - Complete analytics stats (cache, models, totals)
- `CacheStats` - Cache performance metrics
- `ModelStats` - Per-model usage statistics
- `TotalStats` - Aggregate statistics

#### 3. New Endpoints

##### Dashboard Static Files
- `GET /` - Serves `dashboard.html` using `include_str!` macro
- `GET /dashboard.css` - Serves CSS with correct content-type
- `GET /dashboard.js` - Serves JavaScript with correct content-type

##### API Endpoints
- `GET /api/stats` - Returns comprehensive analytics JSON:
  ```json
  {
    "cache": {
      "hit_rate": 70.0,
      "hits": 7,
      "misses": 3,
      "entries": 10,
      "total_savings": 52.5
    },
    "models": [
      {
        "model": "claude-opus-4",
        "requests": 10,
        "cache_hits": 7,
        "cache_misses": 3,
        "actual_cost": 157.5,
        "would_be_cost": 525.0,
        "savings": 367.5
      }
    ],
    "totals": {
      "requests": 10,
      "actual_cost": 157.5,
      "would_be_cost": 525.0,
      "total_savings": 367.5
    }
  }
  ```

##### Enhanced Health Endpoint
- `GET /health` - Now includes analytics data:
  ```json
  {
    "status": "ok",
    "version": "2025.11.1",
    "cache_hit_rate": 70.0,
    "total_requests": 10,
    "total_savings": 367.5
  }
  ```

#### 4. Router Configuration
Updated router with new routes in logical order:
```rust
let app = Router::new()
    // Dashboard routes
    .route("/", get(dashboard_html))
    .route("/dashboard.css", get(dashboard_css))
    .route("/dashboard.js", get(dashboard_js))
    // API routes
    .route("/health", get(health))
    .route("/api/stats", get(stats))
    .route("/v1/chat/completions", post(chat_completion))
    .layer(CorsLayer::permissive())
    .with_state(state);
```

#### 5. Enhanced Logging
```
✅ Server listening on http://127.0.0.1:3333
→ Dashboard: http://127.0.0.1:3333/
→ Health check: http://127.0.0.1:3333/health
→ Analytics API: http://127.0.0.1:3333/api/stats
→ Chat endpoint: http://127.0.0.1:3333/v1/chat/completions
```

## Testing Steps

### 1. Build the Project
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo build --release
```

### 2. Run the Server
```bash
cargo run -- run --port 3333
```

Expected output:
```
🚀 Starting CCO Proxy Server v0.0.0
→ Host: 127.0.0.1
→ Port: 3333
→ Cache size: 1073741824 bytes (1073 MB)
→ Cache TTL: 86400 seconds (24 hours)
✅ Server listening on http://127.0.0.1:3333
→ Dashboard: http://127.0.0.1:3333/
→ Health check: http://127.0.0.1:3333/health
→ Analytics API: http://127.0.0.1:3333/api/stats
→ Chat endpoint: http://127.0.0.1:3333/v1/chat/completions

Press Ctrl+C to stop
```

### 3. Test Dashboard
Open browser to: http://localhost:3333/

Expected:
- Dashboard HTML loads with full styling
- No blank page
- VisiQuate-style dark theme applied
- Tab navigation visible

### 4. Test API Endpoints

#### Health Check
```bash
curl http://localhost:3333/health | jq
```

Expected:
```json
{
  "status": "ok",
  "version": "2025.11.1",
  "cache_hit_rate": 0.0,
  "total_requests": 0,
  "total_savings": 0.0
}
```

#### Analytics Stats
```bash
curl http://localhost:3333/api/stats | jq
```

Expected:
```json
{
  "cache": {
    "hit_rate": 0.0,
    "hits": 0,
    "misses": 0,
    "entries": 0,
    "total_savings": 0.0
  },
  "models": [],
  "totals": {
    "requests": 0,
    "actual_cost": 0.0,
    "would_be_cost": 0.0,
    "total_savings": 0.0
  }
}
```

#### Static Files
```bash
# Test CSS
curl http://localhost:3333/dashboard.css | head -20

# Test JavaScript
curl http://localhost:3333/dashboard.js | head -20
```

### 5. Test with Sample Data

Send a test request to populate analytics:
```bash
curl -X POST http://localhost:3333/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-opus-4",
    "messages": [
      {"role": "user", "content": "Hello, world!"}
    ]
  }'
```

Then check stats again:
```bash
curl http://localhost:3333/api/stats | jq
```

Should now show data in the response.

### 6. Browser Testing

Open browser console (F12) and check:
```javascript
// Test API endpoint
fetch('/api/stats')
  .then(r => r.json())
  .then(d => console.log(d));

// Test health endpoint
fetch('/health')
  .then(r => r.json())
  .then(d => console.log(d));
```

## Expected Results

### Success Criteria
✅ Dashboard loads without blank page
✅ CSS and JS files load correctly
✅ `/api/stats` returns JSON with analytics data
✅ `/health` includes cache hit rate and savings
✅ No 404 errors in browser console
✅ No Rust compilation errors

### Common Issues & Solutions

**Issue**: Blank HTML page
- **Cause**: Routes not registered or static files not found
- **Solution**: Verify routes are in correct order, check `include_str!` paths

**Issue**: 404 on static files
- **Cause**: Route paths don't match HTML references
- **Solution**: Ensure `/dashboard.css` and `/dashboard.js` routes match HTML `<link>` and `<script>` tags

**Issue**: CORS errors
- **Cause**: CORS layer not applied
- **Solution**: Verify `CorsLayer::permissive()` is in router

**Issue**: Empty analytics data
- **Cause**: No requests processed yet
- **Solution**: Send test chat completion request first

## Architecture Notes

### Static File Serving
Using `include_str!` macro to embed static files at compile time:
- **Pros**: No filesystem dependency, single binary deployment
- **Cons**: Requires recompile to update static files
- **Alternative**: Use `tower-http::services::ServeDir` for development

### Data Flow
```
Browser Request → Axum Router → Handler Function
                                    ↓
                            Extract State (Arc<ServerState>)
                                    ↓
                            Query Analytics/Cache
                                    ↓
                            Serialize to JSON
                                    ↓
                            Return Response
```

### Memory Safety
- All state wrapped in `Arc<T>` for shared ownership
- Analytics uses `Mutex` internally for thread-safe updates
- Cache uses Moka's concurrent implementation

## Next Steps

1. Test SSE streaming (not yet implemented)
2. Test WebSocket terminal (not yet implemented)
3. Add integration tests
4. Consider using `tower-http::services::ServeDir` for development
5. Add compression middleware for static assets
6. Add rate limiting for API endpoints

## Files Modified
- `/Users/brent/git/cc-orchestra/cco/src/server.rs` - Complete web UI integration

## No Changes Needed
- `/Users/brent/git/cc-orchestra/cco/Cargo.toml` - All dependencies already present
- Static files (`dashboard.html`, `dashboard.css`, `dashboard.js`) - Work as-is
