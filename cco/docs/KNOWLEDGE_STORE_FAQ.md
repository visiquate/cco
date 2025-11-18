# Knowledge Store FAQ

**Frequently Asked Questions**
**Version:** 1.0.0
**Last Updated:** November 18, 2025

---

## General Questions

### What is the Knowledge Store?

The Knowledge Store is an embedded vector database system that allows agents to store and retrieve knowledge using semantic similarity search. It replaces the previous Node.js subprocess implementation with a fast HTTP API embedded in the CCO daemon.

### Why should I use it?

- **Fast**: 10-40x faster than subprocess (1-5ms vs 50-200ms)
- **Smart Search**: Vector similarity finds semantically related knowledge
- **Per-Project**: Knowledge automatically scoped to your project
- **Type-Safe**: Strong typing with Rust ensures reliability
- **Simple API**: Standard HTTP/JSON interface

### Who should use it?

Any agent that needs to:
- Store decisions, implementations, or patterns
- Search for relevant historical knowledge
- Track what has been done in a project
- Share knowledge across agent sessions

---

## Getting Started

### How do I check if it's working?

```bash
curl http://localhost:8303/api/knowledge/health \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

Expected response:
```json
{"status": "healthy", "database": "connected"}
```

### Where is my auth token?

```bash
cat ~/.cco/api_token
```

If missing, restart the daemon:
```bash
cco daemon restart
```

### How do I store my first knowledge item?

```bash
curl -X POST http://localhost:8303/api/knowledge/store \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "My first knowledge item",
    "type": "general",
    "project_id": "my-project",
    "session_id": "test",
    "agent": "me"
  }'
```

### How do I search for knowledge?

```bash
curl "http://localhost:8303/api/knowledge/search?q=knowledge+item&project_id=my-project&limit=5" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

---

## API Questions

### What HTTP methods are supported?

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/knowledge/store` | POST | Store single item |
| `/api/knowledge/store/batch` | POST | Store multiple items |
| `/api/knowledge/search` | GET | Vector similarity search |
| `/api/knowledge/query` | POST | SQL-like query |
| `/api/knowledge/stats` | GET | Get statistics |
| `/api/knowledge/health` | GET | Health check |
| `/api/knowledge/cleanup` | DELETE | Delete old entries |
| `/api/knowledge/{id}` | GET | Get specific entry |

### What are the valid knowledge types?

```
decision       - Architecture decisions
architecture   - System design patterns
implementation - Code implementation details
configuration  - Configuration settings
credential     - Credential references (not values!)
issue          - Problems and bugs
general        - Uncategorized knowledge
```

### How do I batch store multiple items?

```bash
curl -X POST http://localhost:8303/api/knowledge/store/batch \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {
        "text": "Item 1",
        "type": "general",
        "project_id": "my-project",
        "session_id": "session-1",
        "agent": "me"
      },
      {
        "text": "Item 2",
        "type": "general",
        "project_id": "my-project",
        "session_id": "session-1",
        "agent": "me"
      }
    ]
  }'
```

### What is the maximum request size?

**Current limits:**
- Single store: Unlimited text length (practical limit ~100KB)
- Batch store: Up to 1000 items per request
- Search limit: Up to 100 results per query

### What happens if I exceed rate limits?

You'll receive HTTP 429 with a `Retry-After` header:

```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "details": {"retry_after": 42}
}
```

Wait for the specified seconds, then retry.

---

## Search Questions

### How does vector similarity search work?

1. Your query text is converted to a 384-dimensional vector using a pre-trained model
2. The database compares this vector to all stored knowledge vectors
3. Results are ranked by cosine similarity (0.0 = unrelated, 1.0 = identical)
4. Only results above the threshold are returned

### What is a good similarity threshold?

```
0.9 - 1.0:  Nearly identical (exact matches, paraphrases)
0.7 - 0.9:  Highly related (same topic, different wording)
0.5 - 0.7:  Somewhat related (similar concepts)
< 0.5:      Weakly related or unrelated
```

**Default threshold:** 0.5 (returns most relevant matches)

**Recommendation:** Start with 0.5, increase if too many irrelevant results.

### Why am I getting no search results?

**Common causes:**

1. **Wrong project_id**: Must match exactly (case-sensitive)
   ```bash
   # Bad: "my-project" vs "my_project"
   # Good: Use exact same project_id for store and search
   ```

2. **Threshold too high**: Lower the threshold
   ```bash
   # Add: &threshold=0.3
   curl "...&threshold=0.3"
   ```

3. **No matching knowledge**: Store some knowledge first
   ```bash
   # Check stats
   curl ".../stats?project_id=my-project" -H "Authorization: Bearer $TOKEN"
   ```

### Can I search across all projects?

Not via vector search (per-project isolation). Use SQL query without project filter:

```bash
curl -X POST http://localhost:8303/api/knowledge/query \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "limit": 20
  }'
```

This returns results from all projects.

### How do I filter search results?

Use query parameters:

```bash
# By type
curl "...?q=query&project_id=proj&type=decision"

# By agent
curl "...?q=query&project_id=proj&agent=architect"

# By date range
curl "...?q=query&project_id=proj&start_date=2025-01-01T00:00:00Z&end_date=2025-12-31T23:59:59Z"

# Combined
curl "...?q=query&project_id=proj&type=decision&agent=architect&limit=10"
```

---

## Data Questions

### Where is my knowledge stored?

```
~/.cco/knowledge/{project_id}/
```

Each project has its own LanceDB database.

### What format is the data in?

Apache Arrow with Parquet on-disk storage. This is a standard, portable format.

### How much disk space does it use?

**Approximate sizes:**
- 1,000 entries: ~5 MB
- 10,000 entries: ~50 MB
- 100,000 entries: ~500 MB

**Formula:** ~5 KB per entry (includes text, vector, metadata)

### Can I back up my knowledge?

Yes, copy the entire directory:

```bash
# Backup
tar -czf knowledge-backup.tar.gz ~/.cco/knowledge/

# Restore
tar -xzf knowledge-backup.tar.gz -C ~/
```

### How do I delete old knowledge?

Use the cleanup endpoint:

```bash
# Dry run (see what would be deleted)
curl -X DELETE http://localhost:8303/api/knowledge/cleanup \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "older_than_days": 90,
    "project_id": "my-project",
    "dry_run": true
  }'

# Actual delete
curl -X DELETE http://localhost:8303/api/knowledge/cleanup \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "older_than_days": 90,
    "project_id": "my-project",
    "dry_run": false
  }'
```

### Can I export my knowledge?

Yes, use the query endpoint to get all entries:

```bash
curl -X POST http://localhost:8303/api/knowledge/query \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "project_id": "my-project",
    "limit": 10000
  }' > knowledge-export.json
```

---

## Metadata Questions

### What can I put in metadata?

Any JSON object:

```json
{
  "metadata": {
    "confidence": 0.95,
    "tags": ["api", "security"],
    "priority": "high",
    "related_files": ["src/auth.py", "src/jwt.py"],
    "related_issues": ["#123", "#456"],
    "custom_field": "custom_value"
  }
}
```

### How do I search by metadata?

Currently, use the query endpoint with metadata filtering:

```bash
curl -X POST http://localhost:8303/api/knowledge/query \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "project_id": "my-project",
    "limit": 20,
    "order_by": "timestamp DESC"
  }'
```

Then filter results in your code:

```python
results = client.query(project_id="my-project", limit=100)
high_priority = [r for r in results['results']
                 if r.get('metadata', {}).get('priority') == 'high']
```

**Note:** Native metadata filtering in search is coming in a future version.

### What is the metadata size limit?

**Recommended:** <10 KB per entry

**Hard limit:** 1 MB (but this will slow down queries)

---

## Performance Questions

### How fast is it?

**Typical latencies:**
- Store: <10ms
- Search: <15ms
- Batch store (100 items): <500ms
- Statistics: <5ms

**Throughput:**
- 100 stores/sec
- 66 searches/sec
- 200 items/sec (batch)

### Why is my first search slow?

The embedding model needs to load on first use (~2-3 seconds). Subsequent requests are fast (<30ms per encoding).

**Solution:** Pre-warm on daemon startup (automatic).

### Can I make it faster?

Yes:

1. **Use batch operations** instead of loops
2. **Reuse HTTP connections** (use session)
3. **Cache search results** in your code
4. **Lower search limit** if you don't need many results
5. **Add more filters** to reduce search space

### How many concurrent requests can it handle?

**Tested:** 1000 concurrent requests

**Limit:** Depends on hardware, but Tokio runtime scales well. Expect 100-500 concurrent requests on typical developer machine.

---

## Troubleshooting

### Connection refused error

**Symptom:**
```
Connection refused (os error 61)
```

**Solutions:**

1. **Check daemon is running:**
   ```bash
   cco daemon status
   ```

2. **Start daemon if not running:**
   ```bash
   cco daemon start
   ```

3. **Check correct port:**
   ```bash
   lsof -i :8303
   ```

### 401 Unauthorized error

**Symptom:**
```json
{"error": "Unauthorized", "code": "UNAUTHORIZED"}
```

**Solutions:**

1. **Check token file exists:**
   ```bash
   cat ~/.cco/api_token
   ```

2. **Verify token in request:**
   ```bash
   # Should show: Authorization: Bearer <long-token>
   curl -v http://localhost:8303/api/knowledge/health \
     -H "Authorization: Bearer $(cat ~/.cco/api_token)" 2>&1 | grep Authorization
   ```

3. **Restart daemon to regenerate token:**
   ```bash
   cco daemon restart
   cat ~/.cco/api_token
   ```

### 400 Bad Request error

**Symptom:**
```json
{"error": "Invalid knowledge type", "code": "INVALID_TYPE"}
```

**Solution:** Use a valid type:
```
decision, architecture, implementation, configuration, credential, issue, general
```

### Search returns wrong results

**Possible causes:**

1. **Incorrect project_id**: Check exact spelling
2. **Threshold too low**: Increase threshold to 0.7+
3. **Query too vague**: Be more specific in your search query
4. **Mixed knowledge**: Unrelated knowledge stored with same type

**Debug:**
```bash
# Check what's in your project
curl "http://localhost:8303/api/knowledge/stats?project_id=my-project" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"

# View recent entries
curl -X POST http://localhost:8303/api/knowledge/query \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "project_id": "my-project",
    "limit": 10,
    "order_by": "timestamp DESC"
  }'
```

---

## Migration Questions

### How do I migrate from Node.js knowledge-manager?

See the [Migration Guide](KNOWLEDGE_STORE_MIGRATION.md) for detailed steps.

**Quick version:**

1. Copy data: `cp -r ~/git/cc-orchestra/data/knowledge/* ~/.cco/knowledge/`
2. Update code to use HTTP API instead of subprocess
3. Test thoroughly
4. Remove old code

### Is my old data compatible?

Yes! LanceDB format is the same. Just copy files to the new location.

### Can I run both systems in parallel?

Yes, during migration:
- Old: `~/git/cc-orchestra/data/knowledge/`
- New: `~/.cco/knowledge/`

They won't conflict. Migrate incrementally if needed.

### What if migration fails?

You can rollback:

1. Restore old code: `git revert HEAD`
2. Restore old data: `cp -r backup/* ~/git/cc-orchestra/data/knowledge/`
3. Document issues and retry later

---

## Security Questions

### Is it secure?

**Local daemon:** Yes, all communication is over localhost.

**Authentication:** Required via Bearer token (file-based).

**Encryption:** No encryption (not needed for localhost).

### Should I store passwords in knowledge?

**NO!** Never store actual credentials in knowledge.

**Instead:** Store references to where credentials are kept:

```python
# Bad
client.store("API Key: sk-abc123...", "credential", "agent")

# Good
client.store("API Key stored in ~/.cco/credentials/api_key", "credential", "agent")
```

### Can other users access my knowledge?

If they have:
1. Access to your machine
2. Read access to `~/.cco/api_token`

Then yes. Use file permissions to protect:

```bash
chmod 600 ~/.cco/api_token
chmod 700 ~/.cco/knowledge/
```

### How is the auth token generated?

```rust
// On daemon startup
use rand::Rng;

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}
```

32-64 characters, cryptographically random.

---

## Advanced Questions

### Can I use it from multiple languages?

Yes! Any language with HTTP client:

- **Python**: `requests` library
- **Rust**: `reqwest` crate
- **JavaScript**: `axios` or `fetch`
- **Bash**: `curl`
- **Go**: `net/http` package
- **Java**: `HttpClient`

See [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md) for examples.

### Can I customize the embedding model?

Not currently. Future versions may support:
- Different models (larger/smaller)
- Custom fine-tuned models
- Multi-lingual models

### Can I query with SQL?

Partially. Use the `/query` endpoint with filters:

```bash
curl -X POST http://localhost:8303/api/knowledge/query \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)" \
  -d '{
    "project_id": "my-project",
    "type": "decision",
    "agent": "architect",
    "limit": 20,
    "order_by": "timestamp DESC"
  }'
```

Full SQL support coming in future version.

### How do I monitor usage?

Use the stats endpoint:

```bash
# Overall stats
curl http://localhost:8303/api/knowledge/stats \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"

# Per-project stats
curl "http://localhost:8303/api/knowledge/stats?project_id=my-project" \
  -H "Authorization: Bearer $(cat ~/.cco/api_token)"
```

### Can I run multiple daemons?

Not currently. One daemon per user. Future versions may support:
- Multiple daemons on different ports
- Distributed deployment
- Multi-tenant isolation

---

## Best Practices

### What should I store as knowledge?

**Good:**
- Architectural decisions and rationale
- Implementation patterns used
- Issues encountered and solutions
- Configuration choices
- Important code snippets
- User requirements

**Bad:**
- Large code files (>100 KB)
- Binary data
- Frequent status updates
- Temporary notes

### How should I structure my text?

**Good structure:**
```
Decision: Use PostgreSQL instead of MongoDB

Rationale: PostgreSQL provides better ACID guarantees,
which are critical for our financial transaction system.
MongoDB's eventual consistency could lead to data loss.

Alternatives considered:
- MongoDB: Rejected due to weak consistency
- MySQL: Rejected due to limited JSON support

Related: #123, #456
```

**Bad structure:**
```
postgres
```

**Tips:**
- Start with type prefix ("Decision:", "Implementation:")
- Include context and rationale
- Mention alternatives if applicable
- Add references (files, issues, PRs)

### When should I search vs query?

**Use search (vector similarity) when:**
- Looking for semantically related knowledge
- Query is natural language ("how did we implement auth?")
- Don't know exact terms used
- Want ranked results by relevance

**Use query (SQL-like) when:**
- Need exact filtering (type, agent, date)
- Want all entries from specific agent
- Need recent entries in chronological order
- Know exactly what you're looking for

### Should I batch or store individually?

**Use batch when:**
- Storing 10+ items at once
- End of long task (summary of multiple steps)
- Migration or bulk import

**Use individual when:**
- Real-time storage after each action
- Interactive workflows
- Single important decision

---

## Getting Help

### Where can I find more documentation?

- [API Reference](KNOWLEDGE_STORE_API.md) - Complete API docs
- [Agent Integration Guide](KNOWLEDGE_STORE_AGENT_GUIDE.md) - How to use from agents
- [Migration Guide](KNOWLEDGE_STORE_MIGRATION.md) - Migrate from Node.js
- [Quick Start](KNOWLEDGE_STORE_QUICK_START.md) - 5-minute tutorial
- [Architecture](KNOWLEDGE_STORE_ARCHITECTURE.md) - Technical deep dive

### How do I report a bug?

1. Check daemon logs: `~/.cco/logs/daemon.log`
2. Reproduce with minimal example
3. Include version: `cco --version`
4. File GitHub issue with details

### How do I request a feature?

File a GitHub issue with:
- Use case description
- Current workaround (if any)
- Expected behavior
- Priority (nice-to-have vs critical)

---

**Last Updated:** November 18, 2025
**Version:** 1.0.0
**Maintained by:** CCO Team
