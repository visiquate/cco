# VectorDB Architecture Comparison

Visual comparison of current vs proposed knowledge storage architecture.

---

## Current Architecture (With VectorDB)

```
┌─────────────────────────────────────────────────────────────┐
│                        User                                 │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                   CCO Daemon (Rust)                         │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Knowledge API Endpoint                              │  │
│  │  /api/knowledge/store                                │  │
│  │  /api/knowledge/search                               │  │
│  └────────────────────┬─────────────────────────────────┘  │
│                       │                                     │
│                       │ subprocess spawn                    │
│                       ▼                                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  EXTERNAL NODE.JS PROCESS                            │  │
│  │                                                       │  │
│  │  node knowledge-manager.js store/search              │  │
│  │                                                       │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  LanceDB (vectordb npm package)                │  │  │
│  │  │                                                 │  │  │
│  │  │  - Requires Node.js runtime                    │  │  │
│  │  │  - Requires npm install                        │  │  │
│  │  │  - 384-dim hash-based vectors                  │  │  │
│  │  │  - Separate process lifecycle                  │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│            data/knowledge/{repo-name}/                      │
│            - LanceDB files (.lance)                         │
│            - Binary format                                  │
│            - 6.5MB total                                    │
└─────────────────────────────────────────────────────────────┘

EXTERNAL DEPENDENCIES:
  - Node.js runtime (150MB+)
  - vectordb npm package
  - npm install required
  - Subprocess management overhead

PERFORMANCE:
  - Store: 50ms (subprocess spawn)
  - Search: 230ms (subprocess + Node.js startup)
  - Stats: 150ms

SHIPPING:
  - Rust binary + Node.js runtime + npm packages
  - Multi-component distribution
  - Platform-specific Node.js builds
```

---

## Proposed Architecture (Embedded SQLite)

```
┌─────────────────────────────────────────────────────────────┐
│                        User                                 │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                   CCO Daemon (Rust)                         │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Knowledge API Endpoint                              │  │
│  │  /api/knowledge/store                                │  │
│  │  /api/knowledge/search                               │  │
│  └────────────────────┬─────────────────────────────────┘  │
│                       │                                     │
│                       │ direct function call                │
│                       ▼                                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  PersistenceLayer (Rust)                             │  │
│  │                                                       │  │
│  │  persistence.store_knowledge(entry)                  │  │
│  │  persistence.search_knowledge(query)                 │  │
│  │                                                       │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  Embedded SQLite (sqlx)                        │  │  │
│  │  │                                                 │  │  │
│  │  │  - Zero external dependencies                  │  │  │
│  │  │  - Compiled into Rust binary                   │  │  │
│  │  │  - FTS5 full-text search                       │  │  │
│  │  │  - WAL mode for concurrency                    │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│            ~/.cco/daemon.db                                 │
│            ├── api_metrics                                  │
│            ├── hourly_aggregations                          │
│            ├── daily_summaries                              │
│            └── knowledge_store          ← NEW TABLE         │
│                                                             │
│            - Single SQLite database                         │
│            - Human-readable (SQL)                           │
│            - Same 6.5MB data                                │
└─────────────────────────────────────────────────────────────┘

EXTERNAL DEPENDENCIES:
  - None (SQLite embedded in Rust binary)

PERFORMANCE:
  - Store: 2ms (direct SQLite write)
  - Search: 5-10ms (FTS5 full-text search)
  - Stats: 1ms (indexed SQL query)

SHIPPING:
  - Single Rust binary (~5MB)
  - Zero external dependencies
  - Cross-platform (compile once)
```

---

## Side-by-Side Comparison

### Architecture Complexity

| Aspect | Current (VectorDB) | Proposed (SQLite) |
|--------|-------------------|-------------------|
| **Components** | Rust daemon + Node.js process + LanceDB | Rust daemon only |
| **External Deps** | Node.js runtime + npm packages | None |
| **Process Model** | Multi-process (subprocess spawn) | Single process |
| **Communication** | IPC via subprocess | Direct function calls |
| **Database** | LanceDB (binary format) | SQLite (embedded) |

### Performance Profile

| Operation | Current (VectorDB) | Proposed (SQLite) | Speedup |
|-----------|-------------------|-------------------|---------|
| **Store** | 50ms | 2ms | **25x faster** |
| **Search** | 230ms | 5-10ms | **23-46x faster** |
| **Stats** | 150ms | 1ms | **150x faster** |
| **Startup** | 500ms (Node.js) | 0ms (embedded) | **Instant** |

### Distribution Size

| Component | Current | Proposed | Reduction |
|-----------|---------|----------|-----------|
| **Rust Binary** | ~5MB | ~5MB | 0 |
| **Node.js Runtime** | ~150MB | 0 | **-150MB** |
| **npm Packages** | ~50MB | 0 | **-50MB** |
| **Total** | ~205MB | ~5MB | **-200MB (97% smaller)** |

### Search Capabilities

| Feature | Current (VectorDB) | Proposed (SQLite) |
|---------|-------------------|-------------------|
| **Text Search** | Hash-based vectors | FTS5 full-text search |
| **Semantic Search** | ❌ (hash, not semantic) | ❌ (not needed) |
| **Metadata Filters** | ✅ (post-filter) | ✅ (indexed SQL) |
| **Performance** | 230ms | 5-10ms |
| **Scalability** | Limited by Node.js | Millions of entries |

### Developer Experience

| Aspect | Current (VectorDB) | Proposed (SQLite) |
|--------|-------------------|-------------------|
| **Language** | JavaScript (subprocess) | Rust (native) |
| **Type Safety** | ❌ (runtime checks) | ✅ (compile-time) |
| **Error Handling** | Complex (IPC errors) | Simple (Result<T>) |
| **Debugging** | Multi-process (hard) | Single process (easy) |
| **Testing** | Integration tests | Unit + integration |
| **Documentation** | External API | Internal API |

### Operational Characteristics

| Aspect | Current (VectorDB) | Proposed (SQLite) |
|--------|-------------------|-------------------|
| **Backup** | 2 systems (DB + metrics) | 1 system (daemon.db) |
| **Monitoring** | 2 processes | 1 process |
| **Logs** | 2 log sources | 1 log source |
| **Dependencies** | npm security updates | None |
| **Maintenance** | Node.js + Rust | Rust only |

---

## Data Migration Path

### Current Data Structure
```
data/knowledge/
├── knowbe4-api/
│   └── army_knowledge.lance/
│       ├── data/b9a2cf39.lance
│       ├── data/fff37a80.lance
│       └── ... (6 files, ~4MB)
└── automation-demo/
    └── orchestra_knowledge.lance/
        ├── data/63ae213c.lance
        └── data/c5fbc00e.lance

Total: 6.5MB, ~1000 entries
```

### Proposed Data Structure
```
~/.cco/daemon.db

knowledge_store table:
┌─────────────┬──────────────────────┬──────────────┬────────────┐
│ id          │ text                 │ entry_type   │ project_id │
├─────────────┼──────────────────────┼──────────────┼────────────┤
│ decision-1  │ "We chose FastAPI..." │ decision     │ knowbe4    │
│ impl-2      │ "Implemented JWT..."  │ implementation│ knowbe4   │
│ arch-3      │ "Microservices..."    │ architecture │ demo       │
└─────────────┴──────────────────────┴──────────────┴────────────┘

Same data, more efficient storage
```

### Migration Process
```bash
# Step 1: Read from LanceDB
for repo in data/knowledge/*; do
    node knowledge-manager.js export $repo > /tmp/export.json
done

# Step 2: Import to SQLite
cco migrate-knowledge --input /tmp/export.json

# Step 3: Verify
cco knowledge stats
# Should show same counts as original

# Step 4: Archive old data
mv data/knowledge/ data/knowledge.backup/
```

---

## Code Comparison

### Current: Node.js Subprocess Call

```rust
// cco/src/daemon/server.rs (current approach)
async fn handle_knowledge_store(text: String) -> Result<Response> {
    // Spawn Node.js subprocess
    let output = Command::new("node")
        .arg("/path/to/knowledge-manager.js")
        .arg("store")
        .arg(&text)
        .output()
        .await?;

    // Parse JSON response
    let id = parse_json_output(&output.stdout)?;
    Ok(json!({ "id": id }))
}

// Performance: 50ms (subprocess spawn + Node.js startup)
// Error handling: Complex (subprocess errors + JSON parsing)
```

### Proposed: Direct SQLite Call

```rust
// cco/src/daemon/server.rs (proposed approach)
async fn handle_knowledge_store(text: String) -> Result<Response> {
    // Direct persistence layer call
    let entry = KnowledgeEntry {
        text,
        entry_type: "decision".to_string(),
        project_id: "current-repo".to_string(),
        agent: "architect".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        metadata: json!({}),
    };

    let id = persistence.store_knowledge(entry).await?;
    Ok(json!({ "id": id }))
}

// Performance: 2ms (direct SQLite write)
// Error handling: Simple (Result<T> with typed errors)
```

---

## Conclusion

**Current Architecture Problems:**
- ❌ External Node.js dependency (150MB+)
- ❌ Subprocess overhead (50-230ms latency)
- ❌ Complex error handling (IPC)
- ❌ Multi-component distribution
- ❌ No real semantic search (hash-based)

**Proposed Architecture Benefits:**
- ✅ Zero external dependencies
- ✅ 10-20x faster performance
- ✅ Simple error handling (native Rust)
- ✅ Single binary distribution
- ✅ Better full-text search (SQLite FTS5)

**Recommendation:** Migrate to embedded SQLite (Option B)

**See Also:**
- Full analysis: `docs/VECTORDB_ELIMINATION_ARCHITECTURE.md`
- Quick summary: `docs/VECTORDB_ELIMINATION_SUMMARY.md`
