# `/api/stats` Endpoint Specification Suite

## Overview

Complete documentation of what the TUI expects from the `/api/stats` endpoint. This suite contains 5 comprehensive documents totaling ~150 KB of detailed specifications, examples, parsing flows, and test cases.

---

## Document Index

### 1. STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md (START HERE)
**Purpose**: Quick start guide with implementation checklist
**Length**: ~8 KB
**Best for**:
- Getting started quickly
- Understanding what's required
- Quick reference during implementation
- Debugging common mistakes

**Contains**:
- 30-second TL;DR
- Current implementation status
- Critical requirements
- Cost calculation examples
- Common mistakes (with fixes)
- Testing instructions
- Debugging guide

---

### 2. STATS_ENDPOINT_SPECIFICATION.md (COMPREHENSIVE)
**Purpose**: Formal complete specification with all details
**Length**: ~50 KB
**Best for**:
- Full understanding of requirements
- Implementing from scratch
- Understanding data sources
- Reference during development

**Contains**:
- Complete JSON schema
- Rust type definitions
- How TUI uses each field
- Data source documentation
- Display components reference
- Refresh frequency details
- Example JSON responses
- Error handling specifications
- Testing checklist

---

### 3. STATS_ENDPOINT_QUICK_REFERENCE.md (LOOKUP)
**Purpose**: Quick lookup reference for parsing paths and common issues
**Length**: ~20 KB
**Best for**:
- Quick lookup during development
- Common implementation mistakes
- Model name reference
- Response validation

**Contains**:
- JSON parsing paths used by TUI
- How TUI uses each field
- Rust type definitions (condensed)
- Common mistakes with corrections
- Display output examples
- Implementation checklist
- Testing curl commands

---

### 4. STATS_PARSING_FLOW.md (DETAILED FLOW)
**Purpose**: Visual walkthrough of how TUI parses the response
**Length**: ~30 KB
**Best for**:
- Understanding parsing logic
- Algorithm verification
- Step-by-step breakdown
- Visual data flow

**Contains**:
- Overall parsing diagram
- Step-by-step parsing algorithms
- parse_cost_by_tier() walkthrough
- parse_recent_calls() walkthrough
- Rendering examples
- Critical parsing rules
- Performance characteristics
- Full request/response cycle example

---

### 5. STATS_ENDPOINT_TEST_CASES.md (TESTING)
**Purpose**: 10+ test scenarios to validate implementation
**Length**: ~25 KB
**Best for**:
- Comprehensive testing
- Edge case validation
- Manual testing procedures
- Debugging specific issues

**Contains**:
- Test Case 1: Minimum valid response
- Test Case 2: Full featured response
- Test Case 3: Large numbers
- Test Case 4: Model name variations
- Test Case 5: Zero/empty values
- Test Case 6: Missing optional fields
- Test Case 7: Cache hit scenario
- Test Case 8: Percentage rounding
- Test Case 9: Response time test
- Test Case 10: Concurrent requests
- Manual testing checklist
- Debugging tips

---

## Quick Navigation

### I want to...

**...understand what I need to implement**
→ Start with [STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md](#1-stats_endpoint_implementation_guidemd-start-here)

**...see the complete formal specification**
→ Read [STATS_ENDPOINT_SPECIFICATION.md](#2-stats_endpoint_specificationmd-comprehensive)

**...look up a specific parsing path**
→ Reference [STATS_ENDPOINT_QUICK_REFERENCE.md](#3-stats_endpoint_quick_referencemd-lookup)

**...understand how TUI parses the JSON**
→ Study [STATS_PARSING_FLOW.md](#4-stats_parsing_flowmd-detailed-flow)

**...test my implementation**
→ Run [STATS_ENDPOINT_TEST_CASES.md](#5-stats_endpoint_test_casesmd-testing)

---

## Essential Information at a Glance

### Endpoint URL
```
GET /api/stats
```

### Response Type
```json
{
  "project": {
    "cost": 100.0,
    "tokens": 500000,
    "calls": 1000,
    "name": "string",
    "last_updated": "ISO8601"
  },
  "machine": {...},
  "activity": [{
    "model": "claude-sonnet-4-5",
    "cost": 0.05,
    "file_source": "src/main.py"
  }],
  "chart_data": {
    "model_distribution": [
      {"model": "claude-sonnet-4-5", "percentage": 50.0},
      {"model": "claude-opus-4", "percentage": 30.0},
      {"model": "claude-haiku-4-5", "percentage": 20.0}
    ]
  }
}
```

### Critical Fields (MUST HAVE)
- `project.cost` (f64)
- `project.tokens` (u64)
- `project.calls` (u64)
- `machine` object
- `activity[].model` (e.g., "claude-sonnet-4-5")
- `activity[].cost` (f64)
- `activity[].file_source` (string)
- `chart_data.model_distribution[].model` (string)
- `chart_data.model_distribution[].percentage` (0-100, sum ≈ 100%)

### TUI Parsing Logic
```
For each model in chart_data.model_distribution:
  tier_cost = project.cost * model.percentage / 100.0
  tier_calls = project.calls * model.percentage / 100.0
  tier_name = model.model.contains("sonnet") ? "Sonnet" : ...

For each event in activity (first 20):
  tier = event.model.contains("sonnet") ? "Sonnet" : ...
  cost = event.cost
  file = event.file_source
```

### Response Time
- Cached: < 10ms
- Uncached: < 100ms
- TUI Timeout: 5 seconds

---

## Document Statistics

| Document | Size | Words | Sections | Code Examples |
|----------|------|-------|----------|---|
| Implementation Guide | 8 KB | ~1,200 | 15 | 8 |
| Specification | 50 KB | ~8,000 | 11 | 12 |
| Quick Reference | 20 KB | ~3,500 | 10 | 15 |
| Parsing Flow | 30 KB | ~5,000 | 9 | 20 |
| Test Cases | 25 KB | ~4,500 | 10 | 25 |
| **Total** | **~145 KB** | **~22,000** | **55** | **80** |

---

## Common Issues & Solutions

### Issue: "CostByTier all zeros"
**Solution**: Check `chart_data.model_distribution` has model names with "sonnet", "opus", or "haiku" and percentages sum to ~100%

### Issue: "Recent calls empty"
**Solution**: Verify `activity` array exists with at least one entry, each with `model`, `cost`, and `file_source` fields

### Issue: "Unknown tier names"
**Solution**: Model names must contain (case-insensitive): "sonnet", "opus", or "haiku"

### Issue: "Response takes 5+ seconds"
**Solution**: Check database query performance, consider caching strategy

### Issue: "Display shows 'Unknown' for files"
**Solution**: Add `file_source` field to activity events

---

## Implementation Workflow

```
1. Read STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md
   ↓ (understand requirements)

2. Reference STATS_ENDPOINT_SPECIFICATION.md for details
   ↓ (check specific field requirements)

3. Study STATS_PARSING_FLOW.md examples
   ↓ (understand TUI expectations)

4. Implement endpoint
   ↓ (build/modify code)

5. Test with curl
   ↓ (validate JSON structure)

6. Run STATS_ENDPOINT_TEST_CASES.md scenarios
   ↓ (verify edge cases)

7. Launch TUI, verify display
   ↓ (integration test)

8. Reference STATS_ENDPOINT_QUICK_REFERENCE.md for lookups
   ↓ (ongoing maintenance)
```

---

## File Locations in Repository

```
/Users/brent/git/cc-orchestra/cco/
├── STATS_ENDPOINT_README.md (this file)
├── STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md
├── STATS_ENDPOINT_SPECIFICATION.md
├── STATS_ENDPOINT_QUICK_REFERENCE.md
├── STATS_PARSING_FLOW.md
├── STATS_ENDPOINT_TEST_CASES.md
│
├── src/
│   ├── server.rs (endpoint handler, line 612)
│   ├── tui_app.rs (TUI parser, lines 478-689)
│   ├── api_client.rs (HTTP client with retry logic)
│   ├── analytics.rs (ActivityEvent struct)
│   └── ...
│
└── Cargo.toml
```

---

## Key Source Code References

### Endpoint Implementation
- **File**: `src/server.rs`
- **Function**: `async fn stats()` (line 612)
- **Returns**: `Json<StatsResponse>`

### TUI Parser
- **File**: `src/tui_app.rs`
- **Load function**: `load_agents_and_stats()` (line 479)
- **Parse cost**: `parse_cost_by_tier()` (line 522)
- **Parse calls**: `parse_recent_calls()` (line 654)

### Type Definitions
- **StatsResponse**: `src/server.rs:225`
- **ProjectInfo**: `src/server.rs:920`
- **ActivityEvent**: `src/analytics.rs`
- **ModelDistribution**: `src/server.rs:895`

---

## Testing Checklist

### Automated Testing
- [ ] Response is valid JSON
- [ ] All required fields present
- [ ] Model names recognized
- [ ] Percentages sum to ~100%
- [ ] No null values in critical fields

### Manual Testing
- [ ] Curl test returns valid JSON
- [ ] TUI loads without errors
- [ ] Cost summary displays correctly
- [ ] Recent calls list populated
- [ ] All three tiers shown (Sonnet, Opus, Haiku)

### Edge Cases
- [ ] Zero cost project
- [ ] Large numbers (formatting)
- [ ] Missing optional fields
- [ ] Model name variations
- [ ] Empty activity array

---

## Glossary

| Term | Definition |
|------|-----------|
| **StatsResponse** | Main JSON response struct from `/api/stats` |
| **CostByTier** | Parsed result: cost breakdown by model tier |
| **RecentCall** | Parsed activity event: tier, cost, file |
| **Model Distribution** | Array of models with usage percentages |
| **ActivityEvent** | Single API call or event with metadata |
| **TUI** | Terminal User Interface (the display app) |
| **Tier** | Model category: Sonnet, Opus, or Haiku |

---

## Support & Debugging

### Quick Validation
```bash
# Test endpoint responds
curl http://localhost:3000/api/stats

# Validate JSON structure
curl -s http://localhost:3000/api/stats | jq .

# Check critical fields
curl -s http://localhost:3000/api/stats | jq '{cost:.project.cost, tokens:.project.tokens}'

# Verify percentages sum to 100
curl -s http://localhost:3000/api/stats | jq '[.chart_data.model_distribution[].percentage] | add'
```

### Common Debug Outputs
See **STATS_ENDPOINT_QUICK_REFERENCE.md** section "Debugging Tips"
See **STATS_ENDPOINT_TEST_CASES.md** section "Debugging Tips"

---

## Document Versioning

- **Specification Version**: 1.0 (Final)
- **Effective Date**: 2025-11-26
- **Based on**: TUI source code analysis (tui_app.rs, lines 478-689)
- **Current Implementation**: src/server.rs line 612

---

## How to Use This Suite

### For Quick Implementation (1-2 hours)
1. Read STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md
2. Implement based on checklist
3. Test with curl commands
4. Verify TUI displays correctly

### For Comprehensive Understanding (3-4 hours)
1. Read all 5 documents in order
2. Study examples and diagrams
3. Understand parsing logic in detail
4. Test all edge cases
5. Ready for any future modifications

### For Ongoing Maintenance
- Keep STATS_ENDPOINT_QUICK_REFERENCE.md handy
- Reference parsing details from STATS_PARSING_FLOW.md
- Use TEST_CASES.md for regression testing

---

## Next Steps

1. **Start Implementation**:
   ```bash
   # Read the implementation guide
   cat STATS_ENDPOINT_IMPLEMENTATION_GUIDE.md

   # Then read full specification
   cat STATS_ENDPOINT_SPECIFICATION.md
   ```

2. **Test Your Implementation**:
   ```bash
   # Run endpoint
   curl -s http://localhost:3000/api/stats | jq .

   # Validate structure
   cd cco
   cargo test  # Run existing tests
   ```

3. **Verify TUI Works**:
   ```bash
   # Start TUI
   cco tui

   # Check all sections render
   # Cost summary, recent calls, overall summary
   ```

---

## Questions?

Refer to:
- **"How does the TUI parse this?"** → STATS_PARSING_FLOW.md
- **"What exact fields do I need?"** → STATS_ENDPOINT_SPECIFICATION.md, section 2
- **"What are common mistakes?"** → STATS_ENDPOINT_QUICK_REFERENCE.md
- **"How do I test this?"** → STATS_ENDPOINT_TEST_CASES.md
- **"Which file contains the code?"** → Each document has "Source Code References"

---

## Summary

This specification suite provides **everything needed** to:
- ✅ Understand what `/api/stats` must return
- ✅ Implement the endpoint correctly
- ✅ Parse responses in TUI
- ✅ Test thoroughly
- ✅ Debug issues
- ✅ Maintain long-term

**Total documentation**: ~150 KB, ~22,000 words, 55+ sections, 80+ code examples

**Status**: Complete and ready for implementation.

