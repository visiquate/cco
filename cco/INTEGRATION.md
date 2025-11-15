# CCO Dashboard - Backend Integration Guide

This guide helps integrate the frontend dashboard with your Rust backend.

## File Structure

```
cco/
├── static/
│   ├── dashboard.html       # Main dashboard UI
│   ├── dashboard.css        # Styling (17KB)
│   ├── dashboard.js         # JavaScript (23KB)
│   └── README.md            # Frontend documentation
├── INTEGRATION.md           # This file
└── src/
    └── main.rs              # Rust backend (to be implemented)
```

## Required Routes

Your Rust backend must implement these HTTP and WebSocket routes:

### 1. Dashboard HTML

```rust
#[get("/")]
async fn dashboard() -> impl Responder {
    let html = include_str!("../../static/dashboard.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
```

### 2. Static Assets

```rust
#[get("/static/dashboard.css")]
async fn dashboard_css() -> impl Responder {
    let css = include_str!("../../static/dashboard.css");
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(css)
}

#[get("/static/dashboard.js")]
async fn dashboard_js() -> impl Responder {
    let js = include_str!("../../static/dashboard.js");
    HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .body(js)
}
```

### 3. API Endpoints

#### Current Project Stats

```rust
#[get("/api/project/stats")]
async fn project_stats(state: web::Data<AppState>) -> impl Responder {
    let stats = serde_json::json!({
        "cost": 45.67,
        "costTrend": {
            "value": 5.2,
            "period": "24h"
        },
        "tokens": 123456,
        "tokensTrend": {
            "value": -2.1,
            "period": "24h"
        },
        "calls": 89,
        "callsTrend": {
            "value": 12.3,
            "period": "24h"
        },
        "avgTime": 245,
        "timeTrend": {
            "value": -8.5,
            "period": "24h"
        }
    });
    HttpResponse::Ok().json(stats)
}
```

#### Machine-Wide Stats

```rust
#[get("/api/machine/stats")]
async fn machine_stats(state: web::Data<AppState>) -> impl Responder {
    let stats = serde_json::json!({
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
            },
            // More projects...
        ],
        "models": [
            {
                "name": "claude-opus-4.1",
                "calls": 500,
                "inputTokens": 100000,
                "outputTokens": 50000,
                "cost": 456.78
            },
            {
                "name": "claude-sonnet-4.5",
                "calls": 1000,
                "inputTokens": 200000,
                "outputTokens": 100000,
                "cost": 234.56
            },
            {
                "name": "claude-haiku-4.5",
                "calls": 2000,
                "inputTokens": 100000,
                "outputTokens": 50000,
                "cost": 45.67
            }
        ],
        "chartData": {
            "costOverTime": [
                { "date": "2024-11-01T00:00:00Z", "cost": 45.23 },
                { "date": "2024-11-02T00:00:00Z", "cost": 52.11 },
                // 30 days of data
            ],
            "costByProject": [
                { "project": "Project A", "cost": 456.78 },
                { "project": "Project B", "cost": 234.56 },
                // Top 10 projects
            ],
            "modelDistribution": [
                { "model": "claude-opus-4.1", "count": 500 },
                { "model": "claude-sonnet-4.5", "count": 1000 },
                { "model": "claude-haiku-4.5", "count": 2000 }
            ]
        },
        "discrepancies": [
            {
                "title": "Unused API Key",
                "description": "API key 'old-key-2024' has not been used in 30 days"
            },
            {
                "title": "High Error Rate",
                "description": "Project B has 12% error rate (up from 2% average)"
            }
        ]
    });
    HttpResponse::Ok().json(stats)
}
```

### 4. Server-Sent Events (SSE)

```rust
#[get("/api/stream")]
async fn analytics_stream() -> impl Responder {
    // Use actix-web SSE response
    let (sender, body) = mpsc::channel(100);

    // Spawn task to send updates
    actix_rt::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));

        loop {
            interval.tick().await;

            let update = serde_json::json!({
                "project": {
                    "cost": 45.67,
                    "costTrend": { "value": 5.2, "period": "24h" },
                    "tokens": 123456,
                    "calls": 89,
                    "avgTime": 245
                },
                "activity": {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "event": "API call completed",
                    "type": "api",
                    "duration": 245,
                    "cost": 0.042,
                    "status": "success"
                }
            });

            let event = format!("event: analytics\ndata: {}\n\n", update.to_string());
            if sender.send(Ok(web::Bytes::from(event))).await.is_err() {
                break;
            }
        }
    });

    HttpResponse::Ok()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Access-Control-Allow-Origin", "*")
        .streaming_body(body)
}
```

### 5. WebSocket Terminal

```rust
use actix_web_actors::ws;

#[get("/terminal")]
async fn terminal_ws(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(TerminalSession::new(), &req, stream)
}

struct TerminalSession {
    // Terminal state
}

impl Actor for TerminalSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TerminalSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Binary(bytes)) => {
                // Handle terminal input
                let input = String::from_utf8_lossy(&bytes);
                // Execute command and send output
                let output = execute_command(&input);
                ctx.binary(output);
            }
            _ => {}
        }
    }
}
```

## Data Types

### Project Stats Response

```json
{
  "cost": 45.67,
  "costTrend": { "value": 5.2, "period": "24h" },
  "tokens": 123456,
  "tokensTrend": { "value": -2.1, "period": "24h" },
  "calls": 89,
  "callsTrend": { "value": 12.3, "period": "24h" },
  "avgTime": 245,
  "timeTrend": { "value": -8.5, "period": "24h" }
}
```

### Machine Stats Response

```json
{
  "totalCost": 1234.56,
  "activeProjects": 7,
  "totalCalls": 45678,
  "totalTokens": 12345678,
  "projects": [
    {
      "name": "string",
      "calls": 1234,
      "inputTokens": 50000,
      "outputTokens": 30000,
      "cost": 123.45,
      "lastActivity": "ISO 8601 timestamp"
    }
  ],
  "models": [
    {
      "name": "string",
      "calls": 500,
      "inputTokens": 100000,
      "outputTokens": 50000,
      "cost": 456.78
    }
  ],
  "chartData": {
    "costOverTime": [
      { "date": "ISO 8601 timestamp", "cost": 45.23 }
    ],
    "costByProject": [
      { "project": "string", "cost": 456.78 }
    ],
    "modelDistribution": [
      { "model": "string", "count": 500 }
    ]
  },
  "discrepancies": [
    {
      "title": "string",
      "description": "string"
    }
  ]
}
```

### SSE Activity Event

```json
{
  "activity": {
    "timestamp": "ISO 8601 timestamp",
    "event": "string description",
    "type": "api|error|model",
    "duration": 245,
    "cost": 0.042,
    "status": "success|error|pending"
  }
}
```

## Implementation Checklist

- [ ] Embed dashboard.html in binary with `include_str!`
- [ ] Serve dashboard.css from static route
- [ ] Serve dashboard.js from static route
- [ ] Implement `GET /api/project/stats` endpoint
- [ ] Implement `GET /api/machine/stats` endpoint
- [ ] Implement `GET /api/stream` SSE endpoint
- [ ] Implement `GET /terminal` WebSocket endpoint
- [ ] Test all endpoints with sample data
- [ ] Verify SSE streaming works (send every 1-5 seconds)
- [ ] Verify WebSocket binary I/O works
- [ ] Set CORS headers if needed
- [ ] Test on mobile (responsive)
- [ ] Verify charts render on machine tab
- [ ] Verify terminal works on terminal tab

## Testing

### Manual Testing

1. **Dashboard loads**: Navigate to http://localhost:3939
2. **Project stats appear**: Check Current Project tab
3. **Charts render**: Click Machine-Wide Analytics tab
4. **Activity updates**: Watch Activity table update every 2-5s
5. **Terminal works**: Click Live Terminal, type commands

### Sample Commands for Terminal

```bash
# Test commands for terminal tab
node /Users/brent/git/cc-orchestra/src/knowledge-manager.js stats
docker ps
git log --oneline -10
ps aux | grep node
```

## Performance Optimization

1. **Reduce SSE frequency**: Send updates every 5 seconds instead of 1
2. **Paginate tables**: Load 20 rows initially, lazy-load more
3. **Debounce charts**: Redraw only when data significantly changes
4. **Cache static assets**: Set Cache-Control headers
5. **Compress responses**: Use gzip for JSON responses

## Security Considerations

1. **CORS**: Allow dashboard domain only in production
2. **Authentication**: Add auth middleware if needed
3. **Rate limiting**: Limit SSE and WebSocket connections
4. **XSS protection**: Dashboard already escapes HTML
5. **Secrets**: Never expose sensitive data in JSON responses

## Troubleshooting

### Charts not rendering
- Check D3 CDN is accessible
- Verify chart data format matches schema
- Check browser console for errors

### Terminal not connecting
- Verify WebSocket endpoint is at `/terminal`
- Check WebSocket port is open (usually 3939)
- Verify binary message encoding/decoding

### SSE not updating
- Verify `Content-Type: text/event-stream` header
- Check event format: `event: analytics\ndata: {...}\n\n`
- Verify interval is 1-5 seconds

### Mobile layout issues
- Check viewport meta tag in HTML
- Verify CSS media queries work
- Test on actual mobile device

## Questions?

Refer to:
- Frontend documentation: `/Users/brent/git/cc-orchestra/cco/static/README.md`
- API reference: Schema in this file
- Example responses: Sample data in implementation checklist
