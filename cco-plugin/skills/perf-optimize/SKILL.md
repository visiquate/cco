# Perf Optimize: Performance Optimization

## Trigger
`/perf-optimize <component or requirement>`

Identifies and executes performance optimization with profiling, analysis, and validation.

## Purpose
The Perf Optimize skill provides:
- Performance profiling and analysis
- Bottleneck identification
- Optimization strategy development
- Implementation and validation
- Baseline comparison
- Load testing

## Workflow

### Phase 1: Define Performance Requirements

1. **Query performance targets**
   ```bash
   cco knowledge search "performance target"
   cco knowledge search "performance requirement"
   cco knowledge search "response time"
   ```

2. **Identify current metrics**
   - Current response time
   - Throughput/requests per second
   - Memory usage
   - CPU utilization
   - Database query time

3. **Determine optimization scope**
   - Specific endpoints or functions
   - Overall system
   - Database queries
   - Frontend performance
   - Infrastructure

### Phase 2: Spawn Performance Team

**Parallel team:**
- **Performance Engineer** (Sonnet) - Analysis & strategy
- **Performance Profiler** (Sonnet) - Detailed profiling
- **Database Architect** (Sonnet) - Query optimization
- **Language Specialist** (Haiku) - Code optimization

### Phase 3: Establish Baselines

**Performance Profiler measures:**

1. **Response Time**
   ```bash
   # Before optimization
   Endpoint: GET /api/users
   p50: 245ms
   p95: 380ms
   p99: 520ms
   Target: p95 < 200ms
   Status: EXCEEDS TARGET by 90%
   ```

2. **Throughput**
   ```bash
   # Requests per second
   Current: 120 req/s
   Target: 500 req/s
   Gap: 76% under target
   ```

3. **Resource Usage**
   ```bash
   Memory (peak): 340MB
   CPU (average): 65%
   Database connections: 12/20 (60%)
   ```

4. **Database Metrics**
   ```bash
   Query time (avg): 180ms
   Query count per request: 15 (N+1 problem detected)
   Slow queries: 5 taking > 100ms each
   ```

### Phase 4: Analyze and Identify Bottlenecks

**Performance Engineer analysis:**

1. **Profiling Report**
   ```
   Call Graph Analysis:
   - API handler: 10ms (8%)
   - Database queries: 180ms (72%) ← BOTTLENECK
   - Response marshaling: 30ms (12%)
   - Network latency: 20ms (8%)

   Time Distribution:
   - Query 1 (get user): 5ms
   - Query 2-15 (get posts): 170ms ← PROBLEM
   - Loop inefficiency: 5ms
   ```

2. **Bottleneck Identification**
   ```
   Bottleneck 1: N+1 Database Queries (CRITICAL)
   - Issue: Fetching user, then loop-fetching each post's comments
   - Impact: 170ms out of 250ms (68% of total time)
   - Root cause: No eager loading in ORM
   - Fix: Use JOINs or eager load() syntax

   Bottleneck 2: Memory Usage (MEDIUM)
   - Issue: Large result set loaded into memory
   - Impact: 340MB peak memory
   - Root cause: Fetching all data at once
   - Fix: Implement pagination or streaming

   Bottleneck 3: Unoptimized Query (MEDIUM)
   - Issue: Missing database index
   - Impact: 45ms per filtered query
   - Root cause: No index on user.status column
   - Fix: Add database index
   ```

### Phase 5: Design Optimization Strategy

**Performance Engineer + Database Architect:**

1. **Optimization Plan**
   ```
   Optimization 1: Fix N+1 Queries (Impact: -68%)
   - Current: 15 separate queries
   - Optimized: 1 query with JOINs
   - Expected improvement: 170ms → 15ms
   - Implementation: Modify ORM query, add tests
   - Risk: Low (no behavior change)
   - Timeline: 2 hours

   Optimization 2: Add Database Index (Impact: -12%)
   - Current: Sequential scan on users table
   - Optimized: B-tree index on status column
   - Expected improvement: 45ms → 5ms
   - Implementation: Migration to add index
   - Risk: Low (index-only change)
   - Timeline: 30 minutes

   Optimization 3: Implement Pagination (Impact: -10%)
   - Current: Load all results in memory
   - Optimized: Page-based loading
   - Expected improvement: 340MB → 45MB (peak)
   - Implementation: API endpoint changes
   - Risk: Medium (behavior change for clients)
   - Timeline: 4 hours

   Total Expected Improvement:
   - Response time: 245ms → 48ms (80% improvement)
   - Memory: 340MB → 45MB (87% improvement)
   - Meets target: p95 < 200ms ✓
   ```

2. **Risk Assessment**
   ```
   Risk 1: ORM Query Changes
   - Likelihood: Low (well-tested ORM)
   - Impact: Regression in other queries
   - Mitigation: Comprehensive test suite

   Risk 2: Index Creation
   - Likelihood: Very Low
   - Impact: Query performance regression
   - Mitigation: Test on staging first

   Risk 3: API Pagination
   - Likelihood: Medium (client impact)
   - Impact: Client code breaks
   - Mitigation: Backward-compatible pagination
   ```

### Phase 6: Implement Optimizations

**Language Specialist implements sequentially:**

```python
# BEFORE: N+1 Query Problem
def get_users_with_posts():
    users = db.query(User).all()  # Query 1
    for user in users:
        user.posts = db.query(Post).filter(Post.user_id == user.id).all()  # Query 2-N
    return users

# AFTER: Optimized with eager loading
def get_users_with_posts():
    users = db.query(User).options(
        joinedload(User.posts)  # Load all posts in single query
    ).all()
    return users
```

**After each optimization:**
```bash
# Run tests to ensure behavior unchanged
pytest tests/ -v

# Re-profile to measure improvement
python -m cProfile -s cumulative main.py
```

### Phase 7: Measure Improvement

**Performance Profiler validates:**

```
BEFORE OPTIMIZATION:
- p50: 245ms
- p95: 380ms ← TARGET: 200ms
- p99: 520ms
- Memory peak: 340MB
- Throughput: 120 req/s

AFTER OPTIMIZATION 1 (Fix N+1):
- p50: 95ms (61% improvement)
- p95: 145ms (62% improvement) ← MEETS TARGET ✓
- p99: 210ms (60% improvement)
- Memory peak: 280MB (18% improvement)
- Throughput: 320 req/s (167% improvement)

AFTER OPTIMIZATION 2 (Add Index):
- p50: 82ms (16% improvement from opt1)
- p95: 125ms (14% improvement from opt1) ← TARGET ✓
- p99: 195ms (7% improvement)
- Memory peak: 280MB (no change)
- Throughput: 380 req/s (19% improvement)

AFTER OPTIMIZATION 3 (Pagination):
- p50: 45ms (45% improvement from opt2)
- p95: 48ms (62% improvement) ← TARGET ✓
- p99: 85ms (56% improvement)
- Memory peak: 45MB (84% improvement)
- Throughput: 500 req/s (31% improvement)

FINAL RESULT:
- Response time: 245ms → 48ms (80% IMPROVEMENT)
- Memory: 340MB → 45MB (87% IMPROVEMENT)
- Throughput: 120 → 500 req/s (317% IMPROVEMENT)
- Meets all targets: ✓
```

### Phase 8: Load Testing

**Test performance under load:**

```bash
# k6 load test
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },   // Ramp up
    { duration: '5m', target: 500 },   // Stay at 500
    { duration: '2m', target: 0 },     // Ramp down
  ],
};

export default function() {
  let res = http.get('http://localhost:8000/api/users');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });
}
```

**Results:**
```
Load Test Results (500 concurrent users):
- Avg response: 52ms ✓
- Max response: 145ms ✓
- Error rate: 0% ✓
- P95: 75ms ✓
- P99: 120ms ✓
- Throughput: 485 req/s ✓

System Resources:
- CPU: 42% utilization
- Memory: 68MB (peak)
- Database connections: 8/20 (40%)
- All under capacity ✓
```

### Phase 9: Document and Store Results

```bash
cco knowledge store "Performance Optimization: [component]
Before: [metrics]
After: [metrics]
Improvement: [%]
Optimizations:
1. [Optimization 1]: [improvement]
2. [Optimization 2]: [improvement]

Timeline: [dates]
Status: VALIDATED ✓
Load tested: ✓
Regression tested: ✓" \
  --type completion \
  --agent performance-engineer
```

## Optimization Techniques

### Database Optimization
- **Eager Loading**: JOINs instead of N+1 queries
- **Indexing**: B-tree indexes on filter columns
- **Query Optimization**: Analyze query plans
- **Connection Pooling**: Reuse database connections
- **Caching**: Redis for frequently accessed data

### Code Optimization
- **Algorithmic**: Reduce complexity (O(n²) → O(n log n))
- **Memoization**: Cache expensive calculations
- **Lazy Loading**: Load data only when needed
- **Batch Processing**: Process in batches
- **Concurrency**: Parallel execution where applicable

### Infrastructure Optimization
- **Caching Layers**: CDN, Redis, HTTP caching
- **Load Balancing**: Distribute across servers
- **Horizontal Scaling**: More servers
- **Vertical Scaling**: More powerful server
- **Compression**: Gzip for responses

### Frontend Optimization
- **Code Splitting**: Smaller bundles
- **Minification**: Remove unnecessary bytes
- **Lazy Loading**: Load images/scripts on demand
- **Asset Optimization**: Image compression
- **Caching**: Service workers, HTTP caching

## Metrics to Track

| Metric | Target | Tool |
|--------|--------|------|
| **Response Time (p95)** | < 200ms | APM, Load test |
| **Throughput** | 500+ req/s | Load test |
| **Error Rate** | < 0.1% | APM, Monitoring |
| **Memory (peak)** | < 100MB | Profiling |
| **CPU Utilization** | < 70% | Monitoring |
| **Database Query Time** | < 50ms | Database monitoring |

## Best Practices

1. **Measure first** - Know the baseline
2. **Profile accurately** - Identify real bottlenecks
3. **Optimize strategically** - High impact first
4. **Test thoroughly** - No regressions
5. **Validate improvements** - Measure after each change
6. **Load test** - Ensure performance under stress
7. **Document findings** - Share with team
8. **Monitor in production** - Catch regressions early
9. **Iterate** - Optimization is ongoing
10. **Trade-offs** - Understand complexity vs performance

## Output Format

Return comprehensive optimization report with:
- Baseline performance metrics
- Bottleneck analysis with root causes
- Optimization strategy and plan
- Implementation details for each optimization
- Before/after measurements
- Load testing results
- Risk assessment
- Validation and sign-off
- Knowledge base references
