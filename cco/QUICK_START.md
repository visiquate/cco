# CCO Dashboard - Quick Start Guide

Get the Claude Code Orchestrator analytics dashboard running in 5 minutes.

## Files Overview

```
cco/
├── static/
│   ├── dashboard.html          # Main dashboard HTML (embed in Rust binary)
│   ├── dashboard.css           # Dark theme styling
│   ├── dashboard.js            # Real-time analytics JavaScript
│   └── README.md               # Detailed frontend documentation
├── INTEGRATION.md              # Backend integration guide (Rust examples)
├── FRONTEND_SUMMARY.md         # Implementation summary
└── QUICK_START.md              # This file
```

## Step 1: Get Started with the Dashboard

### Quick Launch (with Auto-Open)

The easiest way to access the dashboard:

```bash
# Start CCO - dashboard auto-opens in your browser
./cco-proxy

# That's it! You should see the analytics dashboard at http://localhost:3000
```

### Manual Access

If the dashboard doesn't auto-open:

```bash
# Start CCO on a custom port
./cco-proxy --port 8000

# Open browser manually
open http://localhost:8000
```

### Dashboard Features

The dashboard provides three main tabs:
- **Tab 1:** Current project real-time metrics (cost, tokens, calls, response times)
- **Tab 2:** Machine-wide analytics with charts and project breakdowns
- **Tab 3:** Integrated terminal emulator for advanced management

## Step 2: Prepare Rust Backend

Your server needs these endpoints:

### HTTP Endpoints (REST JSON)

```bash
GET /                      → dashboard.html
GET /api/project/stats     → Project metrics (JSON)
GET /api/machine/stats     → Machine analytics (JSON)
```

### Streaming

```bash
GET /api/stream            → SSE (Server-Sent Events)
WS /terminal               → WebSocket terminal I/O
```

## Step 3: Embed Dashboard Files

In your Rust `main.rs`:

```rust
use actix_web::{web, App, HttpServer, get, HttpResponse};

#[get("/")]
async fn dashboard() -> HttpResponse {
    let html = include_str!("../cco/static/dashboard.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[get("/static/{filename}")]
async fn serve_static(filename: web::Path<String>) -> HttpResponse {
    match filename.as_str() {
        "dashboard.css" => {
            let css = include_str!("../cco/static/dashboard.css");
            HttpResponse::Ok()
                .content_type("text/css")
                .body(css)
        }
        "dashboard.js" => {
            let js = include_str!("../cco/static/dashboard.js");
            HttpResponse::Ok()
                .content_type("application/javascript")
                .body(js)
        }
        _ => HttpResponse::NotFound().finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(dashboard))
            .route("/static/{filename}", web::get().to(serve_static))
            .route("/api/project/stats", web::get().to(project_stats))
            .route("/api/machine/stats", web::get().to(machine_stats))
            .route("/api/stream", web::get().to(analytics_stream))
            .route("/terminal", web::get().to(terminal_ws))
    })
    .bind("0.0.0.0:3939")?
    .run()
    .await
}
```

## Step 4: Implement API Endpoints

### Project Stats Endpoint

```rust
#[get("/api/project/stats")]
async fn project_stats() -> impl Responder {
    web::Json(serde_json::json!({
        "cost": 45.67,
        "costTrend": { "value": 5.2, "period": "24h" },
        "tokens": 123456,
        "tokensTrend": { "value": -2.1, "period": "24h" },
        "calls": 89,
        "callsTrend": { "value": 12.3, "period": "24h" },
        "avgTime": 245,
        "timeTrend": { "value": -8.5, "period": "24h" }
    }))
}
```

### Machine Stats Endpoint

```rust
#[get("/api/machine/stats")]
async fn machine_stats() -> impl Responder {
    web::Json(serde_json::json!({
        "totalCost": 1234.56,
        "activeProjects": 7,
        "totalCalls": 45678,
        "totalTokens": 12345678,
        "projects": [
            {
                "name": "Project A",
                "calls": 1234,
                "inputTokens": 50000,
                "outputTokens": 30000,
                "cost": 123.45,
                "lastActivity": "2024-11-15T10:30:00Z"
            }
        ],
        "models": [
            {
                "name": "claude-opus-4.1",
                "calls": 500,
                "inputTokens": 100000,
                "outputTokens": 50000,
                "cost": 456.78
            }
        ],
        "chartData": {
            "costOverTime": [
                { "date": "2024-11-01T00:00:00Z", "cost": 45.23 }
            ],
            "costByProject": [
                { "project": "Project A", "cost": 456.78 }
            ],
            "modelDistribution": [
                { "model": "claude-opus-4.1", "count": 500 }
            ]
        },
        "discrepancies": []
    }))
}
```

### SSE Streaming

```rust
#[get("/api/stream")]
async fn analytics_stream() -> impl Responder {
    // Send analytics updates as Server-Sent Events
    // Format: "event: analytics\ndata: {...}\n\n"
    // Send every 2-5 seconds with partial updates
}
```

### WebSocket Terminal

```rust
#[get("/terminal")]
async fn terminal_ws(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(TerminalSession::new(), &req, stream)
}
```

See `INTEGRATION.md` for complete examples.

## Step 5: Test the Dashboard

### Auto-Open on Startup

1. Start CCO:
```bash
./cco-proxy
```

2. Dashboard automatically opens in your default browser
3. You should see:
   - Dashboard header with connection status
   - Three tabs: Project, Machine, Terminal
   - Real-time metrics updating
   - Auto-refresh every 5 seconds

### Manual Testing

If dashboard doesn't auto-open:

```bash
# Start server
cargo run

# Manual browser navigation
open http://localhost:3000
```

### Verify Each Tab Works

**Tab 1: Current Project**
- Shows cost, tokens, calls, response time metrics
- Activity table displays recent API calls
- Metrics update automatically every 5 seconds
- Cache hit rate displays prominently

**Tab 2: Machine-Wide Analytics**
- Summary cards show total costs and projects
- Charts render with historical data
- Tables display active projects and models
- Cost trends visible in graphs

**Tab 3: Live Terminal**
- Terminal is interactive and ready for commands
- Type commands and see output in real-time
- Terminal state persists when switching tabs
- Can manage cache and view system info

## Step 6: Customize (Optional)

### Change Colors

Edit `dashboard.css` `:root` section:
```css
:root {
    --accent-primary: #3b82f6;  /* Change to your brand color */
    --bg-primary: #0f172a;       /* Change background */
}
```

### Adjust Update Frequency

In `dashboard.js`:
```javascript
CONFIG.UPDATE_INTERVAL = 5000;  // Change from 5s to your preference
```

### Add More Projects/Models

Just add more rows to the JSON responses - frontend handles it automatically.

## Troubleshooting

### Dashboard Shows "Disconnected"

- Check if backend is running on localhost:3939
- Verify `/api/stream` endpoint is working
- Check browser console for errors

### Charts Not Rendering

- Verify D3.js CDN is accessible
- Check browser console for errors
- Ensure chart data is in correct format

### Terminal Not Working

- Check if WebSocket endpoint `/terminal` exists
- Verify WebSocket connection in browser DevTools
- Check binary message encoding/decoding

### Mobile Layout Broken

- Check viewport meta tag in HTML
- Verify CSS media queries work
- Test on actual mobile device

## API Reference Quick

See `INTEGRATION.md` for:
- Complete API schemas
- Example request/response
- Implementation checklist
- Performance optimization tips

## Files Reference

- **dashboard.html** - Structure (259 lines)
- **dashboard.css** - Styling (866 lines)
- **dashboard.js** - Logic (792 lines)
- **README.md** - Frontend docs
- **INTEGRATION.md** - Backend guide
- **FRONTEND_SUMMARY.md** - Implementation details

## Next Steps

1. Implement HTTP endpoints with real data
2. Connect to SSE stream
3. Connect to WebSocket terminal
4. Deploy to production
5. Monitor performance
6. Add authentication if needed

## Dependencies

**Zero NPM packages required!** The dashboard uses:
- Vanilla JavaScript (ES6+)
- D3.js v7 (CDN)
- xterm.js v5 (CDN)

All external libraries are loaded from CDN.

## Performance Tips

- SSE updates: Send every 5 seconds (not 1s)
- Paginate tables: Show 20 rows initially
- Debounce chart redraws
- Cache static assets (CSS, JS)
- Gzip JSON responses

## Security Notes

- Dashboard escapes HTML (XSS protection)
- Set CORS headers if frontend is on different domain
- Add authentication middleware if needed
- Rate limit WebSocket connections
- Never expose sensitive data in JSON

## Support

For detailed help:
- Backend integration: See `INTEGRATION.md`
- Frontend details: See `static/README.md`
- Implementation summary: See `FRONTEND_SUMMARY.md`

---

**You're ready!** Start with Step 1 and work through each step. The dashboard will be live on `http://localhost:3939` once the backend endpoints are implemented.
