# Complete Model Override Implementation - Quick Guide

**Status**: Server logic complete, analytics tracking missing
**Estimated Time**: 30 minutes
**Files to Modify**: `/Users/brent/git/cc-orchestra/cco/src/analytics.rs`

---

## Current Compilation Error

```
error[E0599]: no method named `record_model_override` found for struct
              `Arc<AnalyticsEngine>` in the current scope
   --> src/server.rs:240:14
```

**Root Cause**: The method is called in server.rs but not implemented in analytics.rs

---

## Step-by-Step Implementation

### Step 1: Add OverrideRecord struct to analytics.rs

**Location**: After line 17 (after `ApiCallRecord` struct)

**Note**: The imports are already present but unused (lines 3-4):
```rust
use chrono::{DateTime, Utc};  // Already imported
use serde::Serialize;         // Already imported
```

**Add this code**:
```rust
/// Record of a model override event
#[derive(Clone, Debug, Serialize)]
pub struct OverrideRecord {
    pub original_model: String,
    pub override_to: String,
    pub timestamp: DateTime<Utc>,
}
```

---

### Step 2: Add model_overrides field to AnalyticsEngine

**Location**: Line 33 (in the `AnalyticsEngine` struct)

**Current code**:
```rust
pub struct AnalyticsEngine {
    records: Arc<Mutex<Vec<ApiCallRecord>>>,
}
```

**Change to**:
```rust
pub struct AnalyticsEngine {
    records: Arc<Mutex<Vec<ApiCallRecord>>>,
    model_overrides: Arc<Mutex<Vec<OverrideRecord>>>,  // ADD THIS LINE
}
```

---

### Step 3: Update AnalyticsEngine::new() constructor

**Location**: Line 38 (in the `impl AnalyticsEngine` block)

**Current code**:
```rust
pub fn new() -> Self {
    Self {
        records: Arc::new(Mutex::new(Vec::new())),
    }
}
```

**Change to**:
```rust
pub fn new() -> Self {
    Self {
        records: Arc::new(Mutex::new(Vec::new())),
        model_overrides: Arc::new(Mutex::new(Vec::new())),  // ADD THIS LINE
    }
}
```

---

### Step 4: Implement record_model_override() method

**Location**: After line 48 (after the `record_api_call()` method)

**Add this code**:
```rust
/// Record a model override event
///
/// This tracks when a model override rule was applied, storing the original
/// model requested and the model it was overridden to.
pub async fn record_model_override(
    &self,
    original_model: &str,
    override_model: &str,
) {
    let record = OverrideRecord {
        original_model: original_model.to_string(),
        override_to: override_model.to_string(),
        timestamp: Utc::now(),
    };

    let mut overrides = self.model_overrides.lock().await;
    overrides.push(record);
}
```

---

### Step 5: Implement get_override_statistics() method

**Location**: After the `record_model_override()` method you just added

**Add this code**:
```rust
/// Get all override records
///
/// Returns a clone of all recorded model override events for analytics purposes.
pub async fn get_override_statistics(&self) -> Vec<OverrideRecord> {
    let overrides = self.model_overrides.lock().await;
    overrides.clone()
}

/// Get count of overrides by original model
///
/// Returns a HashMap where keys are original model names and values are the
/// number of times that model was overridden.
pub async fn get_override_counts(&self) -> std::collections::HashMap<String, u64> {
    let overrides = self.model_overrides.lock().await;
    let mut counts = std::collections::HashMap::new();

    for record in overrides.iter() {
        *counts.entry(record.original_model.clone()).or_insert(0) += 1;
    }

    counts
}
```

---

### Step 6: Update clear() method to also clear overrides

**Location**: Around line 141 (the existing `clear()` method)

**Current code**:
```rust
pub async fn clear(&self) {
    let mut records = self.records.lock().await;
    records.clear();
}
```

**Change to**:
```rust
pub async fn clear(&self) {
    let mut records = self.records.lock().await;
    records.clear();

    // Also clear override records
    let mut overrides = self.model_overrides.lock().await;
    overrides.clear();
}
```

---

## Verification Steps

### 1. Verify Compilation
```bash
cd /Users/brent/git/cc-orchestra/cco
cargo check
```

**Expected Output**:
```
    Checking cco v0.0.0 (/Users/brent/git/cc-orchestra/cco)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

**Should have ZERO errors** (the 2 unused import warnings should now be gone)

---

### 2. Run Clippy
```bash
cargo clippy
```

**Expected**: No warnings or errors

---

### 3. Build Release
```bash
cargo build --release
```

**Expected**: Successful compilation with binary at `./target/release/cco`

---

### 4. Run Existing Tests
```bash
cargo test
```

**Expected**: All 92 existing tests should still pass

---

## Manual Verification

### Start the Server
```bash
cd /Users/brent/git/cc-orchestra/cco
./target/release/cco run --port 3000
```

**Expected in startup logs**:
```
📋 Loaded 3 model override rules
```

### Test Model Override

**Terminal 1**: Keep server running

**Terminal 2**: Send a request with sonnet model
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
  -d '{
    "model": "claude-sonnet-4.5-20250929",
    "messages": [{"role": "user", "content": "Test"}],
    "max_tokens": 100
  }'
```

**Expected in server logs**:
```
🔄 Model override: claude-sonnet-4.5-20250929 → claude-haiku-4-5-20251001
📝 Processing chat request for model: claude-haiku-4-5-20251001
```

**Key Observations**:
1. The override message should appear FIRST
2. The processing message should show the OVERRIDDEN model (haiku), not the original (sonnet)
3. No errors or panics should occur
4. The response should complete successfully

---

## Optional: Add Statistics Endpoint

If you want to expose override statistics via API, add this to `server.rs`:

### 1. Add response struct (near other response structs around line 178)
```rust
#[derive(serde::Serialize)]
pub struct OverrideStatsResponse {
    total_overrides: u64,
    overrides_by_model: HashMap<String, u64>,
}
```

### 2. Add handler function (near other handlers around line 400)
```rust
/// Override statistics endpoint
async fn override_stats(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<OverrideStatsResponse>, ServerError> {
    let counts = state.analytics.get_override_counts().await;
    let total = counts.values().sum();

    Ok(Json(OverrideStatsResponse {
        total_overrides: total,
        overrides_by_model: counts,
    }))
}
```

### 3. Add route (in run_server() around line 616)
```rust
.route("/api/overrides/stats", get(override_stats))
```

### 4. Test the endpoint
```bash
curl http://localhost:3000/api/overrides/stats
```

**Expected Response**:
```json
{
  "total_overrides": 5,
  "overrides_by_model": {
    "claude-sonnet-4.5-20250929": 3,
    "claude-sonnet-4": 2
  }
}
```

---

## Common Issues

### Issue: Still getting "method not found" error
**Solution**: Make sure you added the method to the `impl AnalyticsEngine` block (not outside it)

### Issue: "field `model_overrides` is never read"
**Solution**: This is expected if you haven't added the optional statistics endpoint yet. The field is used internally by `record_model_override()`.

### Issue: Compilation warnings about unused `DateTime` or `Utc`
**Solution**: Make sure the `OverrideRecord` struct is using `DateTime<Utc>` for the timestamp field.

---

## Testing Checklist

After implementation, verify:

- [ ] Code compiles without errors: `cargo check`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Release build succeeds: `cargo build --release`
- [ ] All existing tests pass: `cargo test`
- [ ] Server starts without errors
- [ ] Startup log shows "Loaded 3 model override rules"
- [ ] Test request shows override log message
- [ ] Override log shows correct original → override mapping
- [ ] Processing log shows overridden model (not original)
- [ ] Response completes successfully
- [ ] (Optional) Statistics endpoint returns valid JSON

---

## Expected Results

### Before Implementation
```
error[E0599]: no method named `record_model_override` found
```

### After Implementation
```
✅ cargo check passes
✅ cargo clippy passes
✅ cargo build --release succeeds
✅ cargo test passes (92 tests)
✅ Server starts successfully
✅ Overrides are applied correctly
✅ Overrides are logged correctly
✅ Overrides are tracked in analytics
```

---

## Summary

**Total Changes Required**: 6 small edits to analytics.rs
**Estimated Time**: 30 minutes
**Complexity**: Low (basic struct and methods)
**Risk**: Low (well-defined, isolated changes)
**Impact**: High (enables cost savings feature)

**Files Modified**:
- `/Users/brent/git/cc-orchestra/cco/src/analytics.rs` (6 edits)

**Files Created**:
- None (all code additions to existing file)

**Dependencies Added**:
- None (using existing chrono and serde imports)

**After completion, the feature will be 100% implemented and ready for production use.**
