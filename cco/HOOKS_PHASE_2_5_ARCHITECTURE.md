# Claude Orchestra Hooks System - Phases 2-5 Architecture

## Executive Summary

This document defines the complete architecture for Phases 2-5 of the Claude Orchestra Hooks System, building upon the Phase 1A foundation of embedded CRUD classification. The architecture introduces permission confirmation, audit logging, TUI integration, and comprehensive documentation to create a production-ready command interception and safety system.

## Phase 2: Permission Confirmation System

### Overview
Implement a permission confirmation system that intercepts C/U/D operations and requests user approval before execution. READ operations are auto-approved for seamless workflow.

### Architecture Components

#### 2.1 RESTful API Endpoints

```rust
// New endpoints in daemon/server.rs
POST /api/hooks/permission-request
POST /api/hooks/permission-response
GET  /api/hooks/pending-permissions
DELETE /api/hooks/permission/{id}
```

#### 2.2 Data Structures

```rust
// daemon/hooks/permission.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: Uuid,
    pub command: String,
    pub classification: ClassificationResult,
    pub timestamp: SystemTime,
    pub context: PermissionContext,
    pub ttl_seconds: u32, // Auto-deny after timeout
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub request_id: Uuid,
    pub decision: PermissionDecision,
    pub reasoning: Option<String>,
    pub timestamp: SystemTime,
    pub responder: ResponderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionDecision {
    Approved,
    Denied,
    Skipped, // For dangerously-skip-confirmations mode
    Timeout, // Auto-denied after TTL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponderType {
    TuiInteractive,  // User clicked in TUI
    ApiClient,       // API response
    AutoApproval,    // READ operations
    ConfigOverride,  // dangerously-skip-confirmations
    Timeout,         // TTL expired
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    pub mode: ExecutionMode,
    pub user: Option<String>,
    pub working_directory: PathBuf,
    pub environment_hash: String, // Hash of env vars for security
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Interactive, // TUI mode
    Api,        // API mode
    Batch,      // Batch processing
}
```

#### 2.3 Permission Flow

```
Command Received
        │
        ▼
    Classify (Phase 1A)
        │
        ▼
    Is READ? ────Yes───→ Auto-approve → Execute
        │
        No
        │
        ▼
    Check Config
        │
    dangerously-skip?──Yes──→ Log & Execute
        │
        No
        │
        ▼
    Create Permission Request
        │
        ▼
    Store in Permission Queue
        │
        ▼
    Notify (TUI/API)
        │
        ▼
    Wait for Response (with TTL)
        │
    ┌───┴───┐
Approved    Denied/Timeout
    │           │
    ▼           ▼
Execute     Block & Log
```

#### 2.4 Storage Strategy

**Decision**: Use SQLite for permission history with in-memory cache for active requests

```rust
// daemon/hooks/permission_store.rs

pub struct PermissionStore {
    // Active requests (in-memory for speed)
    active: Arc<RwLock<HashMap<Uuid, PermissionRequest>>>,

    // Historical decisions (SQLite)
    db: Arc<Mutex<Connection>>,

    // TTL enforcement
    ttl_enforcer: Arc<Mutex<TtlEnforcer>>,
}

impl PermissionStore {
    pub async fn create_request(&self, request: PermissionRequest) -> Result<Uuid>;
    pub async fn record_decision(&self, response: PermissionResponse) -> Result<()>;
    pub async fn get_pending(&self) -> Result<Vec<PermissionRequest>>;
    pub async fn cleanup_expired(&self) -> Result<u32>;
}
```

#### 2.5 TUI Integration for Interactive Mode

```rust
// tui_app.rs additions

pub struct PermissionPrompt {
    request: PermissionRequest,
    display_state: PromptState,
}

pub enum PromptState {
    Waiting,
    Highlighted,
    Deciding(Decision),
}

// Non-blocking prompt overlay
impl PermissionPrompt {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Modal overlay with:
        // - Command display
        // - CRUD classification with color
        // - Confidence score
        // - [A]pprove / [D]eny buttons
        // - Timeout countdown
    }
}
```

#### 2.6 API Response Structure

```json
// POST /api/hooks/permission-request response
{
  "request_id": "uuid-here",
  "command": "rm -rf /tmp/test",
  "classification": {
    "type": "DELETE",
    "confidence": 0.95,
    "reasoning": "Command removes files/directories"
  },
  "requires_confirmation": true,
  "ttl_seconds": 30,
  "approval_url": "/api/hooks/permission-response",
  "auto_deny_at": "2025-11-17T12:00:30Z"
}

// POST /api/hooks/permission-response body
{
  "request_id": "uuid-here",
  "decision": "approved",
  "reasoning": "User confirmed deletion is intended"
}
```

## Phase 3: Decision History & Audit Logging

### Overview
Implement comprehensive audit logging for all classification decisions and permission responses, with efficient storage and query capabilities.

### Architecture Components

#### 3.1 Database Schema

```sql
-- SQLite schema in ~/.cco/hooks_audit.db

CREATE TABLE classification_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    classification TEXT NOT NULL CHECK(classification IN ('READ', 'CREATE', 'UPDATE', 'DELETE')),
    confidence REAL NOT NULL,
    reasoning TEXT,
    timestamp_ms INTEGER NOT NULL,
    inference_time_ms INTEGER,
    model_version TEXT,
    INDEX idx_timestamp (timestamp_ms DESC),
    INDEX idx_classification (classification),
    INDEX idx_command_prefix (command COLLATE NOCASE)
);

CREATE TABLE permission_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT UNIQUE NOT NULL,
    command TEXT NOT NULL,
    classification TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('approved', 'denied', 'skipped', 'timeout')),
    responder_type TEXT NOT NULL,
    reasoning TEXT,
    timestamp_ms INTEGER NOT NULL,
    response_time_ms INTEGER,
    user TEXT,
    working_directory TEXT,
    INDEX idx_request_id (request_id),
    INDEX idx_timestamp (timestamp_ms DESC),
    INDEX idx_decision (decision)
);

CREATE TABLE hooks_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_type TEXT NOT NULL,
    metric_value REAL NOT NULL,
    timestamp_ms INTEGER NOT NULL,
    metadata TEXT, -- JSON
    INDEX idx_type_time (metric_type, timestamp_ms DESC)
);

-- Retention view for cleanup
CREATE VIEW old_records AS
SELECT id, 'classification' as table_name FROM classification_history
WHERE id NOT IN (SELECT id FROM classification_history ORDER BY timestamp_ms DESC LIMIT 1000)
UNION ALL
SELECT id, 'permission' as table_name FROM permission_decisions
WHERE id NOT IN (SELECT id FROM permission_decisions ORDER BY timestamp_ms DESC LIMIT 1000);
```

#### 3.2 Audit Logger

```rust
// daemon/hooks/audit.rs

pub struct HooksAuditLogger {
    db: Arc<Mutex<Connection>>,
    buffer: Arc<RwLock<Vec<AuditEntry>>>,
    flush_interval: Duration,
    retention_limit: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub entry_type: AuditEntryType,
    pub timestamp: SystemTime,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub enum AuditEntryType {
    Classification(ClassificationAudit),
    Permission(PermissionAudit),
    Metric(MetricAudit),
}

impl HooksAuditLogger {
    pub async fn log_classification(&self, audit: ClassificationAudit) -> Result<()>;
    pub async fn log_permission(&self, audit: PermissionAudit) -> Result<()>;
    pub async fn log_metric(&self, metric: MetricAudit) -> Result<()>;

    // Batch insert for performance
    async fn flush_buffer(&self) -> Result<usize>;

    // Retention management
    pub async fn cleanup_old_records(&self) -> Result<usize>;

    // Query methods
    pub async fn get_recent_classifications(&self, limit: usize) -> Result<Vec<ClassificationAudit>>;
    pub async fn get_classification_stats(&self, window: Duration) -> Result<ClassificationStats>;
    pub async fn search_commands(&self, pattern: &str) -> Result<Vec<AuditEntry>>;
}
```

#### 3.3 Metrics Collection

```rust
// daemon/hooks/metrics.rs

pub struct HooksMetrics {
    classifications: Arc<RwLock<ClassificationMetrics>>,
    permissions: Arc<RwLock<PermissionMetrics>>,
    performance: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Default)]
pub struct ClassificationMetrics {
    pub total_count: u64,
    pub by_type: HashMap<CrudClassification, u64>,
    pub confidence_sum: f64,
    pub confidence_count: u64,
}

#[derive(Debug, Default)]
pub struct PermissionMetrics {
    pub total_requests: u64,
    pub approved: u64,
    pub denied: u64,
    pub timeouts: u64,
    pub auto_approvals: u64,
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub classification_times: Vec<u64>, // Rolling window
    pub response_times: Vec<u64>,
    pub model_load_time: Option<u64>,
}
```

#### 3.4 Cleanup Strategy

```rust
// Daemon lifecycle integration

impl DaemonLifecycle {
    async fn on_startup(&self) {
        // Start retention cleanup task
        tokio::spawn(async {
            let mut interval = time::interval(Duration::from_hours(1));
            loop {
                interval.tick().await;
                if let Err(e) = audit_logger.cleanup_old_records().await {
                    error!("Retention cleanup failed: {}", e);
                }
            }
        });
    }

    async fn on_shutdown(&self) {
        // Final flush of audit buffer
        audit_logger.flush_buffer().await?;

        // Optional: Archive old records before cleanup
        if config.archive_on_shutdown {
            audit_logger.archive_old_records().await?;
        }
    }
}
```

## Phase 4: TUI Hooks Status Display

### Overview
Integrate hooks status into the TUI with a dedicated section showing real-time classification activity, statistics, and model status.

### Architecture Components

#### 4.1 TUI Section Layout

```
┌─────────────────────────────────────────────────────┐
│                  Claude Orchestra TUI                │
├───────────────────────┬─────────────────────────────┤
│  Cost Monitor (60%)   │    Hooks Status (40%)       │
│                       │                              │
│  • Overall Summary    │  Status: ● Enabled          │
│  • Project Summaries  │  Model: TinyLLaMA (Loaded)  │
│  • Cost by Tier       │                              │
│  • Recent API Calls   │  Recent Classifications:     │
│                       │  [23:45:02] READ    ls -la  │
│                       │  [23:45:00] DELETE  rm tmp   │
│                       │  [23:44:58] CREATE  mkdir    │
│                       │  [23:44:55] READ    cat f    │
│                       │  [23:44:50] UPDATE  echo >>  │
│                       │                              │
│                       │  Statistics (last hour):    │
│                       │  ├─ READ:   142 (71%)       │
│                       │  ├─ CREATE:  32 (16%)       │
│                       │  ├─ UPDATE:  18 (9%)        │
│                       │  └─ DELETE:   8 (4%)        │
│                       │                              │
│                       │  Avg Response: 125ms        │
│                       │  Permissions: 3 pending     │
└───────────────────────┴─────────────────────────────┤
```

#### 4.2 Data Structure for TUI

```rust
// tui_app.rs additions

#[derive(Debug, Clone)]
pub struct HooksStatusSection {
    enabled: bool,
    model_status: ModelStatus,
    recent_classifications: VecDeque<ClassificationDisplay>,
    statistics: ClassificationStats,
    pending_permissions: Vec<PermissionRequest>,
    performance: PerformanceDisplay,
}

#[derive(Debug, Clone)]
pub struct ClassificationDisplay {
    timestamp: String,
    classification: CrudClassification,
    command: String, // Truncated for display
    confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ModelStatus {
    name: String,
    loaded: bool,
    memory_mb: u32,
    load_time_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PerformanceDisplay {
    avg_response_ms: u64,
    p95_response_ms: u64,
    total_today: u64,
}
```

#### 4.3 Color Coding

```rust
// tui/colors.rs

impl CrudClassification {
    pub fn to_color(&self) -> Color {
        match self {
            CrudClassification::Read => Color::Green,    // Safe
            CrudClassification::Create => Color::Yellow, // Caution
            CrudClassification::Update => Color::Yellow, // Caution
            CrudClassification::Delete => Color::Red,    // Danger
        }
    }

    pub fn to_symbol(&self) -> &'static str {
        match self {
            CrudClassification::Read => "●",    // Filled circle
            CrudClassification::Create => "◆",  // Diamond
            CrudClassification::Update => "▲",  // Triangle
            CrudClassification::Delete => "■",  // Square
        }
    }
}
```

#### 4.4 Real-time Updates

```rust
// SSE integration for live updates

pub struct HooksStatusStream {
    classifications: broadcast::Receiver<ClassificationEvent>,
    permissions: broadcast::Receiver<PermissionEvent>,
    metrics: broadcast::Receiver<MetricEvent>,
}

impl HooksStatusSection {
    pub async fn update(&mut self, event: HooksEvent) {
        match event {
            HooksEvent::Classification(c) => {
                // Add to recent list (keep last 5)
                self.recent_classifications.push_front(c.into());
                if self.recent_classifications.len() > 5 {
                    self.recent_classifications.pop_back();
                }
                // Update stats
                self.statistics.update(&c);
            }
            HooksEvent::Permission(p) => {
                self.pending_permissions = p.pending;
            }
            HooksEvent::Metric(m) => {
                self.performance.update(&m);
            }
        }
    }
}
```

#### 4.5 Responsive Layout

```rust
// Dynamic width allocation

impl HooksStatusSection {
    pub fn calculate_layout(&self, available_width: u16) -> Layout {
        if available_width < 50 {
            // Minimal mode - just status
            Layout::Minimal
        } else if available_width < 80 {
            // Compact mode - status + recent
            Layout::Compact
        } else {
            // Full mode - everything
            Layout::Full
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let layout = self.calculate_layout(area.width);

        match layout {
            Layout::Full => self.render_full(f, area),
            Layout::Compact => self.render_compact(f, area),
            Layout::Minimal => self.render_minimal(f, area),
        }
    }
}
```

## Phase 5: Documentation

### Overview
Comprehensive documentation covering user guides, developer guides, API references, and configuration examples.

### Documentation Structure

#### 5.1 User Guide (`docs/hooks-user-guide.md`)

```markdown
# Hooks System User Guide

## What Are Hooks?

Hooks intercept commands before execution to:
- Classify them (READ/CREATE/UPDATE/DELETE)
- Request permission for potentially dangerous operations
- Log all activity for audit purposes

## How It Works

### Automatic Classification
Every command is automatically classified using an embedded AI model...

### Permission Flow
1. READ operations execute immediately
2. CREATE/UPDATE/DELETE operations require confirmation
3. You have 30 seconds to respond before auto-deny

### TUI Interface
The hooks status appears in the right panel...

### Configuration
Edit ~/.cco/daemon.toml to customize behavior...
```

#### 5.2 Developer Guide (`docs/hooks-developer-guide.md`)

```markdown
# Hooks System Developer Guide

## Architecture Overview

### Component Hierarchy
- HooksRegistry: Central registration point
- HooksExecutor: Manages hook lifecycle
- LlmClassifier: Embedded CRUD classification
- PermissionStore: Request/response management
- AuditLogger: Historical tracking

## API Endpoints

### Classification
POST /api/hooks/classify
- Classify a command without executing

### Permissions
POST /api/hooks/permission-request
- Request permission for C/U/D operation

## Extension Points

### Custom Hooks
Implement the Hook trait...

### Custom Classifiers
Extend the Classifier trait...
```

#### 5.3 Configuration Guide (`docs/hooks-configuration.md`)

```toml
# Complete hooks configuration reference

[hooks]
# Enable/disable the hooks system
enabled = true

# Global timeout for hook execution (ms)
timeout_ms = 5000

# Maximum retry attempts
max_retries = 2

# Skip all confirmations (DANGEROUS!)
dangerously_skip_confirmations = false

[hooks.llm]
# Model configuration
model_type = "tinyllama"
model_name = "tinyllama-1.1b-chat-v1.0.Q4_K_M"
model_path = "~/.cco/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"

# Performance tuning
inference_timeout_ms = 2000
temperature = 0.1

[hooks.permissions]
# Permission behavior
auto_approve_read = true
ttl_seconds = 30
require_reason = false

[hooks.audit]
# Audit configuration
retention_records = 1000
archive_on_shutdown = false
database_path = "~/.cco/hooks_audit.db"
```

#### 5.4 Examples (`docs/hooks-examples.md`)

```markdown
# Hooks System Examples

## Basic Usage

### Command Classification
```bash
# Via API
curl -X POST http://localhost:3000/api/hooks/classify \
  -H "Content-Type: application/json" \
  -d '{"command": "rm -rf /tmp/test"}'

# Response
{
  "classification": "DELETE",
  "confidence": 0.95,
  "requires_confirmation": true
}
```

## Integration Examples

### Python Client
```python
import requests

class HooksClient:
    def classify(self, command):
        return requests.post(
            "http://localhost:3000/api/hooks/classify",
            json={"command": command}
        ).json()
```

## Troubleshooting

### Model Not Loading
Check ~/.cco/daemon.log for model download errors...
```

## Implementation Order & Agent Assignments

### Phase 2: Permission Confirmation (2-3 days)

1. **Backend Implementation** (Rust Expert + TDD Agent)
   - Create permission data structures
   - Implement PermissionStore with SQLite backend
   - Add API endpoints for permission request/response
   - Integrate with existing executor flow

2. **TUI Integration** (Rust Expert)
   - Add permission prompt overlay
   - Implement keyboard handling for approve/deny
   - Add timeout countdown display

3. **Testing** (QA Engineer)
   - Unit tests for permission flow
   - Integration tests with mock commands
   - TUI interaction tests

### Phase 3: Audit Logging (2 days)

1. **Database Setup** (Rust Expert + Database Architect)
   - Create SQLite schema
   - Implement migrations
   - Add retention policies

2. **Audit Implementation** (Rust Expert + TDD Agent)
   - Create AuditLogger with buffering
   - Implement metrics collection
   - Add cleanup tasks

3. **Testing** (QA Engineer)
   - Performance tests for batch inserts
   - Retention policy verification
   - Query performance optimization

### Phase 4: TUI Display (1-2 days)

1. **TUI Section** (Rust Expert)
   - Create HooksStatusSection component
   - Implement responsive layout
   - Add real-time SSE updates

2. **Visual Design** (User Experience Designer)
   - Color scheme for CRUD types
   - Layout optimization
   - Accessibility review

3. **Integration Testing** (QA Engineer)
   - SSE event flow testing
   - Layout responsiveness tests
   - Performance monitoring

### Phase 5: Documentation (1 day)

1. **User Documentation** (Technical Writer)
   - User guide with screenshots
   - Configuration examples
   - FAQ and troubleshooting

2. **Developer Documentation** (Documentation Lead)
   - API reference
   - Architecture diagrams
   - Extension guide

3. **Review** (Chief Architect)
   - Technical accuracy review
   - Consistency check
   - Final approval

## Configuration Changes Required

### daemon.toml Additions

```toml
[hooks.permissions]
enabled = true
auto_approve_read = true
ttl_seconds = 30
dangerously_skip_confirmations = false

[hooks.audit]
enabled = true
retention_records = 1000
database_path = "~/.cco/hooks_audit.db"
flush_interval_ms = 5000

[hooks.tui]
show_in_dashboard = true
max_recent_items = 5
show_statistics = true
```

## Risk Mitigation

### Performance Risks
- **Risk**: SQLite writes blocking classification
- **Mitigation**: Async writes with buffering, WAL mode

### Security Risks
- **Risk**: Permission bypass via API
- **Mitigation**: Signed requests, rate limiting

### UX Risks
- **Risk**: Too many permission prompts
- **Mitigation**: Smart grouping, "allow similar" option

## Success Metrics

1. **Performance**
   - Classification latency < 200ms p95
   - Permission decision < 100ms
   - Audit write < 50ms

2. **Reliability**
   - Zero lost audit records
   - 100% permission enforcement for C/U/D
   - Graceful degradation if model fails

3. **Usability**
   - < 3 clicks to approve/deny
   - Clear visual distinction for CRUD types
   - Intuitive configuration options

## Conclusion

This architecture provides a complete, production-ready hooks system with:
- Robust permission management for dangerous operations
- Comprehensive audit trail for compliance
- Intuitive TUI integration for operators
- Extensive documentation for users and developers

The phased approach ensures incremental value delivery while maintaining system stability throughout development.