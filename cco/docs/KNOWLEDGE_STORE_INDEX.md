# Knowledge Store Documentation Index

**Complete documentation for the CCO Knowledge Store HTTP API**

---

## Overview

The Knowledge Store is an embedded vector database system that allows agents to store and retrieve knowledge using semantic similarity search. This documentation suite provides everything you need to understand, integrate, and use the Knowledge Store effectively.

---

## Quick Navigation

### For New Users

Start here to get up and running in 5 minutes:

1. **[Quick Start Guide](KNOWLEDGE_STORE_QUICK_START.md)** - Get started immediately
   - 5-minute setup
   - Basic curl examples
   - Python quick start
   - Common patterns

### For Agent Developers

Integrate the Knowledge Store into your agents:

2. **[Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md)** - Complete integration guide
   - HTTP client setup
   - Common patterns (store, search, batch)
   - Error handling strategies
   - Best practices
   - Code examples in Python, Rust, JavaScript, Bash
   - Connection management
   - Testing examples

### For API Users

Complete API reference and specifications:

3. **[API Reference](KNOWLEDGE_STORE_API.md)** - Complete API documentation
   - All 8 endpoints documented
   - Request/response schemas
   - Authentication details
   - Error codes and handling
   - Rate limits
   - Complete curl examples
   - Client code examples (Python, Rust, Bash)
   - Performance tips

### For Migrators

Moving from the old Node.js system:

4. **[Migration Guide](KNOWLEDGE_STORE_MIGRATION.md)** - Node.js to HTTP API migration
   - Before/after comparisons
   - Step-by-step migration process
   - Code migration examples
   - Data migration (automatic)
   - Troubleshooting migration issues
   - Rollback plan

### For Architects

Understanding how it works internally:

5. **[Architecture Documentation](KNOWLEDGE_STORE_ARCHITECTURE.md)** - Technical deep dive
   - System architecture diagrams
   - Component responsibilities
   - Data flow diagrams
   - Database schema design
   - Embedding strategy (384D vectors)
   - Per-project isolation design
   - Concurrency model
   - Performance characteristics
   - API design rationale

### For Troubleshooters

Common questions and problems:

6. **[FAQ](KNOWLEDGE_STORE_FAQ.md)** - Frequently asked questions
   - Getting started questions
   - API usage questions
   - Search behavior questions
   - Data and storage questions
   - Performance optimization
   - Troubleshooting common errors
   - Migration questions
   - Security questions
   - Best practices

---

## Documentation Structure

### Beginner Track (5-15 minutes)

```
Quick Start → FAQ (Getting Started) → First Integration
```

**Goal:** Store and search your first knowledge item

**Time:** 5 minutes

### Developer Track (30-60 minutes)

```
Quick Start → Agent Integration Guide → API Reference → FAQ
```

**Goal:** Fully integrate Knowledge Store into your agent

**Time:** 30-60 minutes

### Migration Track (1-2 hours)

```
Migration Guide → Agent Integration Guide → Testing → Deployment
```

**Goal:** Successfully migrate from Node.js to HTTP API

**Time:** 1-2 hours

### Architect Track (2-3 hours)

```
Architecture → API Reference → Performance Analysis → Planning
```

**Goal:** Understand system design and make informed decisions

**Time:** 2-3 hours

---

## Document Summary

### 1. Quick Start Guide

**File:** `KNOWLEDGE_STORE_QUICK_START.md`

**Pages:** 5

**Audience:** New users, quick setup

**Contents:**
- Prerequisites check
- Quick test (4 commands)
- Python quick start
- Bash quick start
- Common patterns
- Knowledge types
- Troubleshooting basics
- Next steps

**Key Takeaway:** Get up and running in 5 minutes

---

### 2. Agent Integration Guide

**File:** `KNOWLEDGE_STORE_AGENT_GUIDE.md`

**Pages:** 25

**Audience:** Agent developers, integration engineers

**Contents:**
- What changed from Node.js
- HTTP client setup
- 5 common patterns with examples
- Error handling strategies
- Best practices (7 rules)
- Language examples (Python, Rust, JavaScript, Bash)
- Testing examples
- Connection management
- Health checks

**Key Takeaway:** Everything you need to integrate the Knowledge Store into your agent

---

### 3. API Reference

**File:** `KNOWLEDGE_STORE_API.md`

**Pages:** 35

**Audience:** API users, developers, integrators

**Contents:**
- Complete overview
- Authentication setup
- 8 endpoints fully documented:
  1. Store Knowledge
  2. Search Knowledge
  3. Batch Store
  4. Query Knowledge
  5. Get Statistics
  6. Get Entry by ID
  7. Cleanup Old Entries
  8. Health Check
- Data models (KnowledgeItem, SearchRequest, etc.)
- Error handling (9 error codes)
- Rate limits
- Complete examples (Python, Rust, Bash)
- Performance tips
- Troubleshooting

**Key Takeaway:** Complete API specification with working examples

---

### 4. Migration Guide

**File:** `KNOWLEDGE_STORE_MIGRATION.md`

**Pages:** 20

**Audience:** Teams migrating from Node.js subprocess

**Contents:**
- Migration overview and timeline
- Key differences (5 major changes)
- Before/after code comparisons
- 8-step migration process
- 4 code migration examples
- Data migration (automatic)
- Troubleshooting 5 common issues
- Rollback plan
- Success checklist

**Key Takeaway:** Safe, step-by-step migration from old to new system

---

### 5. Architecture Documentation

**File:** `KNOWLEDGE_STORE_ARCHITECTURE.md`

**Pages:** 30

**Audience:** Architects, senior developers, technical leaders

**Contents:**
- System overview with diagrams
- 5 architecture components explained:
  1. HTTP Server (Axum)
  2. Knowledge Manager
  3. Vector Store (LanceDB)
  4. Embedding Generator
  5. Search Engine
- Data flow diagrams (store and search)
- Database design (Arrow schema)
- Embedding strategy (384D vectors)
- Per-project isolation
- Concurrency model (async/await)
- Performance benchmarks
- API design rationale

**Key Takeaway:** Deep technical understanding of how it works

---

### 6. FAQ

**File:** `KNOWLEDGE_STORE_FAQ.md`

**Pages:** 25

**Audience:** Everyone (organized by topic)

**Contents:**
- General questions (5)
- Getting started (5)
- API questions (6)
- Search questions (6)
- Data questions (6)
- Metadata questions (3)
- Performance questions (4)
- Troubleshooting (4)
- Migration questions (4)
- Security questions (4)
- Advanced questions (4)
- Best practices (3)

**Total:** 54 questions answered

**Key Takeaway:** Quick answers to common questions

---

## Visual Documentation Map

```
┌─────────────────────────────────────────────────────────┐
│                KNOWLEDGE STORE DOCS                     │
└─────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐      ┌─────▼─────┐     ┌─────▼─────┐
   │ QUICK   │      │  AGENT    │     │    API    │
   │ START   │      │  GUIDE    │     │ REFERENCE │
   └────┬────┘      └─────┬─────┘     └─────┬─────┘
        │                 │                  │
        │    ┌────────────┼──────────┐       │
        │    │            │          │       │
   ┌────▼────▼───┐   ┌───▼────┐  ┌──▼───────▼───┐
   │ MIGRATION   │   │  FAQ   │  │ ARCHITECTURE │
   │   GUIDE     │   └────────┘  └──────────────┘
   └─────────────┘
```

---

## Key Concepts Reference

### Knowledge Types

| Type | Description | Example |
|------|-------------|---------|
| `decision` | Architecture decisions | "Use FastAPI for REST API" |
| `architecture` | System design | "Microservices pattern chosen" |
| `implementation` | Code details | "JWT auth implemented" |
| `configuration` | Settings | "DB pool size: 10" |
| `credential` | Credential refs | "API key in vault" |
| `issue` | Problems/bugs | "Memory leak in worker" |
| `general` | Other | "User feedback" |

### Endpoints Summary

| Endpoint | Method | Purpose | Doc Section |
|----------|--------|---------|-------------|
| `/store` | POST | Store single item | API Ref §3.1 |
| `/store/batch` | POST | Store multiple | API Ref §3.3 |
| `/search` | GET | Vector search | API Ref §3.2 |
| `/query` | POST | SQL-like query | API Ref §3.4 |
| `/stats` | GET | Statistics | API Ref §3.5 |
| `/{id}` | GET | Get by ID | API Ref §3.6 |
| `/cleanup` | DELETE | Delete old | API Ref §3.7 |
| `/health` | GET | Health check | API Ref §3.8 |

### Response Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | OK | Success |
| 201 | Created | Item stored |
| 400 | Bad Request | Fix request |
| 401 | Unauthorized | Check token |
| 404 | Not Found | Item doesn't exist |
| 429 | Rate Limited | Wait and retry |
| 500 | Server Error | Check logs |
| 503 | Unavailable | Check daemon |

---

## Common Workflows

### Workflow 1: First-Time Setup

```
1. Read: Quick Start Guide
2. Test: Health check (curl)
3. Try: Store first item
4. Try: Search for item
5. Read: Agent Integration Guide
6. Integrate: Add to your agent
```

**Time:** 15 minutes

### Workflow 2: Full Integration

```
1. Read: Quick Start Guide
2. Read: Agent Integration Guide
3. Create: HTTP client wrapper
4. Test: Store, search, stats
5. Integrate: Add to agent code
6. Test: End-to-end workflow
7. Deploy: Production deployment
8. Monitor: Check logs and stats
```

**Time:** 2-3 hours

### Workflow 3: Migration from Node.js

```
1. Read: Migration Guide
2. Test: New API locally
3. Create: HTTP client wrapper
4. Find: All subprocess calls
5. Replace: With HTTP calls
6. Test: Local integration tests
7. Migrate: Data (automatic copy)
8. Deploy: To production
9. Monitor: 1-2 weeks
10. Cleanup: Remove old code
```

**Time:** 1 week (including monitoring)

### Workflow 4: Troubleshooting

```
1. Check: Daemon status
2. Check: Auth token exists
3. Test: Health endpoint
4. Read: FAQ troubleshooting section
5. Check: Daemon logs
6. Try: Specific error solutions
7. If stuck: File GitHub issue
```

**Time:** 10-30 minutes

---

## Code Example Index

### Python Examples

- **Basic client**: Agent Guide §6.1, API Ref §7
- **Error handling**: Agent Guide §4
- **Retry logic**: Agent Guide §4.1
- **Batch operations**: Agent Guide §3.3
- **Connection pooling**: Agent Guide §9
- **Unit tests**: Agent Guide §8

### Rust Examples

- **Basic client**: API Ref §7
- **Async/await**: Architecture §7.2
- **Error types**: Architecture §6.1

### Bash Examples

- **CLI wrapper**: Quick Start §3, Agent Guide §6.2
- **Store/search**: Quick Start §2-3
- **Stats**: Quick Start §4

### JavaScript Examples

- **Node.js client**: Agent Guide §6.3
- **Async/await**: Agent Guide §6.3

---

## Performance Reference

### Expected Latencies

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Store (single) | <10ms | 100/sec |
| Store (batch 100) | <500ms | 200 items/sec |
| Search (10 results) | <15ms | 66/sec |
| Search (100K entries) | <100ms | 10/sec |
| Stats | <5ms | 200/sec |

**Source:** Architecture §8

### Scalability

| Entries | Search Time |
|---------|-------------|
| 10K | <10ms |
| 100K | <50ms |
| 1M | <200ms |

**Source:** Architecture §8

---

## Checklist: Am I Ready?

### For Using the API

- [ ] Daemon is running (`cco daemon status`)
- [ ] Token exists (`cat ~/.cco/api_token`)
- [ ] Health check passes (curl test)
- [ ] Understand knowledge types
- [ ] Know how to store and search
- [ ] Have error handling strategy

**If all checked:** You're ready! Start with Quick Start Guide.

### For Integration

- [ ] Read Agent Integration Guide
- [ ] Have HTTP client library
- [ ] Created client wrapper
- [ ] Tested store operation
- [ ] Tested search operation
- [ ] Tested error handling
- [ ] Have retry logic
- [ ] Added to agent code
- [ ] Ran integration tests

**If all checked:** You're ready to deploy!

### For Migration

- [ ] Read Migration Guide
- [ ] Tested new API locally
- [ ] Found all subprocess calls
- [ ] Replaced with HTTP calls
- [ ] Tested locally
- [ ] Have rollback plan
- [ ] Backed up data
- [ ] Ready to monitor post-deployment

**If all checked:** You're ready to migrate!

---

## Getting Help

### Self-Service Resources

1. **Quick answers**: [FAQ](KNOWLEDGE_STORE_FAQ.md)
2. **Common errors**: FAQ §7 (Troubleshooting)
3. **Code examples**: [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md) §6
4. **API details**: [API Reference](KNOWLEDGE_STORE_API.md)

### When Stuck

1. **Check daemon logs**: `~/.cco/logs/daemon.log`
2. **Check health**: `curl .../health`
3. **Check stats**: `curl .../stats`
4. **Search FAQ**: Press Ctrl+F in FAQ doc
5. **File issue**: GitHub with details

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-18 | Initial documentation release |

---

## Future Documentation

Planned for future versions:

- **Advanced Search Guide**: Hybrid search, full-text search, advanced filters
- **Performance Tuning Guide**: Optimization strategies, profiling, benchmarking
- **Multi-Tenant Guide**: Running multiple isolated instances
- **Custom Models Guide**: Using different embedding models
- **Monitoring Guide**: Metrics, dashboards, alerting
- **GraphQL API Guide**: Alternative to REST (if implemented)

---

## Document Statistics

| Metric | Value |
|--------|-------|
| Total Documents | 6 |
| Total Pages | 140 |
| Code Examples | 50+ |
| Curl Examples | 30+ |
| FAQ Questions | 54 |
| Endpoints Documented | 8 |
| Languages Covered | 4 (Python, Rust, JavaScript, Bash) |
| Diagrams | 8 |

---

## Quick Links

- [Quick Start Guide](KNOWLEDGE_STORE_QUICK_START.md)
- [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md)
- [API Reference](KNOWLEDGE_STORE_API.md)
- [Migration Guide](KNOWLEDGE_STORE_MIGRATION.md)
- [Architecture](KNOWLEDGE_STORE_ARCHITECTURE.md)
- [FAQ](KNOWLEDGE_STORE_FAQ.md)

---

**Documentation Complete**

All documentation is located in `/Users/brent/git/cc-orchestra/cco/docs/`

**Last Updated:** November 18, 2025
**Version:** 1.0.0
**Maintained by:** CCO Team
