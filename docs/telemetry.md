# Telemetry and Privacy

CCO collects aggregate metrics (token counts, costs) and optionally full conversation transcripts. This guide explains what is collected, what is sent, and how to control both.

## Privacy Posture

CCO separates telemetry into two categories:

1. **Aggregate metrics** (ON by default in official builds) — token counts and cost data *only*, no text
2. **Full transcripts** (OFF by default, opt-in) — complete prompts and responses, for debugging and analysis

### Official vs. Community Builds

- **Official builds** (released via GitHub Actions) have embedded ingest credentials and can send telemetry.
- **Community builds** (compiled from source without the ingest credentials) do not send anything.

To check your build:

```bash
cco version
```

## What Gets Sent

### Aggregate Metrics (Token/Cost Data)

Sent every 15 minutes to the VisiQuate ingest endpoint when `telemetry_enabled = true` (the default).

**Contains:**

- Conversation metadata: `session_id`, `project_name`, `model`, `event_ts`
- Token counts: `input_tokens`, `output_tokens`, `cache_write_tokens`, `cache_read_tokens`
- Cost data: `total_cost` (computed locally; no pricing secrets are sent)
- Agent metrics: `agent_name`, `task_type`, `success`, `duration_ms`, `model_tier`

**Does NOT contain:**

- Prompt text
- Response text
- API keys or secrets
- System prompts
- User data beyond the token counts

**Example aggregate payload** (NDJSON line):

```json
{
  "event_type": "conversation",
  "session_id": "conv-abc123",
  "project_name": "my-project",
  "model": "claude-opus-4-5",
  "input_tokens": 10500,
  "output_tokens": 2300,
  "cache_write_tokens": 0,
  "cache_read_tokens": 800,
  "total_cost": 0.065,
  "event_ts": 1707355200
}
```

### Full Transcripts (Prompt + Response)

Sent when `telemetry_upload_transcripts = true` (requires explicit opt-in).

**Contains:**

- Entire `~/.claude/projects/*/*.jsonl` files (gzipped)
- All prompts, responses, tool calls, and metadata

**When sent:**

- Only files modified since the last upload cycle
- Gzipped and transmitted with the same authentication as aggregate metrics

**Opt-in note:** Transcript upload only takes effect when `telemetry_enabled` is also on.

## Configuration

### Configuration Hierarchy

Settings are resolved in priority order (top wins):

```
1. MDM / Managed Config (system-level, admin-locked)
   ↓ (if no MDM, fall through to...)
2. Environment Variables (CCO_TELEMETRY_*)
   ↓ (if no env var, fall through to...)
3. User Config (~/.cco/config.toml)
   ↓ (if no config, use...)
4. Default (aggregate telemetry ON, transcripts OFF)
```

### User Config

Edit `~/.cco/config.toml`:

```toml
# Aggregate telemetry: on by default (opt-out model).
# Set to false to disable token/cost uploads.
telemetry_enabled = false

# Transcript upload: off by default (opt-in model).
# Set to true to enable full prompt/response uploads.
telemetry_upload_transcripts = true
```

### Environment Variables

Override config at runtime (for automation, CI, containerized deployments):

```bash
# Disable aggregate telemetry
export CCO_TELEMETRY_DISABLED=1

# Enable transcript upload
export CCO_TELEMETRY_TRANSCRIPTS=1

# Then run the daemon
cco daemon start
```

**Note:** Environment variables are read once at daemon startup. Restart the daemon to pick up changes.

### MDM / Managed Configuration

Administrators can enforce org-wide telemetry policy via a system-level config file. Users cannot override settings locked by MDM.

#### macOS

Create `/Library/Application Support/cco/managed.toml` (requires admin write access):

```toml
# Force aggregate telemetry on for the entire org
telemetry_enabled = true

# Force transcript upload on (for audit/compliance)
telemetry_upload_transcripts = true
```

#### Linux

Create `/etc/cco/managed.toml`:

```toml
telemetry_enabled = false
telemetry_upload_transcripts = false
```

#### Windows

Create `C:\ProgramData\cco\managed.toml`:

```toml
telemetry_enabled = true
```

#### Verification

The daemon logs when managed config is loaded:

```
Metrics export enabled (org=, user=, host=, every 15m, transcripts=true)
```

If a setting is locked by MDM, it cannot be changed via environment variable or user config.

## Settings Reference Table

| Setting | Config Key | Env Var | Default | Can User Override? |
|---------|-----------|---------|---------|-------------------|
| Aggregate telemetry | `telemetry_enabled` | `CCO_TELEMETRY_DISABLED=1` | `true` | Yes (unless MDM locked) |
| Transcript upload | `telemetry_upload_transcripts` | `CCO_TELEMETRY_TRANSCRIPTS=1` | `false` | Yes (unless MDM locked) |

**Key points:**

- `CCO_TELEMETRY_DISABLED=1` *disables* aggregate telemetry (flag name is negative)
- `CCO_TELEMETRY_TRANSCRIPTS=1` *enables* transcript upload (flag name is positive)
- Transcripts are always off when aggregate telemetry is disabled (even if explicitly enabled)
- MDM-locked settings override all user/env choices

## How Telemetry is Sent

### Transport and Auth

- **Protocol**: HTTPS POST to VisiQuate ingest Worker
- **Authentication**: HMAC-SHA256 signature (key embedded at build time)
- **Headers**:
  - `X-CCO-Org`: organization slug
  - `X-CCO-User`: local username (sanitized)
  - `X-CCO-Host`: hostname (sanitized)
  - `X-CCO-Kind`: "aggregates" or "transcripts"
  - `X-CCO-Filename`: batch filename (e.g., `conversations-20260214T150230Z.ndjson`)
  - `X-CCO-Timestamp`: UTC RFC 3339 timestamp
  - `X-CCO-Signature`: HMAC-SHA256 hex digest

### Watermarking

The daemon tracks what has been sent via `~/.cco/metrics_export_state.json` so it only uploads new data:

```json
{
  "conversations_last_at": "2026-02-14T15:02:30Z",
  "api_metrics_last_ts": 1707955350,
  "transcripts": {
    "/Users/me/.claude/projects/cc-orchestra/conv-abc.jsonl": 1707955350
  },
  "last_run_at": "2026-02-14T15:03:00Z",
  "last_run_uploads": 3,
  "last_run_bytes": 45821
}
```

On daemon restart, it resumes from these watermarks—no re-uploads.

### Cycle Timing

- **First cycle**: 60 seconds after daemon startup (to let data settle)
- **Subsequent cycles**: every 15 minutes
- **Transient errors**: silently retried next cycle
- **Permanent errors** (4xx): marked as uploaded to avoid retry storms

## Data Retention and Processing

- **Aggregate metrics**: stored in VisiQuate's D1 database; used for dashboards and usage reports
- **Transcripts**: stored in R2 object storage under a user/host prefix; encrypted at rest

The ingest endpoint (VisiQuate Worker) validates signatures, deduplicates, and routes data to the backend for storage. Dashboards can query aggregate metrics directly from D1 without re-parsing raw data.

## Common Scenarios

### Scenario 1: Developer Wants No Telemetry

Add to `~/.cco/config.toml`:

```toml
telemetry_enabled = false
```

Result: No data sent; daemon starts and runs normally.

### Scenario 2: Team Wants to Collect Transcripts for Audit

Admins deploy a managed config:

**Linux/macOS:**

```toml
# /etc/cco/managed.toml or /Library/Application Support/cco/managed.toml
telemetry_enabled = true
telemetry_upload_transcripts = true
```

Result: All machines on the team send both metrics and transcripts. Users cannot disable this.

### Scenario 3: CI Pipeline with No Telemetry

Set environment variables in the CI job:

```bash
export CCO_TELEMETRY_DISABLED=1
./target/release/cco daemon start
```

Result: Metrics are not sent in CI. Works in both official and community builds.

### Scenario 4: Opt-In Transcripts for Debugging

Enable transcripts temporarily:

```bash
export CCO_TELEMETRY_TRANSCRIPTS=1
cco daemon restart
```

Check that they're being collected (should see logs):

```
Metrics export enabled (org=..., transcripts=true)
```

Disable again when done by restarting without the env var.

## Troubleshooting

### "Metrics export: configured-but-empty values, refusing to spawn"

**Cause**: The binary was compiled without ingest credentials (community build).

**Fix**: Use an official release or accept that telemetry won't be sent.

### Transcripts not being uploaded

**Check**:

1. Is aggregate telemetry enabled? Run `cco cost dashboard` — if it succeeds, telemetry is working.
2. Is transcript upload actually enabled? Check the daemon log: `grep "transcripts=" ~/.cco/daemon.log`
3. Are conversations in `~/.claude/projects/`? If empty, nothing to upload.

### "HTTP 400 — Bad Signature"

**Cause**: HMAC signature mismatch (may indicate clock skew or corrupted binary).

**Fix**:

1. Check that the machine's clock is accurate: `date`
2. Restart the daemon
3. Verify the binary wasn't corrupted: `cco version`

### I disabled telemetry but data is still being sent

**Check**:

1. Is an MDM config in place? It overrides user settings. Check `/Library/Application Support/cco/managed.toml` (macOS) or `/etc/cco/managed.toml` (Linux).
2. Did you restart the daemon? Settings are read at startup, not dynamically.

## See Also

- [Cost and Metrics Analysis](cost-metrics.md) — how token data is collected and priced
- [User Configuration](../src/user_config.rs) — source reference for config fields
- [Metrics Export](../src/daemon/metrics_export.rs) — implementation details
