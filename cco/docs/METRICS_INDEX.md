# CCO Metrics Backend - Documentation Index

**Complete documentation for the API cost monitoring daemon backend**
**Version**: 2025.11.2
**Status**: ✅ Design Complete - Ready for Implementation

---

## 📚 Documentation Overview

This directory contains the complete backend architecture for CCO's metrics tracking system. All documents are production-ready and implementation can begin immediately.

---

## 🎯 Quick Start (Choose Your Path)

### For Chief Architect (Review & Approval)
1. Read **Summary** for executive overview
2. Scan **Architecture** for technical depth
3. Review **Checklist** for timeline/scope
4. Sign off for implementation

### For Implementation Team
1. Read **Quick Reference** for developer guide
2. Follow **Checklist** for phased tasks
3. Reference **Architecture** for details
4. Use **Diagrams** for visual understanding

### For TUI Developer (Dashboard Integration)
1. Read **Quick Reference** - REST API section
2. Review **Architecture** - Section 5 (API for TUI)
3. Test endpoints with provided curl commands
4. Coordinate polling strategy (1s interval)

### For Test Engineer
1. Read **Checklist** - Phase 6 (Testing)
2. Review **Quick Reference** - Testing section
3. Use **Architecture** for edge cases
4. Implement test suite with 90%+ coverage

---

## 📖 Complete Documentation

### 1. Executive Summary
**File**: `METRICS_BACKEND_SUMMARY.md`
**Length**: ~4,000 words
**Purpose**: High-level overview for stakeholders

**Contents**:
- Deliverables checklist
- Architecture highlights
- Design decisions and rationale
- Success criteria
- Next steps

**Read if**: You need a quick overview or sign-off approval

**Path**: `/Users/brent/git/cc-orchestra/cco/docs/METRICS_BACKEND_SUMMARY.md`

---

### 2. Full Architecture Specification
**File**: `METRICS_BACKEND_ARCHITECTURE.md`
**Length**: ~12,000 words
**Purpose**: Complete technical specification

**Contents**:
- Data model design (SQLite schema)
- Integration points with CCO proxy
- Cost calculation specifications
- Persistence strategy (batch writes)
- Real-time metrics pipeline
- REST API for TUI dashboard
- Configuration management
- Error handling and recovery
- Performance optimization
- Security considerations
- Appendices (SQL queries, examples)

**Read if**: You need deep technical understanding or implementation guidance

**Path**: `/Users/brent/git/cc-orchestra/cco/docs/METRICS_BACKEND_ARCHITECTURE.md`

---

### 3. Implementation Checklist
**File**: `METRICS_IMPLEMENTATION_CHECKLIST.md`
**Length**: ~3,000 words
**Purpose**: Phased implementation plan

**Contents**:
- 7 phases over 15 days
- Task breakdown by phase
- Verification commands
- Success criteria per phase
- Risk mitigation strategies
- Rollback plan

**Read if**: You're implementing the backend step-by-step

**Path**: `/Users/brent/git/cc-orchestra/cco/docs/METRICS_IMPLEMENTATION_CHECKLIST.md`

---

### 4. Developer Quick Reference
**File**: `METRICS_QUICK_REFERENCE.md`
**Length**: ~5,000 words
**Purpose**: Developer-friendly cheat sheet

**Contents**:
- Architecture diagrams (ASCII)
- Core data structures
- Implementation snippets
- Cost calculation examples
- Testing commands
- Common issues & solutions
- Performance targets
- Integration checklist

**Read if**: You want practical code examples and quick lookups

**Path**: `/Users/brent/git/cc-orchestra/cco/docs/METRICS_QUICK_REFERENCE.md`

---

### 5. Visual Architecture Diagrams
**File**: `METRICS_ARCHITECTURE_DIAGRAM.md`
**Length**: ~3,000 words
**Purpose**: Visual system understanding

**Contents**:
- System architecture (high-level)
- Data flow sequence diagrams
- Component architecture
- RingBuffer visualization
- Database schema relationships
- Cost calculation flow
- Tier breakdown visualization
- Performance optimization flow
- Error recovery flow

**Read if**: You learn best from visual diagrams

**Path**: `/Users/brent/git/cc-orchestra/cco/docs/METRICS_ARCHITECTURE_DIAGRAM.md`

---

## 🗄️ Database Migrations

### Migration 001: Initial Schema
**File**: `001_initial_schema.sql`
**Purpose**: Core table definitions

**Tables Created**:
- `api_calls` - Individual API call tracking
- `metrics_aggregated` - Pre-computed summaries
- `model_tiers` - Pricing configuration
- `sessions` - Multi-agent workflow tracking
- `config` - Runtime configuration

**Path**: `/Users/brent/git/cc-orchestra/cco/migrations/001_initial_schema.sql`

---

### Migration 002: Seed Pricing
**File**: `002_seed_pricing.sql`
**Purpose**: Insert default pricing (January 2025)

**Models Included**:
- Claude Opus 4 ($15/$75 per 1M tokens)
- Claude Sonnet 4.5 ($3/$15 per 1M tokens)
- Claude Haiku 4.5 ($0.80/$4.00 per 1M tokens)
- OpenAI GPT-4 (for comparison)
- Ollama models (free, self-hosted)

**Path**: `/Users/brent/git/cc-orchestra/cco/migrations/002_seed_pricing.sql`

---

### Migration 003: Performance Indexes
**File**: `003_add_indexes.sql`
**Purpose**: Query optimization

**Indexes Added**:
- Covering indexes for tier/model queries
- Composite indexes for dashboard queries
- Partial indexes for error tracking
- Session performance indexes

**Path**: `/Users/brent/git/cc-orchestra/cco/migrations/003_add_indexes.sql`

---

## 🔍 Document Navigation

### By Role

**Chief Architect**:
```
1. METRICS_BACKEND_SUMMARY.md (start here)
2. METRICS_BACKEND_ARCHITECTURE.md (deep dive)
3. METRICS_IMPLEMENTATION_CHECKLIST.md (timeline)
```

**Backend Developer**:
```
1. METRICS_QUICK_REFERENCE.md (start here)
2. METRICS_IMPLEMENTATION_CHECKLIST.md (tasks)
3. METRICS_BACKEND_ARCHITECTURE.md (reference)
4. migrations/*.sql (database setup)
```

**TUI Developer**:
```
1. METRICS_QUICK_REFERENCE.md - REST API section
2. METRICS_BACKEND_ARCHITECTURE.md - Section 5
3. METRICS_ARCHITECTURE_DIAGRAM.md - Data flow
```

**Test Engineer**:
```
1. METRICS_IMPLEMENTATION_CHECKLIST.md - Phase 6
2. METRICS_QUICK_REFERENCE.md - Testing section
3. METRICS_BACKEND_ARCHITECTURE.md - Appendix C
```

---

### By Topic

**Architecture & Design**:
- Full spec: `METRICS_BACKEND_ARCHITECTURE.md`
- Visual diagrams: `METRICS_ARCHITECTURE_DIAGRAM.md`
- Quick overview: `METRICS_BACKEND_SUMMARY.md`

**Implementation**:
- Step-by-step tasks: `METRICS_IMPLEMENTATION_CHECKLIST.md`
- Code examples: `METRICS_QUICK_REFERENCE.md`
- Database setup: `migrations/*.sql`

**Integration**:
- REST API: `METRICS_BACKEND_ARCHITECTURE.md` - Section 5
- Data flow: `METRICS_ARCHITECTURE_DIAGRAM.md` - Section 2
- CCO proxy hooks: `METRICS_QUICK_REFERENCE.md` - Integration section

**Testing**:
- Test strategy: `METRICS_IMPLEMENTATION_CHECKLIST.md` - Phase 6
- Test commands: `METRICS_QUICK_REFERENCE.md` - Testing section
- Edge cases: `METRICS_BACKEND_ARCHITECTURE.md` - Section 11

**Performance**:
- Targets: `METRICS_BACKEND_ARCHITECTURE.md` - Section 9
- Optimization: `METRICS_ARCHITECTURE_DIAGRAM.md` - Section 8
- Benchmarks: `METRICS_QUICK_REFERENCE.md` - Performance section

---

## 📊 Key Specifications at a Glance

### Performance Targets

| Metric | Target | Verification |
|--------|--------|--------------|
| Write latency | <1ms | Benchmark: 10k calls in <10s |
| Query latency | <10ms | curl timing |
| Throughput | 10k calls/min | Load test with ab |
| Memory usage | <100MB | Ring buffers + counters |
| Database size | <500MB/month | 1M calls ≈ 400MB |

### Database Tables

| Table | Rows (estimate) | Purpose |
|-------|-----------------|---------|
| api_calls | 10k-1M/month | Individual API calls |
| metrics_aggregated | 100-1k/month | Pre-computed summaries |
| model_tiers | 10-20 | Pricing configuration |
| sessions | 10-100/month | Multi-agent workflows |
| config | 5-10 | Runtime configuration |

### REST API Endpoints

| Endpoint | Method | Purpose | Cache |
|----------|--------|---------|-------|
| /api/metrics/stats | GET | Summary (10m window) | 1s TTL |
| /api/metrics/realtime | GET | Real-time (1m window) | 1s TTL |
| /api/metrics/tiers/{tier} | GET | Per-tier breakdown | 1s TTL |
| /api/metrics/recent | GET | Recent calls (20) | None |
| /api/metrics/health | GET | Backend health | None |

### Cost Calculation (January 2025)

| Model | Tier | Input ($/1M) | Output ($/1M) |
|-------|------|--------------|---------------|
| Opus 4 | opus | $15.00 | $75.00 |
| Sonnet 4.5 | sonnet | $3.00 | $15.00 |
| Haiku 4.5 | haiku | $0.80 | $4.00 |

---

## 🚀 Implementation Timeline

### 15-Day Plan

| Phase | Days | Deliverables |
|-------|------|--------------|
| 1. Database Foundation | 2 | SQLite schema, MetricsDatabase |
| 2. In-Memory Aggregation | 2 | MetricsAggregator, RingBuffer |
| 3. Backend Integration | 2 | MetricsBackend, BatchWriter |
| 4. Server Integration | 2 | REST API endpoints, QueryCache |
| 5. Configuration & CLI | 2 | Pricing config, CLI commands |
| 6. Testing & Optimization | 4 | Tests, benchmarks, profiling |
| 7. Documentation | 1 | User guides, API reference |

**Estimated LOC**: ~2,000 lines (backend only, excluding TUI)

---

## ✅ Readiness Checklist

**Architecture Documentation**:
- [x] Full specification written (12k words)
- [x] Implementation checklist created (15-day plan)
- [x] Quick reference guide for developers
- [x] Visual diagrams for all components
- [x] Executive summary for stakeholders

**Database Design**:
- [x] SQLite schema designed
- [x] Migration files written (001, 002, 003)
- [x] Indexes for performance
- [x] Seed data for pricing

**Integration Specifications**:
- [x] CCO proxy integration points identified
- [x] REST API contract defined
- [x] Data flow documented
- [x] Cost calculation specified

**Testing Strategy**:
- [x] Unit test requirements
- [x] Integration test plan
- [x] Performance benchmarks
- [x] Coverage targets (90%+)

**Status**: ✅ **READY FOR IMPLEMENTATION**

---

## 📞 Coordination

### Chief Architect
- **Review**: METRICS_BACKEND_SUMMARY.md
- **Approve**: Architecture and timeline
- **Coordinate**: Integration with TUI developer

### TUI Developer
- **Read**: METRICS_QUICK_REFERENCE.md - REST API section
- **Test**: Endpoint contracts with curl
- **Implement**: 1-second polling from dashboard
- **Coordinate**: Data format with backend team

### Implementation Team
- **Follow**: METRICS_IMPLEMENTATION_CHECKLIST.md
- **Reference**: METRICS_QUICK_REFERENCE.md for code examples
- **Verify**: Success criteria at each phase
- **Report**: Progress to Chief Architect

### Test Engineer
- **Plan**: Test suite based on specifications
- **Target**: 90%+ code coverage
- **Benchmark**: Performance targets
- **Validate**: Edge cases and error recovery

---

## 📝 Notes for Implementers

### Critical Design Decisions

1. **SQLite over PostgreSQL**: Single executable requirement
2. **Batch writes**: 100x performance improvement
3. **Ring buffers**: Real-time aggregation without DB queries
4. **1s cache TTL**: Balance real-time feel with performance

### Common Pitfalls to Avoid

1. **Don't query DB for real-time stats** - Use in-memory aggregator
2. **Don't block on writes** - Use async batch writer
3. **Don't hardcode pricing** - Load from model_tiers table
4. **Don't skip error recovery** - Always update aggregator first

### Performance Optimization Tips

1. Enable WAL mode on SQLite (in migration 001)
2. Use prepared statements (SQLx handles this)
3. Batch writes (100 calls or 5 seconds)
4. Cache queries (1s TTL)
5. Index frequently queried columns

---

## 🔗 External References

### CCO Proxy Codebase

| File | Relevance |
|------|-----------|
| `src/analytics.rs` | Existing API tracking (integrate with) |
| `src/router.rs` | Cost calculation (use pricing from) |
| `src/server.rs` | REST API endpoints (add metrics/*) |
| `src/cache.rs` | Cache implementation (similar pattern) |
| `Cargo.toml` | Dependencies (sqlx already included) |

### SQL Resources
- SQLite documentation: https://www.sqlite.org/docs.html
- SQLx Rust crate: https://docs.rs/sqlx/latest/sqlx/

### Rust Libraries Used
- `sqlx` - Async SQL database driver
- `tokio` - Async runtime
- `serde` - Serialization
- `dashmap` - Concurrent HashMap
- `chrono` - Date/time handling

---

## 📄 Document Change Log

| Version | Date | Changes |
|---------|------|---------|
| 2025.11.2 | 2025-01-17 | Initial architecture complete |

---

**End of Documentation Index**

All documents are ready for implementation. No blockers identified.

For questions or clarifications, refer to specific documents or contact the Backend Architect.
