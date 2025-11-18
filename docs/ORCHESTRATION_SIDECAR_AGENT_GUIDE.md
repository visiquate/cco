# Orchestration Sidecar Agent Integration Guide

**Version**: 1.0.0
**Date**: November 2025
**Audience**: Developers building agents that integrate with the sidecar

## Table of Contents

1. [Overview](#overview)
2. [Agent Lifecycle](#agent-lifecycle)
3. [Getting Context](#getting-context)
4. [Storing Results](#storing-results)
5. [Publishing Events](#publishing-events)
6. [Subscribing to Events](#subscribing-to-events)
7. [Error Handling](#error-handling)
8. [Best Practices](#best-practices)
9. [Code Examples](#code-examples)

---

## Overview

This guide shows you how to build agents that integrate with the orchestration sidecar. Agents use the sidecar's HTTP API to:

1. **Get context** - Receive relevant project information
2. **Store results** - Save their work for other agents
3. **Publish events** - Notify other agents of completion
4. **Subscribe to events** - Wait for dependencies to complete

### Agent Architecture

```
┌──────────────────────────────────┐
│         Your Agent               │
│                                  │
│  ┌────────────────────────────┐  │
│  │  1. Get Context            │◄─┼─── Sidecar API
│  └────────┬───────────────────┘  │
│           │                      │
│           ▼                      │
│  ┌────────────────────────────┐  │
│  │  2. Do Work                │  │
│  │     (Implementation)       │  │
│  └────────┬───────────────────┘  │
│           │                      │
│           ▼                      │
│  ┌────────────────────────────┐  │
│  │  3. Store Results          │──┼───► Sidecar API
│  └────────┬───────────────────┘  │
│           │                      │
│           ▼                      │
│  ┌────────────────────────────┐  │
│  │  4. Publish Event          │──┼───► Event Bus
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

---

## Agent Lifecycle

### 1. Agent Spawn

When the sidecar spawns an agent, it:

1. Generates a JWT token
2. Injects environment variables
3. Launches the agent process
4. Provides a context URL

**Environment Variables Injected:**

```bash
AGENT_ID="python-specialist-uuid"
AGENT_TYPE="python-specialist"
ISSUE_ID="issue-123"
PROJECT_ID="project-abc"
SIDECAR_URL="http://localhost:3001/api"
JWT_TOKEN="eyJhbGciOiJIUzI1NiIs..."
```

### 2. Agent Initialization

Your agent should initialize by:

```python
import os

# Read environment variables
agent_id = os.getenv("AGENT_ID")
agent_type = os.getenv("AGENT_TYPE")
issue_id = os.getenv("ISSUE_ID")
project_id = os.getenv("PROJECT_ID")
sidecar_url = os.getenv("SIDECAR_URL")
jwt_token = os.getenv("JWT_TOKEN")

# Validate required variables
if not all([agent_id, agent_type, issue_id, sidecar_url, jwt_token]):
    raise ValueError("Missing required environment variables")

# Setup HTTP client
headers = {"Authorization": f"Bearer {jwt_token}"}
```

### 3. Get Context

Request relevant context from the sidecar:

```python
import requests

context_url = f"{sidecar_url}/context/{issue_id}/{agent_type}"
response = requests.get(context_url, headers=headers)

if response.status_code == 200:
    context = response.json()
    print(f"Received {len(context['context']['relevant_files'])} files")
else:
    print(f"Failed to get context: {response.status_code}")
    exit(1)
```

### 4. Execute Work

Use the context to perform your task:

```python
# Extract relevant information
files = context["context"]["relevant_files"]
previous_decisions = context["context"]["previous_agent_outputs"]
metadata = context["context"]["metadata"]

# Implement your agent logic
results = implement_feature(files, previous_decisions)
```

### 5. Store Results

Save your work to the sidecar:

```python
result_data = {
    "agent_id": agent_id,
    "agent_type": agent_type,
    "issue_id": issue_id,
    "project_id": project_id,
    "result": {
        "status": "success",
        "files_created": results["files"],
        "decisions": results["decisions"],
        "metrics": results["metrics"]
    }
}

response = requests.post(
    f"{sidecar_url}/results",
    json=result_data,
    headers=headers
)

if response.status_code == 201:
    print(f"Results stored: {response.json()['id']}")
else:
    print(f"Failed to store results: {response.status_code}")
    exit(1)
```

### 6. Publish Completion Event

Notify other agents:

```python
event_data = {
    "event_type": "agent_completed",
    "publisher": agent_id,
    "topic": "implementation",
    "data": {
        "issue_id": issue_id,
        "status": "success",
        "agent_type": agent_type
    }
}

response = requests.post(
    f"{sidecar_url}/events/agent_completed",
    json=event_data,
    headers=headers
)

if response.status_code == 202:
    print(f"Event published: {response.json()['event_id']}")
```

---

## Getting Context

### What Context Contains

The context object includes:

1. **Project Structure** - Files and directories
2. **Relevant Files** - Files specific to your agent type
3. **Git Context** - Commits, branches, uncommitted changes
4. **Previous Agent Outputs** - Results from other agents
5. **Metadata** - Project type, dependencies, test coverage

### Context by Agent Type

Different agent types receive different context:

#### Architecture Agents (Chief Architect)

```python
# Full project structure
context["context"]["project_structure"]  # All files and directories
context["context"]["relevant_files"]     # All source files
context["context"]["git_context"]        # Full git history
```

#### Coding Specialists (Python, Go, Rust, etc.)

```python
# Language-specific files only
context["context"]["relevant_files"]  # *.py, *.go, *.rs
# Test files for the module
# Dependencies and imports
# Previous code reviews
```

#### QA/Security Agents

```python
# All test files
context["context"]["relevant_files"]  # test_*.py, *_test.go
# Security policies
# Previous vulnerabilities
# Test coverage reports
```

#### Documentation Agents

```python
# Documentation files
context["context"]["relevant_files"]  # README.md, docs/*.md
# API specifications
# Previous documentation
# Code comments
```

### Requesting Specific Context

You can request additional context requirements:

```python
# When spawning an agent (done by orchestrator)
spawn_data = {
    "agent_type": "python-specialist",
    "issue_id": "issue-123",
    "context_requirements": [
        "project_structure",
        "previous_decisions",
        "security_policies",
        "test_coverage_reports"
    ]
}
```

### Handling Large Context

If context is truncated, the response indicates this:

```python
context = get_context(issue_id, agent_type)

if context["truncated"]:
    strategy = context["truncation_strategy"]
    print(f"Context truncated using strategy: {strategy}")

    # Request specific files if needed
    # (not yet implemented - future enhancement)
```

---

## Storing Results

### Result Schema

Your results should follow this structure:

```python
result = {
    "status": "success",  # or "failed", "partial"

    "files_created": [
        {
            "path": "src/api.py",
            "size": 4096,
            "language": "python",
            "checksum": "sha256:abc123..."
        }
    ],

    "files_modified": [
        {
            "path": "requirements.txt",
            "diff": "+1 -0",
            "changes": ["Added fastapi==0.104.0"]
        }
    ],

    "files_deleted": ["old/deprecated.py"],

    "decisions": [
        "Implemented REST API with FastAPI",
        "Added comprehensive test suite",
        "Used Pydantic models for validation"
    ],

    "metrics": {
        "execution_time_ms": 4500,
        "tokens_used": 15000,
        "test_coverage": 92.3,
        "lines_of_code": 250,
        "complexity_score": 3.2
    },

    "artifacts": {
        "api_documentation": "# API Docs...",
        "test_report": "All tests passing",
        "performance_report": "API < 50ms"
    },

    "errors": [],

    "warnings": [
        "Consider adding rate limiting",
        "Missing OpenAPI tags"
    ],

    "next_steps": [
        "Security audit required",
        "Load testing recommended"
    ]
}
```

### Success vs. Failure Results

**Success:**

```python
result = {
    "status": "success",
    "files_created": ["src/feature.py"],
    "decisions": ["Feature implemented successfully"],
    "metrics": {"test_coverage": 95.0}
}
```

**Partial Success:**

```python
result = {
    "status": "partial",
    "files_created": ["src/feature.py"],
    "decisions": ["Basic implementation complete"],
    "warnings": ["Advanced features pending"],
    "next_steps": ["Complete edge case handling"]
}
```

**Failure:**

```python
result = {
    "status": "failed",
    "errors": [
        "Cannot implement: missing dependency X",
        "Conflicting requirement Y detected"
    ],
    "next_steps": [
        "Resolve dependency conflict",
        "Consult Chief Architect"
    ]
}
```

### Next Agent Suggestions

The sidecar automatically suggests next agents based on your results:

```python
response = store_results(result_data)

# Sidecar responds with next agents
next_agents = response["next_agents"]
# [
#   {"agent_type": "test-engineer", "priority": "high"},
#   {"agent_type": "security-auditor", "priority": "medium"}
# ]
```

---

## Publishing Events

### Event Topics

Publish events to these standard topics:

| Topic | When to Use | Subscribers |
|-------|-------------|-------------|
| `architecture` | Design decisions made | All coding agents |
| `implementation` | Code completed | QA, Security, DevOps |
| `testing` | Tests written/passing | DevOps, Architect |
| `security` | Security findings | Architect, DevOps |
| `deployment` | Deployment complete | All agents |
| `documentation` | Docs updated | Architect |
| `error` | Errors encountered | Architect, DevOps |

### Publishing Examples

**Architecture Decision:**

```python
event_data = {
    "event_type": "architecture_defined",
    "publisher": agent_id,
    "topic": "architecture",
    "data": {
        "issue_id": issue_id,
        "decision": "Use microservices architecture",
        "rationale": "Better scalability",
        "services": ["api", "worker", "database"]
    },
    "ttl_seconds": 86400
}

publish_event(event_data)
```

**Implementation Complete:**

```python
event_data = {
    "event_type": "implementation_complete",
    "publisher": agent_id,
    "topic": "implementation",
    "data": {
        "issue_id": issue_id,
        "agent_type": agent_type,
        "files_changed": 5,
        "next_phase": "testing"
    }
}

publish_event(event_data)
```

**Error Encountered:**

```python
event_data = {
    "event_type": "error_encountered",
    "publisher": agent_id,
    "topic": "error",
    "data": {
        "issue_id": issue_id,
        "error": "Dependency conflict detected",
        "severity": "high",
        "retryable": False
    }
}

publish_event(event_data)
```

### Event Correlation

Link related events using `correlation_id`:

```python
# First event
event1 = {
    "event_type": "review_requested",
    "publisher": "code-reviewer",
    "correlation_id": "review-session-123",
    "data": {"findings": ["Missing error handling"]}
}

# Response event
event2 = {
    "event_type": "review_addressed",
    "publisher": "python-specialist",
    "correlation_id": "review-session-123",  # Same ID
    "data": {"changes": ["Added error handling"]}
}
```

---

## Subscribing to Events

### Long Polling Pattern

Subscribe to events using long polling:

```python
def wait_for_event(event_type, timeout=30000, filter=None):
    """Wait for a specific event type"""
    params = {"timeout": timeout}
    if filter:
        params["filter"] = filter

    response = requests.get(
        f"{sidecar_url}/events/wait/{event_type}",
        params=params,
        headers=headers,
        timeout=timeout/1000 + 5  # Add buffer
    )

    return response.json()

# Wait for architecture decisions
events = wait_for_event(
    "architecture_defined",
    timeout=60000,
    filter=f"issue_id:{issue_id}"
)

if events["events"]:
    for event in events["events"]:
        handle_architecture_event(event)
```

### Continuous Subscription

Loop to continuously listen:

```python
def subscribe_continuously(event_type, handler, filter=None):
    """Subscribe to events continuously"""
    while True:
        try:
            result = wait_for_event(event_type, timeout=30000, filter=filter)

            if result.get("timeout"):
                # No events, retry
                continue

            for event in result.get("events", []):
                handler(event)

        except Exception as e:
            print(f"Subscription error: {e}")
            time.sleep(5)  # Backoff before retry
```

### Event Filtering

Filter events by attributes:

```python
# Filter by issue_id
wait_for_event("agent_completed", filter="issue_id:issue-123")

# Filter by status
wait_for_event("testing_complete", filter="status:success")

# Multiple filters (AND logic)
wait_for_event("deployment", filter="issue_id:issue-123,status:success")
```

### Background Subscriptions

Run subscriptions in background threads:

```python
import threading

def background_event_listener():
    """Listen for events in background"""
    def handle_event(event):
        print(f"Event received: {event['event_type']}")

    subscribe_continuously("agent_completed", handle_event)

# Start listener thread
listener_thread = threading.Thread(
    target=background_event_listener,
    daemon=True
)
listener_thread.start()

# Main work continues
do_work()
```

---

## Error Handling

### Authentication Errors

```python
def get_context_with_retry(issue_id, agent_type, max_retries=3):
    """Get context with automatic token refresh"""
    for attempt in range(max_retries):
        try:
            response = requests.get(
                f"{sidecar_url}/context/{issue_id}/{agent_type}",
                headers=headers
            )

            if response.status_code == 401:
                # Token expired, refresh
                refresh_jwt_token()
                continue

            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            if attempt == max_retries - 1:
                raise
            time.sleep(2 ** attempt)  # Exponential backoff
```

### Rate Limiting

```python
def handle_rate_limit(response):
    """Handle rate limit responses"""
    if response.status_code == 429:
        retry_after = response.headers.get("Retry-After", 30)
        print(f"Rate limited. Waiting {retry_after} seconds...")
        time.sleep(int(retry_after))
        return True
    return False

def api_call_with_rate_limit(url, **kwargs):
    """Make API call with rate limit handling"""
    while True:
        response = requests.get(url, **kwargs)

        if not handle_rate_limit(response):
            return response
```

### Network Errors

```python
from requests.exceptions import ConnectionError, Timeout

def robust_api_call(url, max_retries=3, **kwargs):
    """API call with network error handling"""
    for attempt in range(max_retries):
        try:
            response = requests.get(url, timeout=10, **kwargs)
            response.raise_for_status()
            return response.json()

        except (ConnectionError, Timeout) as e:
            if attempt == max_retries - 1:
                print(f"Failed after {max_retries} attempts: {e}")
                raise

            wait_time = 2 ** attempt
            print(f"Network error, retrying in {wait_time}s...")
            time.sleep(wait_time)

        except requests.exceptions.HTTPError as e:
            print(f"HTTP error: {e.response.status_code}")
            raise
```

### Sidecar Unavailable

```python
def check_sidecar_health():
    """Check if sidecar is healthy"""
    try:
        response = requests.get(
            "http://localhost:3001/health",
            timeout=5
        )
        return response.status_code == 200
    except:
        return False

def wait_for_sidecar(timeout=60):
    """Wait for sidecar to become available"""
    start = time.time()
    while time.time() - start < timeout:
        if check_sidecar_health():
            print("Sidecar is healthy")
            return True

        print("Waiting for sidecar...")
        time.sleep(5)

    return False

# At agent startup
if not wait_for_sidecar():
    print("Sidecar unavailable, exiting")
    exit(1)
```

---

## Best Practices

### 1. Always Validate Context

```python
def validate_context(context):
    """Validate context before use"""
    required_keys = ["project_structure", "relevant_files", "metadata"]

    for key in required_keys:
        if key not in context["context"]:
            raise ValueError(f"Missing required context: {key}")

    if not context["context"]["relevant_files"]:
        raise ValueError("No relevant files in context")

    return True
```

### 2. Use Correlation IDs

```python
import uuid

# Generate correlation ID for related operations
correlation_id = str(uuid.uuid4())

# Use in all related events
event1 = {
    "correlation_id": correlation_id,
    # ...
}

event2 = {
    "correlation_id": correlation_id,
    # ...
}
```

### 3. Log All Interactions

```python
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Log context retrieval
logger.info(f"Getting context for {issue_id}/{agent_type}")
context = get_context(issue_id, agent_type)
logger.info(f"Received {len(context['context']['relevant_files'])} files")

# Log results storage
logger.info(f"Storing results for {agent_id}")
result = store_results(result_data)
logger.info(f"Results stored: {result['id']}")
```

### 4. Handle Partial Context

```python
def handle_truncated_context(context):
    """Handle truncated context gracefully"""
    if context["truncated"]:
        strategy = context["truncation_strategy"]
        logger.warning(f"Context truncated using {strategy}")

        # Work with available context
        # Request critical files if missing
        # Or fail gracefully
```

### 5. Provide Detailed Results

```python
# Good: Detailed results
result = {
    "status": "success",
    "files_created": ["src/api.py", "tests/test_api.py"],
    "decisions": [
        "Implemented REST API with FastAPI",
        "Used Pydantic for validation",
        "Added comprehensive error handling"
    ],
    "metrics": {
        "test_coverage": 95.0,
        "execution_time_ms": 4500,
        "lines_of_code": 250
    }
}

# Bad: Minimal results
result = {
    "status": "success"
}
```

### 6. Clean Up Resources

```python
import atexit

def cleanup():
    """Clean up resources on exit"""
    # Publish final status
    publish_event({
        "event_type": "agent_shutdown",
        "publisher": agent_id,
        "topic": "system",
        "data": {"reason": "normal_exit"}
    })

    # Clear temporary files
    # Close connections
    # etc.

# Register cleanup handler
atexit.register(cleanup)
```

---

## Code Examples

### Complete Python Agent

```python
#!/usr/bin/env python3
"""
Python Specialist Agent
Integrates with orchestration sidecar
"""

import os
import sys
import requests
import logging
from typing import Dict, Any

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class PythonSpecialistAgent:
    def __init__(self):
        """Initialize agent from environment"""
        self.agent_id = os.getenv("AGENT_ID")
        self.agent_type = os.getenv("AGENT_TYPE")
        self.issue_id = os.getenv("ISSUE_ID")
        self.project_id = os.getenv("PROJECT_ID")
        self.sidecar_url = os.getenv("SIDECAR_URL")
        self.jwt_token = os.getenv("JWT_TOKEN")

        if not all([self.agent_id, self.issue_id, self.sidecar_url, self.jwt_token]):
            raise ValueError("Missing required environment variables")

        self.headers = {"Authorization": f"Bearer {self.jwt_token}"}

    def get_context(self) -> Dict[str, Any]:
        """Get context from sidecar"""
        logger.info(f"Getting context for {self.issue_id}")

        url = f"{self.sidecar_url}/context/{self.issue_id}/{self.agent_type}"
        response = requests.get(url, headers=self.headers)
        response.raise_for_status()

        context = response.json()
        logger.info(f"Received {len(context['context']['relevant_files'])} files")

        return context

    def do_work(self, context: Dict[str, Any]) -> Dict[str, Any]:
        """Implement the feature"""
        logger.info("Implementing feature...")

        # Your implementation logic here
        # Use context["context"]["relevant_files"]
        # Use context["context"]["previous_agent_outputs"]

        return {
            "status": "success",
            "files_created": ["src/feature.py", "tests/test_feature.py"],
            "decisions": ["Implemented feature X"],
            "metrics": {
                "execution_time_ms": 5000,
                "test_coverage": 95.0
            }
        }

    def store_results(self, result: Dict[str, Any]):
        """Store results to sidecar"""
        logger.info("Storing results...")

        result_data = {
            "agent_id": self.agent_id,
            "agent_type": self.agent_type,
            "issue_id": self.issue_id,
            "project_id": self.project_id,
            "result": result
        }

        url = f"{self.sidecar_url}/results"
        response = requests.post(url, json=result_data, headers=self.headers)
        response.raise_for_status()

        result_id = response.json()["id"]
        logger.info(f"Results stored: {result_id}")

    def publish_event(self, event_type: str, data: Dict[str, Any]):
        """Publish event"""
        logger.info(f"Publishing event: {event_type}")

        event_data = {
            "event_type": event_type,
            "publisher": self.agent_id,
            "topic": "implementation",
            "data": data
        }

        url = f"{self.sidecar_url}/events/{event_type}"
        response = requests.post(url, json=event_data, headers=self.headers)
        response.raise_for_status()

        event_id = response.json()["event_id"]
        logger.info(f"Event published: {event_id}")

    def run(self):
        """Main agent workflow"""
        try:
            # 1. Get context
            context = self.get_context()

            # 2. Do work
            result = self.do_work(context)

            # 3. Store results
            self.store_results(result)

            # 4. Publish completion event
            self.publish_event("implementation_complete", {
                "issue_id": self.issue_id,
                "status": result["status"]
            })

            logger.info("Agent completed successfully")
            return 0

        except Exception as e:
            logger.error(f"Agent failed: {e}")

            # Publish error event
            try:
                self.publish_event("error_encountered", {
                    "issue_id": self.issue_id,
                    "error": str(e),
                    "agent_type": self.agent_type
                })
            except:
                pass

            return 1

if __name__ == "__main__":
    agent = PythonSpecialistAgent()
    sys.exit(agent.run())
```

### Complete Rust Agent

```rust
//! Rust Specialist Agent
//! Integrates with orchestration sidecar

use anyhow::{Context, Result};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct AgentContext {
    issue_id: String,
    agent_type: String,
    context: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentResult {
    status: String,
    files_created: Vec<String>,
    decisions: Vec<String>,
    metrics: Value,
}

struct RustSpecialistAgent {
    agent_id: String,
    agent_type: String,
    issue_id: String,
    project_id: String,
    sidecar_url: String,
    client: Client,
    headers: HeaderMap,
}

impl RustSpecialistAgent {
    fn new() -> Result<Self> {
        let agent_id = env::var("AGENT_ID")?;
        let agent_type = env::var("AGENT_TYPE")?;
        let issue_id = env::var("ISSUE_ID")?;
        let project_id = env::var("PROJECT_ID")?;
        let sidecar_url = env::var("SIDECAR_URL")?;
        let jwt_token = env::var("JWT_TOKEN")?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", jwt_token))?
        );

        Ok(Self {
            agent_id,
            agent_type,
            issue_id,
            project_id,
            sidecar_url,
            client: Client::new(),
            headers,
        })
    }

    async fn get_context(&self) -> Result<AgentContext> {
        let url = format!("{}/context/{}/{}",
            self.sidecar_url, self.issue_id, self.agent_type);

        let response = self.client
            .get(&url)
            .headers(self.headers.clone())
            .send()
            .await?;

        response.error_for_status_ref()?;

        let context: AgentContext = response.json().await?;
        Ok(context)
    }

    async fn do_work(&self, _context: &AgentContext) -> Result<AgentResult> {
        // Your implementation logic here
        Ok(AgentResult {
            status: "success".to_string(),
            files_created: vec!["src/main.rs".to_string()],
            decisions: vec!["Implemented feature".to_string()],
            metrics: json!({
                "execution_time_ms": 5000,
                "test_coverage": 95.0
            }),
        })
    }

    async fn store_results(&self, result: &AgentResult) -> Result<()> {
        let result_data = json!({
            "agent_id": self.agent_id,
            "agent_type": self.agent_type,
            "issue_id": self.issue_id,
            "project_id": self.project_id,
            "result": result
        });

        let url = format!("{}/results", self.sidecar_url);

        let response = self.client
            .post(&url)
            .headers(self.headers.clone())
            .json(&result_data)
            .send()
            .await?;

        response.error_for_status_ref()?;
        Ok(())
    }

    async fn publish_event(&self, event_type: &str, data: Value) -> Result<()> {
        let event_data = json!({
            "event_type": event_type,
            "publisher": self.agent_id,
            "topic": "implementation",
            "data": data
        });

        let url = format!("{}/events/{}", self.sidecar_url, event_type);

        let response = self.client
            .post(&url)
            .headers(self.headers.clone())
            .json(&event_data)
            .send()
            .await?;

        response.error_for_status_ref()?;
        Ok(())
    }

    async fn run(&self) -> Result<()> {
        // 1. Get context
        let context = self.get_context().await
            .context("Failed to get context")?;

        // 2. Do work
        let result = self.do_work(&context).await
            .context("Failed to do work")?;

        // 3. Store results
        self.store_results(&result).await
            .context("Failed to store results")?;

        // 4. Publish event
        self.publish_event("implementation_complete", json!({
            "issue_id": self.issue_id,
            "status": result.status
        })).await
            .context("Failed to publish event")?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let agent = RustSpecialistAgent::new()?;
    agent.run().await?;

    Ok(())
}
```

---

## Next Steps

- **Learn the Event System**: [ORCHESTRATION_SIDECAR_EVENTS.md](ORCHESTRATION_SIDECAR_EVENTS.md)
- **CLI Reference**: [ORCHESTRATION_SIDECAR_CLI_REFERENCE.md](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md)
- **API Reference**: [ORCHESTRATION_SIDECAR_API_REFERENCE.md](ORCHESTRATION_SIDECAR_API_REFERENCE.md)
- **Troubleshooting**: [ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md](ORCHESTRATION_SIDECAR_TROUBLESHOOTING.md)
