# Orchestration Sidecar Event System Guide

**Version**: 1.0.0
**Date**: November 2025
**Audience**: Developers using the event coordination system

## Table of Contents

1. [Overview](#overview)
2. [Event Topics](#event-topics)
3. [Event Types](#event-types)
4. [Publishing Events](#publishing-events)
5. [Subscribing to Events](#subscribing-to-events)
6. [Event Filtering](#event-filtering)
7. [Multi-Round Workflows](#multi-round-workflows)
8. [Event Patterns](#event-patterns)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

---

## Overview

The orchestration sidecar provides a publish-subscribe (pub-sub) event system that enables agents to coordinate asynchronously without direct communication.

### Key Features

- **Topic-based routing** - Events organized by functional topics
- **Long polling** - Efficient event delivery without constant polling
- **Event filtering** - Subscribe to specific event types or attributes
- **Event retention** - 24-hour retention with circular buffer (10,000 events)
- **Correlation** - Link related events with correlation IDs
- **Guaranteed delivery** - Events delivered to all active subscribers

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Event Bus                            │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │  Topic:  │  │  Topic:  │  │  Topic:  │            │
│  │   arch   │  │   impl   │  │  testing │            │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘            │
│       │             │             │                   │
│  ┌────▼─────────────▼─────────────▼────┐             │
│  │        Event Queue (Circular)        │             │
│  │        (Last 10,000 events)          │             │
│  └──────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────┘
           │                    │
    ┌──────▼──────┐      ┌─────▼──────┐
    │  Publisher  │      │ Subscriber │
    │   Agents    │      │   Agents   │
    └─────────────┘      └────────────┘
```

---

## Event Topics

Events are organized into functional topics that determine routing and subscribers.

### Standard Topics

| Topic | Purpose | Publishers | Subscribers |
|-------|---------|------------|-------------|
| `architecture` | Design decisions | Chief Architect, Backend Architect | All coding agents |
| `implementation` | Code completed | Coding specialists | QA, Security, DevOps |
| `testing` | Test results | QA agents | DevOps, Architect |
| `security` | Security findings | Security auditor | Architect, DevOps |
| `deployment` | Deployment status | DevOps | All agents |
| `documentation` | Docs updated | Doc agents | Architect |
| `error` | Error reporting | All agents | Architect, DevOps |
| `coordination` | Agent coordination | All agents | All agents |

### Topic Subscription Rules

Agents automatically subscribe to topics based on their type:

```json
{
  "python-specialist": {
    "subscribes_to": ["architecture", "coordination", "error"]
  },
  "test-engineer": {
    "subscribes_to": ["implementation", "coordination", "error"]
  },
  "security-auditor": {
    "subscribes_to": ["implementation", "coordination", "error"]
  },
  "devops-engineer": {
    "subscribes_to": ["testing", "security", "coordination", "error"]
  },
  "chief-architect": {
    "subscribes_to": ["all"]
  }
}
```

### Custom Topics

You can create custom topics for project-specific coordination:

```python
# Publish to custom topic
event_data = {
    "event_type": "custom_checkpoint",
    "topic": "project_specific",
    "data": {"checkpoint": "phase_1_complete"}
}
```

---

## Event Types

### Standard Event Types

#### Architecture Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `architecture_defined` | Architecture decisions made | `design`, `rationale`, `components` |
| `design_updated` | Design changed | `changes`, `reason` |
| `technology_selected` | Tech stack chosen | `technologies`, `rationale` |

#### Implementation Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `implementation_started` | Implementation began | `agent_type`, `estimated_duration_ms` |
| `implementation_complete` | Implementation done | `files_changed`, `next_phase` |
| `code_review_requested` | Review needed | `files`, `priority` |

#### Testing Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `tests_written` | Tests created | `test_count`, `coverage` |
| `tests_passing` | All tests pass | `coverage`, `duration_ms` |
| `tests_failing` | Tests failed | `failures`, `errors` |

#### Security Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `security_audit_started` | Audit began | `scope` |
| `security_audit_complete` | Audit done | `issues`, `severity` |
| `vulnerability_found` | Vuln detected | `vulnerability`, `severity`, `remediation` |

#### Deployment Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `deployment_started` | Deploy began | `environment`, `version` |
| `deployment_complete` | Deploy done | `environment`, `url` |
| `deployment_failed` | Deploy failed | `error`, `rollback_status` |

#### Error Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `error_encountered` | Error occurred | `error`, `severity`, `retryable` |
| `agent_failed` | Agent crashed | `agent_type`, `error` |

#### Coordination Events

| Event Type | Description | Data Fields |
|-----------|-------------|-------------|
| `agent_spawned` | New agent started | `agent_type`, `task` |
| `agent_completed` | Agent finished | `agent_type`, `status` |
| `phase_complete` | Workflow phase done | `phase`, `next_phase` |
| `review_requested` | Review needed | `reviewer`, `target` |
| `review_complete` | Review done | `findings`, `approved` |

---

## Publishing Events

### Basic Event Publishing

```python
import requests

def publish_event(event_type, topic, data):
    """Publish an event to the event bus"""
    event_data = {
        "event_type": event_type,
        "publisher": agent_id,  # From environment
        "topic": topic,
        "data": data
    }

    response = requests.post(
        f"{sidecar_url}/events/{event_type}",
        json=event_data,
        headers=headers
    )

    if response.status_code == 202:
        event_id = response.json()["event_id"]
        print(f"Event published: {event_id}")
        return event_id
    else:
        raise Exception(f"Failed to publish event: {response.status_code}")

# Usage
event_id = publish_event(
    event_type="implementation_complete",
    topic="implementation",
    data={
        "issue_id": "issue-123",
        "files_changed": 3,
        "next_phase": "testing"
    }
)
```

### Event with TTL

Set a custom time-to-live for events:

```python
event_data = {
    "event_type": "temporary_status",
    "publisher": agent_id,
    "topic": "coordination",
    "data": {"status": "in_progress"},
    "ttl_seconds": 3600  # Expire after 1 hour
}
```

### Event with Correlation ID

Link related events:

```python
import uuid

# Initial event
correlation_id = str(uuid.uuid4())

event1 = {
    "event_type": "review_requested",
    "publisher": "code-reviewer",
    "topic": "implementation",
    "correlation_id": correlation_id,
    "data": {
        "findings": ["Missing error handling in api.py"],
        "severity": "medium"
    }
}

# Response event
event2 = {
    "event_type": "review_addressed",
    "publisher": "python-specialist",
    "topic": "implementation",
    "correlation_id": correlation_id,  # Same ID
    "data": {
        "changes": ["Added comprehensive error handling"],
        "ready_for_re_review": True
    }
}
```

### Event Priority

While not explicitly supported, you can use custom data fields:

```python
event_data = {
    "event_type": "implementation_complete",
    "publisher": agent_id,
    "topic": "implementation",
    "data": {
        "issue_id": "issue-123",
        "priority": "high",  # Custom field
        "requires_immediate_review": True
    }
}
```

---

## Subscribing to Events

### Long Polling Subscription

```python
def wait_for_event(event_type, timeout=30000, filter_expr=None):
    """Wait for an event using long polling"""
    params = {"timeout": timeout}
    if filter_expr:
        params["filter"] = filter_expr

    response = requests.get(
        f"{sidecar_url}/events/wait/{event_type}",
        params=params,
        headers=headers,
        timeout=timeout/1000 + 5  # Add buffer
    )

    result = response.json()

    if result.get("timeout"):
        return None

    return result.get("events", [])

# Usage
events = wait_for_event(
    event_type="implementation_complete",
    timeout=60000,
    filter_expr="issue_id:issue-123"
)

if events:
    for event in events:
        print(f"Received: {event['event_type']}")
        handle_event(event)
```

### Continuous Subscription

```python
import time

def subscribe_continuously(event_type, handler, filter_expr=None):
    """Subscribe to events continuously"""
    print(f"Subscribing to {event_type}...")

    while True:
        try:
            events = wait_for_event(event_type, timeout=30000, filter_expr=filter_expr)

            if events:
                for event in events:
                    handler(event)

        except KeyboardInterrupt:
            print("Subscription stopped")
            break

        except Exception as e:
            print(f"Subscription error: {e}")
            time.sleep(5)  # Backoff

# Usage
def handle_implementation_event(event):
    print(f"Implementation by {event['publisher']} complete")
    # Process event...

subscribe_continuously(
    event_type="implementation_complete",
    handler=handle_implementation_event,
    filter_expr="issue_id:issue-123"
)
```

### Background Subscription

```python
import threading

def background_subscriber(event_type, handler, filter_expr=None):
    """Run subscription in background thread"""
    thread = threading.Thread(
        target=subscribe_continuously,
        args=(event_type, handler, filter_expr),
        daemon=True
    )
    thread.start()
    return thread

# Usage
def handle_error(event):
    print(f"Error: {event['data']['error']}")
    # Handle error...

# Start background listener
error_thread = background_subscriber(
    event_type="error_encountered",
    handler=handle_error
)

# Main work continues
do_main_work()

# Wait for background thread if needed
error_thread.join()
```

### Multiple Event Types

```python
def wait_for_any_event(event_types, timeout=30000):
    """Wait for any of multiple event types"""
    # Note: Current API requires separate calls
    # Future enhancement: single call for multiple types

    import concurrent.futures

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = {
            executor.submit(wait_for_event, et, timeout): et
            for et in event_types
        }

        done, _ = concurrent.futures.wait(
            futures,
            return_when=concurrent.futures.FIRST_COMPLETED
        )

        for future in done:
            events = future.result()
            if events:
                return events

    return None

# Usage
events = wait_for_any_event([
    "implementation_complete",
    "error_encountered"
], timeout=60000)
```

---

## Event Filtering

### Filter Syntax

Filters use the format: `field:value`

Multiple filters are ANDed together: `field1:value1,field2:value2`

### Common Filters

```python
# Filter by issue_id
filter = "issue_id:issue-123"

# Filter by status
filter = "status:success"

# Filter by agent_type
filter = "agent_type:python-specialist"

# Multiple filters (AND logic)
filter = "issue_id:issue-123,status:success"

# Filter by custom field
filter = "priority:high"
```

### Filter Examples

```python
# Wait for successful implementations only
events = wait_for_event(
    "implementation_complete",
    filter_expr="status:success"
)

# Wait for high-priority events
events = wait_for_event(
    "security_audit_complete",
    filter_expr="severity:high"
)

# Wait for specific agent's events
events = wait_for_event(
    "agent_completed",
    filter_expr="agent_type:python-specialist,issue_id:issue-123"
)
```

### Advanced Filtering (Future Enhancement)

```python
# Not yet implemented - planned features

# OR logic
filter = "status:success|status:partial"

# Negation
filter = "!status:failed"

# Pattern matching
filter = "issue_id:issue-*"

# Numeric comparison
filter = "coverage:>90"
```

---

## Multi-Round Workflows

### Feedback Loop Pattern

```python
def feedback_loop(initial_work_fn, review_fn, max_rounds=3):
    """Implement multi-round feedback loop"""
    correlation_id = str(uuid.uuid4())

    for round_num in range(1, max_rounds + 1):
        print(f"Round {round_num}")

        # Do work
        result = initial_work_fn()

        # Publish work completion
        publish_event(
            event_type="work_complete",
            topic="implementation",
            data={
                "round": round_num,
                "result": result,
                "correlation_id": correlation_id
            }
        )

        # Wait for review
        review_events = wait_for_event(
            event_type="review_complete",
            filter_expr=f"correlation_id:{correlation_id}",
            timeout=60000
        )

        if not review_events:
            print("Review timeout")
            break

        review = review_events[0]["data"]

        if review["approved"]:
            print("Work approved!")
            break

        # Address feedback
        print(f"Feedback: {review['findings']}")
        # Implement changes based on review...

# Usage
def do_implementation():
    # Your implementation
    return {"files": ["api.py"]}

def review_code():
    # Your review logic
    return {"approved": True}

feedback_loop(do_implementation, review_code)
```

### Phased Workflow Pattern

```python
def phased_workflow(phases):
    """Execute multi-phase workflow with event coordination"""
    correlation_id = str(uuid.uuid4())

    for phase_num, phase_config in enumerate(phases, 1):
        phase_name = phase_config["name"]
        phase_fn = phase_config["function"]
        next_agents = phase_config.get("next_agents", [])

        print(f"Phase {phase_num}: {phase_name}")

        # Execute phase
        result = phase_fn()

        # Publish phase completion
        publish_event(
            event_type="phase_complete",
            topic="coordination",
            data={
                "phase": phase_name,
                "phase_number": phase_num,
                "result": result,
                "next_agents": next_agents,
                "correlation_id": correlation_id
            }
        )

        # Wait for next agents to complete
        if next_agents:
            for agent_type in next_agents:
                print(f"Waiting for {agent_type}...")
                wait_for_event(
                    event_type="agent_completed",
                    filter_expr=f"agent_type:{agent_type},correlation_id:{correlation_id}",
                    timeout=120000
                )

# Usage
phases = [
    {
        "name": "architecture",
        "function": lambda: design_architecture(),
        "next_agents": ["python-specialist", "go-specialist"]
    },
    {
        "name": "implementation",
        "function": lambda: implement_code(),
        "next_agents": ["test-engineer"]
    },
    {
        "name": "testing",
        "function": lambda: run_tests(),
        "next_agents": ["security-auditor"]
    }
]

phased_workflow(phases)
```

### Dependency Chain Pattern

```python
def wait_for_dependencies(dependencies, timeout=300000):
    """Wait for all dependencies to complete"""
    print(f"Waiting for {len(dependencies)} dependencies...")

    completed = set()

    while len(completed) < len(dependencies):
        remaining = [d for d in dependencies if d not in completed]

        events = wait_for_event(
            event_type="agent_completed",
            timeout=30000
        )

        if not events:
            continue

        for event in events:
            agent_type = event["data"].get("agent_type")
            if agent_type in remaining:
                completed.add(agent_type)
                print(f"✓ {agent_type} completed")

    print("All dependencies satisfied")

# Usage
# Wait for prerequisites before starting work
wait_for_dependencies([
    "chief-architect",
    "python-specialist",
    "database-architect"
])

# Now do work
do_work()
```

---

## Event Patterns

### Request-Response Pattern

```python
def request_response(request_event_type, response_event_type, request_data, timeout=60000):
    """Implement request-response pattern"""
    correlation_id = str(uuid.uuid4())

    # Publish request
    publish_event(
        event_type=request_event_type,
        topic="coordination",
        data={**request_data, "correlation_id": correlation_id}
    )

    # Wait for response
    responses = wait_for_event(
        event_type=response_event_type,
        filter_expr=f"correlation_id:{correlation_id}",
        timeout=timeout
    )

    if responses:
        return responses[0]["data"]

    return None

# Usage
response = request_response(
    request_event_type="review_requested",
    response_event_type="review_complete",
    request_data={
        "files": ["api.py"],
        "reviewer": "code-reviewer"
    }
)

if response:
    print(f"Review result: {response['approved']}")
```

### Broadcast Pattern

```python
def broadcast_to_all(event_type, data):
    """Broadcast event to all subscribers"""
    publish_event(
        event_type=event_type,
        topic="coordination",
        data={**data, "broadcast": True}
    )

# Usage
# Announce architecture decision to all agents
broadcast_to_all(
    event_type="architecture_defined",
    data={
        "decision": "Use microservices",
        "rationale": "Better scalability",
        "affects": "all"
    }
)
```

### Fan-Out Pattern

```python
def fan_out(tasks):
    """Fan out work to multiple agents"""
    correlation_id = str(uuid.uuid4())

    # Spawn multiple agents
    for task in tasks:
        publish_event(
            event_type="task_assigned",
            topic="coordination",
            data={
                "task": task,
                "correlation_id": correlation_id
            }
        )

    # Wait for all to complete
    completed = 0
    while completed < len(tasks):
        events = wait_for_event(
            event_type="agent_completed",
            filter_expr=f"correlation_id:{correlation_id}",
            timeout=30000
        )

        if events:
            completed += len(events)
            print(f"Completed: {completed}/{len(tasks)}")

# Usage
fan_out([
    {"agent_type": "python-specialist", "work": "API"},
    {"agent_type": "go-specialist", "work": "Worker"},
    {"agent_type": "rust-specialist", "work": "CLI"}
])
```

### Circuit Breaker Pattern

```python
class EventCircuitBreaker:
    """Circuit breaker for event subscriptions"""

    def __init__(self, failure_threshold=5, timeout=60):
        self.failure_count = 0
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.state = "closed"  # closed, open, half_open
        self.last_failure_time = None

    def call(self, fn, *args, **kwargs):
        """Execute function with circuit breaker"""
        if self.state == "open":
            if time.time() - self.last_failure_time > self.timeout:
                self.state = "half_open"
            else:
                raise Exception("Circuit breaker is open")

        try:
            result = fn(*args, **kwargs)
            self.on_success()
            return result

        except Exception as e:
            self.on_failure()
            raise

    def on_success(self):
        """Handle successful call"""
        self.failure_count = 0
        if self.state == "half_open":
            self.state = "closed"

    def on_failure(self):
        """Handle failed call"""
        self.failure_count += 1
        self.last_failure_time = time.time()

        if self.failure_count >= self.failure_threshold:
            self.state = "open"

# Usage
breaker = EventCircuitBreaker()

def subscribe_with_breaker():
    return breaker.call(
        wait_for_event,
        "implementation_complete",
        timeout=30000
    )
```

---

## Best Practices

### 1. Use Correlation IDs

Always use correlation IDs to link related events:

```python
import uuid

correlation_id = str(uuid.uuid4())

# All related events use same ID
event1 = {"correlation_id": correlation_id, ...}
event2 = {"correlation_id": correlation_id, ...}
```

### 2. Set Appropriate TTLs

Set TTLs based on event lifetime:

```python
# Short-lived status updates
event = {"ttl_seconds": 3600, ...}  # 1 hour

# Long-lived decisions
event = {"ttl_seconds": 86400, ...}  # 24 hours

# Critical events (max retention)
event = {"ttl_seconds": 86400, ...}  # 24 hours (max)
```

### 3. Include Context in Events

Provide sufficient context in event data:

```python
# Good: Detailed context
event_data = {
    "issue_id": "issue-123",
    "agent_type": "python-specialist",
    "status": "success",
    "files_changed": ["api.py", "test_api.py"],
    "metrics": {"coverage": 95.0},
    "next_steps": ["security_audit"]
}

# Bad: Minimal context
event_data = {
    "status": "done"
}
```

### 4. Handle Timeouts Gracefully

```python
def robust_wait(event_type, max_attempts=3):
    """Wait for event with retries"""
    for attempt in range(max_attempts):
        events = wait_for_event(event_type, timeout=30000)

        if events:
            return events

        print(f"Attempt {attempt + 1} timed out")

    return None
```

### 5. Use Appropriate Topics

```python
# Good: Use standard topics
publish_event("implementation_complete", topic="implementation", ...)

# Bad: Wrong topic
publish_event("implementation_complete", topic="testing", ...)
```

### 6. Log All Event Operations

```python
import logging

logger = logging.getLogger(__name__)

def publish_with_logging(event_type, topic, data):
    logger.info(f"Publishing {event_type} to {topic}")
    event_id = publish_event(event_type, topic, data)
    logger.info(f"Published: {event_id}")
    return event_id
```

---

## Examples

### Complete Workflow Example

```python
#!/usr/bin/env python3
"""
Complete multi-agent workflow using events
"""

import uuid
import requests
import time
from typing import List, Dict, Any

class AgentCoordinator:
    def __init__(self, sidecar_url, jwt_token):
        self.sidecar_url = sidecar_url
        self.headers = {"Authorization": f"Bearer {jwt_token}"}
        self.correlation_id = str(uuid.uuid4())

    def publish_event(self, event_type: str, topic: str, data: Dict[str, Any]):
        """Publish event"""
        event_data = {
            "event_type": event_type,
            "publisher": "coordinator",
            "topic": topic,
            "correlation_id": self.correlation_id,
            "data": data
        }

        response = requests.post(
            f"{self.sidecar_url}/events/{event_type}",
            json=event_data,
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()["event_id"]

    def wait_for_event(self, event_type: str, filter_expr: str = None, timeout: int = 30000):
        """Wait for event"""
        params = {"timeout": timeout}
        if filter_expr:
            params["filter"] = filter_expr

        response = requests.get(
            f"{self.sidecar_url}/events/wait/{event_type}",
            params=params,
            headers=self.headers,
            timeout=timeout/1000 + 5
        )

        result = response.json()
        return result.get("events", [])

    def coordinate_workflow(self):
        """Coordinate complete workflow"""
        print("Starting workflow...")

        # Phase 1: Architecture
        print("\n=== Phase 1: Architecture ===")
        self.publish_event(
            "architecture_defined",
            "architecture",
            {
                "design": "microservices",
                "services": ["api", "worker", "database"]
            }
        )

        # Wait for architecture acknowledgment
        events = self.wait_for_event(
            "agent_completed",
            filter_expr=f"agent_type:python-specialist,correlation_id:{self.correlation_id}"
        )

        if not events:
            print("Architecture phase timeout")
            return

        # Phase 2: Implementation
        print("\n=== Phase 2: Implementation ===")
        self.publish_event(
            "implementation_complete",
            "implementation",
            {
                "files_changed": 5,
                "next_phase": "testing"
            }
        )

        # Wait for testing
        events = self.wait_for_event(
            "tests_passing",
            filter_expr=f"correlation_id:{self.correlation_id}",
            timeout=60000
        )

        if not events:
            print("Testing phase timeout")
            return

        test_results = events[0]["data"]
        print(f"Tests passing: {test_results['coverage']}% coverage")

        # Phase 3: Security
        print("\n=== Phase 3: Security ===")
        events = self.wait_for_event(
            "security_audit_complete",
            filter_expr=f"correlation_id:{self.correlation_id}",
            timeout=60000
        )

        if not events:
            print("Security phase timeout")
            return

        security_results = events[0]["data"]
        if security_results["issues"]:
            print(f"Security issues found: {security_results['issues']}")
            return

        # Phase 4: Deployment
        print("\n=== Phase 4: Deployment ===")
        self.publish_event(
            "deployment_started",
            "deployment",
            {"environment": "production"}
        )

        events = self.wait_for_event(
            "deployment_complete",
            filter_expr=f"correlation_id:{self.correlation_id}",
            timeout=120000
        )

        if events:
            print("Workflow complete!")
        else:
            print("Deployment timeout")

# Usage
coordinator = AgentCoordinator(
    sidecar_url="http://localhost:3001/api",
    jwt_token=os.getenv("JWT_TOKEN")
)

coordinator.coordinate_workflow()
```

---

## See Also

- [Quick Start Guide](ORCHESTRATION_SIDECAR_QUICKSTART.md)
- [API Reference](ORCHESTRATION_SIDECAR_API_REFERENCE.md)
- [Agent Integration](ORCHESTRATION_SIDECAR_AGENT_GUIDE.md)
- [CLI Reference](ORCHESTRATION_SIDECAR_CLI_REFERENCE.md)
- [Advanced Topics](ORCHESTRATION_SIDECAR_ADVANCED.md)
