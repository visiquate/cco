# Hooks API Reference

**Version**: 1.0.0
**Last Updated**: November 17, 2025
**Status**: Complete (Phases 2-5)

## Overview

The Hooks API provides programmatic access to the Claude Orchestra command classification and permission system. All endpoints are available on the local daemon and follow REST conventions.

**Base URL**: `http://localhost:3000/api/hooks`

**Authentication**: None required (local-only API)

**Content-Type**: Application/JSON

## Quick Reference

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/hooks/permission-request` | Classify a command and request permission |
| GET | `/api/hooks/decisions` | Retrieve audit trail of decisions |
| GET | `/api/hooks/stats` | Get aggregate statistics |
| DELETE | `/api/hooks/cleanup` | Delete old audit records |

## Endpoints

### POST /api/hooks/permission-request

Classify a command and determine whether it requires user confirmation.

**Purpose**: Primary endpoint for command classification. Called by Claude Code before executing user commands.

#### Request

**Content-Type**: `application/json`

**Request Body Schema**:

```json
{
  "command": "string (required)",
  "context": "string (optional)"
}
```

**Fields**:

| Field | Type | Required | Max Length | Description |
|-------|------|----------|------------|-------------|
| command | string | yes | 10000 | The full command text to classify |
| context | string | no | 100 | Optional context (e.g., "git", "npm", "docker") |

**Request Examples**:

Basic request:
```json
{
  "command": "git commit -m 'Fix bug in parser'"
}
```

With context:
```json
{
  "command": "npm install --save-dev jest",
  "context": "npm"
}
```

#### Response

**Success Response (200 OK)**:

```json
{
  "decision": "REQUIRES_CONFIRMATION",
  "classification": "UPDATE",
  "reasoning": "Modifying existing git repository state",
  "confidence": 0.96,
  "explanation": "The 'git commit' command records changes to the repository. This modifies your project history and requires approval.",
  "classification_method": "pattern_match",
  "timestamp": "2025-11-17T10:30:00Z"
}
```

**Response Fields**:

| Field | Type | Values | Description |
|-------|------|--------|-------------|
| decision | string | REQUIRES_CONFIRMATION, AUTO_ALLOWED | Whether confirmation is needed |
| classification | string | READ, CREATE, UPDATE, DELETE | Command type classification |
| reasoning | string | - | Brief reason for decision |
| confidence | number | 0.0 - 1.0 | Classifier confidence (0.0 = uncertain, 1.0 = certain) |
| explanation | string | - | Detailed explanation suitable for display |
| classification_method | string | pattern_match, semantic, fallback | How the classification was determined |
| timestamp | ISO8601 | - | When classification occurred |

**Response Examples**:

Auto-allowed READ operation:
```json
{
  "decision": "AUTO_ALLOWED",
  "classification": "READ",
  "reasoning": "Safe read operation",
  "confidence": 0.99,
  "explanation": "The 'ls' command displays files without modifying anything. No confirmation needed.",
  "classification_method": "pattern_match",
  "timestamp": "2025-11-17T10:30:00Z"
}
```

Requires confirmation CREATE operation:
```json
{
  "decision": "REQUIRES_CONFIRMATION",
  "classification": "CREATE",
  "reasoning": "Will create new files",
  "confidence": 0.94,
  "explanation": "The 'mkdir' command creates new directories. This adds resources to your system and requires approval.",
  "classification_method": "pattern_match",
  "timestamp": "2025-11-17T10:30:01Z"
}
```

Timeout scenario:
```json
{
  "decision": "REQUIRES_CONFIRMATION",
  "classification": "CREATE",
  "reasoning": "Classification timed out; defaulting to safest option",
  "confidence": 0.0,
  "explanation": "The classifier could not determine the command type within 2 seconds. Defaulting to CREATE (safest) to require confirmation.",
  "classification_method": "fallback",
  "timestamp": "2025-11-17T10:30:05Z"
}
```

#### Error Responses

**400 Bad Request** - Invalid input

```json
{
  "error": "Invalid request body",
  "code": "INVALID_JSON",
  "details": "Failed to parse JSON at line 1"
}
```

**400 Bad Request** - Missing required field

```json
{
  "error": "Missing required field: command",
  "code": "MISSING_FIELD"
}
```

**400 Bad Request** - Field too long

```json
{
  "error": "Field 'command' exceeds maximum length of 10000 characters",
  "code": "FIELD_TOO_LONG"
}
```

**408 Request Timeout** - Classification took too long

```json
{
  "error": "Classification request timed out after 2000ms",
  "code": "TIMEOUT",
  "details": "Classifier did not respond within timeout window. Defaulting to safe classification."
}
```

**429 Too Many Requests** - Rate limited

```json
{
  "error": "Rate limit exceeded: 100 requests per minute",
  "code": "RATE_LIMITED",
  "retry_after_seconds": 45
}
```

**503 Service Unavailable** - Classifier not ready

```json
{
  "error": "Classifier is not ready",
  "code": "CLASSIFIER_NOT_READY",
  "details": "Model is still loading or system is initializing"
}
```

**500 Internal Server Error** - Unexpected error

```json
{
  "error": "Internal server error",
  "code": "INTERNAL_ERROR",
  "request_id": "req_abc123xyz",
  "details": "An unexpected error occurred. Please check the daemon logs."
}
```

#### Error Response Fields

| Field | Type | Description |
|-------|------|-------------|
| error | string | Human-readable error message |
| code | string | Machine-readable error code |
| details | string | Additional context (optional) |
| retry_after_seconds | integer | For rate limits, when to retry |
| request_id | string | For 500 errors, includes request ID for debugging |

#### Rate Limits

- **Limit**: 100 requests per minute per IP
- **Window**: Rolling 60-second window
- **Response Header**: `X-RateLimit-Limit: 100`
- **Response Header**: `X-RateLimit-Remaining: 45`
- **Response Header**: `X-RateLimit-Reset: 1700223000` (Unix timestamp)

#### HTTP Status Codes

| Code | Condition | Note |
|------|-----------|------|
| 200 | Request successful | Decision returned (may require confirmation) |
| 400 | Invalid input | Client error (fix request) |
| 408 | Timeout | Fallback classification applied |
| 429 | Rate limited | Retry after indicated time |
| 503 | Not ready | Retry later |
| 500 | Server error | Check daemon logs |

---

### GET /api/hooks/decisions

Retrieve classification decisions from the audit trail.

**Purpose**: Query historical decisions for auditing, analysis, and debugging.

#### Request

**Query Parameters**:

| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| limit | integer | 50 | 1000 | Number of records to return |
| offset | integer | 0 | - | Starting position for pagination |
| classification | string | (all) | - | Filter by CRUD type |
| response | string | (all) | - | Filter by user response |
| since | ISO8601 | (all) | - | Only include decisions after this time |
| until | ISO8601 | (now) | - | Only include decisions before this time |
| command_filter | string | (all) | - | Text search in command field |

#### Request Examples

Get last 50 decisions:
```bash
GET /api/hooks/decisions
```

Get paginated results:
```bash
GET /api/hooks/decisions?limit=20&offset=40
```

Get only CREATE operations:
```bash
GET /api/hooks/decisions?classification=CREATE
```

Get denied confirmations:
```bash
GET /api/hooks/decisions?response=DENIED
```

Get decisions from last hour:
```bash
GET /api/hooks/decisions?since=2025-11-17T09:30:00Z
```

Get git-related decisions:
```bash
GET /api/hooks/decisions?command_filter=git&limit=100
```

Complex query:
```bash
GET /api/hooks/decisions?classification=DELETE&response=APPROVED&since=2025-11-17T00:00:00Z&limit=50
```

#### Response

**Success Response (200 OK)**:

```json
{
  "decisions": [
    {
      "id": 1234,
      "timestamp": "2025-11-17T10:30:00Z",
      "command": "git commit -m 'Fix bug'",
      "classification": "UPDATE",
      "decision_requires_confirmation": true,
      "user_response": "APPROVED",
      "execution_happened": true,
      "response_time_ms": 1240,
      "confidence": 0.96,
      "error": null
    },
    {
      "id": 1233,
      "timestamp": "2025-11-17T10:29:00Z",
      "command": "rm -rf /tmp/*.log",
      "classification": "DELETE",
      "decision_requires_confirmation": true,
      "user_response": "DENIED",
      "execution_happened": false,
      "response_time_ms": 0,
      "confidence": 0.99,
      "error": null
    }
  ],
  "stats": {
    "total_decisions": 5432,
    "read_operations": 3421,
    "create_operations": 892,
    "update_operations": 756,
    "delete_operations": 363,
    "approved_confirmations": 2198,
    "denied_confirmations": 287,
    "timeout_classifications": 45,
    "avg_response_time_ms": 892
  },
  "pagination": {
    "limit": 50,
    "offset": 0,
    "total": 5432,
    "returned": 50
  },
  "generated_at": "2025-11-17T10:35:00Z"
}
```

#### Response Fields

**decisions array** - List of audit records

| Field | Type | Description |
|-------|------|-------------|
| id | integer | Unique decision ID |
| timestamp | ISO8601 | When decision was made |
| command | string | The command that was classified |
| classification | string | READ, CREATE, UPDATE, or DELETE |
| decision_requires_confirmation | boolean | Whether confirmation was requested |
| user_response | string | APPROVED, DENIED, or TIMEOUT |
| execution_happened | boolean | Whether command executed |
| response_time_ms | integer | Classification latency in milliseconds |
| confidence | number | Classifier confidence (0.0-1.0) |
| error | string | Error message if classification failed |

**stats object** - Aggregate statistics

| Field | Type | Description |
|-------|------|-------------|
| total_decisions | integer | Total number of decisions |
| read_operations | integer | Count of READ classifications |
| create_operations | integer | Count of CREATE classifications |
| update_operations | integer | Count of UPDATE classifications |
| delete_operations | integer | Count of DELETE classifications |
| approved_confirmations | integer | Count of user approvals |
| denied_confirmations | integer | Count of user denials |
| timeout_classifications | integer | Count of timeout fallbacks |
| avg_response_time_ms | number | Average classification time |

**pagination object** - Pagination info

| Field | Type | Description |
|-------|------|-------------|
| limit | integer | Requested limit |
| offset | integer | Requested offset |
| total | integer | Total records available |
| returned | integer | Records returned in this response |

#### Error Responses

**400 Bad Request** - Invalid filter

```json
{
  "error": "Invalid classification filter: INVALID_TYPE",
  "code": "INVALID_FILTER",
  "valid_values": ["READ", "CREATE", "UPDATE", "DELETE"]
}
```

**400 Bad Request** - Invalid date

```json
{
  "error": "Invalid ISO8601 date format for 'since' parameter",
  "code": "INVALID_DATE",
  "example": "2025-11-17T10:30:00Z"
}
```

**500 Internal Server Error** - Database error

```json
{
  "error": "Database query failed",
  "code": "DB_ERROR",
  "request_id": "req_xyz789abc"
}
```

---

### GET /api/hooks/stats

Get aggregate statistics about all decisions.

**Purpose**: Get high-level metrics without retrieving individual decisions.

#### Request

**Query Parameters**:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| since | ISO8601 | (all time) | Only count decisions after this time |
| until | ISO8601 | (now) | Only count decisions before this time |

#### Request Examples

Get all-time statistics:
```bash
GET /api/hooks/stats
```

Get statistics for today:
```bash
GET /api/hooks/stats?since=2025-11-17T00:00:00Z
```

Get statistics for specific period:
```bash
GET /api/hooks/stats?since=2025-11-15T00:00:00Z&until=2025-11-17T23:59:59Z
```

#### Response

**Success Response (200 OK)**:

```json
{
  "total_decisions": 5432,
  "classifications": {
    "read": 3421,
    "create": 892,
    "update": 756,
    "delete": 363
  },
  "user_responses": {
    "approved": 2198,
    "denied": 287,
    "timeouts": 45
  },
  "performance": {
    "avg_response_time_ms": 892,
    "min_response_time_ms": 1,
    "max_response_time_ms": 2043,
    "p50_response_time_ms": 645,
    "p95_response_time_ms": 1852,
    "p99_response_time_ms": 1998
  },
  "confidence": {
    "avg_confidence": 0.94,
    "high_confidence_count": 5201,
    "low_confidence_count": 231
  },
  "period": {
    "since": "2025-01-01T00:00:00Z",
    "until": "2025-11-17T10:35:00Z",
    "duration_hours": 9432
  },
  "generated_at": "2025-11-17T10:35:00Z"
}
```

#### Response Fields

**classifications object** - Counts by CRUD type

| Field | Type | Description |
|-------|------|-------------|
| read | integer | READ operations classified |
| create | integer | CREATE operations classified |
| update | integer | UPDATE operations classified |
| delete | integer | DELETE operations classified |

**user_responses object** - User decision counts

| Field | Type | Description |
|-------|------|-------------|
| approved | integer | User approved confirmations |
| denied | integer | User denied confirmations |
| timeouts | integer | Classifications that timed out |

**performance object** - Performance metrics

| Field | Type | Description |
|-------|------|-------------|
| avg_response_time_ms | number | Mean classification time |
| min_response_time_ms | integer | Fastest classification |
| max_response_time_ms | integer | Slowest classification |
| p50_response_time_ms | integer | Median (50th percentile) |
| p95_response_time_ms | integer | 95th percentile |
| p99_response_time_ms | integer | 99th percentile |

**confidence object** - Classifier confidence metrics

| Field | Type | Description |
|-------|------|-------------|
| avg_confidence | number | Mean confidence score |
| high_confidence_count | integer | Classifications >= 0.9 |
| low_confidence_count | integer | Classifications < 0.8 |

**period object** - Query period info

| Field | Type | Description |
|-------|------|-------------|
| since | ISO8601 | Start of query period |
| until | ISO8601 | End of query period |
| duration_hours | integer | Period duration in hours |

---

### DELETE /api/hooks/cleanup

Delete old audit records to reclaim space.

**Purpose**: Maintenance operation to remove audit entries older than retention period.

#### Request

**Query Parameters**:

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| days | integer | no | 30 | Delete records older than N days |
| force | boolean | no | false | Force deletion without confirmation |

#### Request Examples

Delete records older than 30 days (default):
```bash
DELETE /api/hooks/cleanup
```

Delete records older than 7 days:
```bash
DELETE /api/hooks/cleanup?days=7
```

Force deletion without confirmation:
```bash
DELETE /api/hooks/cleanup?days=7&force=true
```

#### Response

**Success Response (200 OK)**:

```json
{
  "deleted": 1234,
  "remaining": 4198,
  "freed_bytes": 524288,
  "duration_ms": 234,
  "timestamp": "2025-11-17T10:35:00Z"
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| deleted | integer | Number of records deleted |
| remaining | integer | Records still in database |
| freed_bytes | integer | Approximate space freed |
| duration_ms | integer | Operation duration |
| timestamp | ISO8601 | When cleanup occurred |

#### Error Responses

**400 Bad Request** - Invalid days parameter

```json
{
  "error": "Invalid days parameter: must be between 1 and 365",
  "code": "INVALID_DAYS"
}
```

**403 Forbidden** - Requires force flag

```json
{
  "error": "Cleanup requires confirmation. Add ?force=true to proceed.",
  "code": "REQUIRES_CONFIRMATION"
}
```

---

## Common Patterns

### Example: Process All Decisions Since Last Hour

```bash
#!/bin/bash

# Get last hour in ISO8601 format
SINCE=$(date -u -d "1 hour ago" +%Y-%m-%dT%H:%M:%SZ)

# Query decisions
curl -s "http://localhost:3000/api/hooks/decisions?since=$SINCE" | jq '.decisions[] | {command, classification, user_response}'
```

Output:
```json
{
  "command": "git commit -m 'Fix'",
  "classification": "UPDATE",
  "user_response": "APPROVED"
}
{
  "command": "rm old.txt",
  "classification": "DELETE",
  "user_response": "DENIED"
}
```

### Example: Find All Denied DELETE Operations

```bash
curl -s "http://localhost:3000/api/hooks/decisions?classification=DELETE&response=DENIED" | jq '.decisions[] | {timestamp, command}'
```

### Example: Monitor Classification Performance

```bash
#!/bin/bash

while true; do
  curl -s "http://localhost:3000/api/hooks/stats" | jq '.performance | {avg: .avg_response_time_ms, p95: .p95_response_time_ms, p99: .p99_response_time_ms}'
  sleep 5
done
```

### Example: Audit All Executions Today

```bash
curl -s "http://localhost:3000/api/hooks/decisions?since=2025-11-17T00:00:00Z" | \
  jq '.decisions[] | select(.execution_happened == true) | {command, timestamp, classification}'
```

---

## Error Handling Best Practices

### Handling Rate Limits

```javascript
async function classifyWithRetry(command, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch('http://localhost:3000/api/hooks/permission-request', {
        method: 'POST',
        body: JSON.stringify({ command })
      });

      if (response.status === 429) {
        const retryAfter = response.headers.get('retry-after');
        await sleep((parseInt(retryAfter) || 60) * 1000);
        continue;
      }

      return await response.json();
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await sleep(1000 * Math.pow(2, i));
    }
  }
}
```

### Handling Timeouts

```javascript
async function classifyWithFallback(command) {
  try {
    const response = await fetch('http://localhost:3000/api/hooks/permission-request', {
      method: 'POST',
      body: JSON.stringify({ command }),
      signal: AbortSignal.timeout(5000)
    });

    const data = await response.json();

    // Check if classifier itself timed out
    if (data.classification_method === 'fallback') {
      console.warn('Classifier timed out, using safe default');
    }

    return data;
  } catch (error) {
    if (error.name === 'AbortError') {
      console.error('Request timed out, daemon may not be responsive');
      // Fall back to manual confirmation
      return { decision: 'REQUIRES_CONFIRMATION', classification: 'UNKNOWN' };
    }
    throw error;
  }
}
```

---

## Health Checks

The daemon health endpoint includes hooks status:

```bash
curl http://localhost:3000/health | jq '.hooks'
```

Response:
```json
{
  "enabled": true,
  "classifier_available": true,
  "model_loaded": false,
  "model_name": "tinyllama-1.1b-chat-v1.0.Q4_K_M",
  "classification_latency_ms": 523
}
```

---

## Deprecation Policy

API changes follow semantic versioning:

- **v1.x.x**: Current version, backward compatible
- **v2.0.0**: Breaking changes possible
- **Deprecations**: 6-month notice before removal

---

**Last Updated**: November 17, 2025
**Version**: 1.0.0
**Status**: Complete for Phases 2-5
