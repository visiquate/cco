# CCO Cortex: Local-First Self-Improving Memory

Cortex is CCO's self-improving memory system. Memories are stored in a machine-local
directory at `~/.cco/cortex/<repo-id>/memories/` (one file per memory), completely
separate from your git repositories. The self-improving loop (capture, consolidation,
recall) runs automatically and entirely locally. When you're ready to share memories
with your team, you explicitly run `cco cortex export` to write them into the repo's
`.cco/cortex/memories/` directory, stage them with git, and commit/push yourself.

**Cortex is enabled by default.** The full self-improving loop runs automatically
across all your Claude Code repos: recall/auto-prime at session start; a daily
maintenance worker that incrementally captures new transcript content and consolidates
near-duplicates. All work happens in your local store. Nothing is written to git
automatically.

To fully disable Cortex (including recall): `cco config set cortex_enabled false`.

---

## Quickstart

Cortex runs automatically with no configuration required. The daily worker discovers
your Claude Code repos, captures new transcript content incrementally, and consolidates
memories all within your local machine-local store. When you want to share memories with
your team, export them to git.

```bash
# Check status of local memories across discovered repos
cco cortex status

# Seed local store from existing Claude Code transcript history (dry-run first)
cco cortex bootstrap --dry-run
cco cortex bootstrap

# Export local memories into the repo for git tracking (stages them with git add)
cco cortex export
cco cortex export --repo /path/to/repo

# Import memories from a repo into your local store
cco cortex import
cco cortex import --repo /path/to/repo

# Pin Cortex to a single repo instead of all discovered repos
cco config set cortex_repo_root /absolute/path/to/your/repo

# Exclude a sensitive repo from all Cortex operations
cco config set cortex_denylist secret-repo,another-repo

# To fully disable Cortex (including recall):
cco config set cortex_enabled false
```

Bootstrap seeds your local store from Claude Code transcript history. Export writes
local memories into the repo when you're ready to commit them. Import and auto-seed
pull committed memories back into your local store so you and your teammates benefit
from shared memories.

---

## How It Works

Cortex implements five core capabilities. The sections below describe each.

### Foundation: local memory store and semantic recall

Memories live in `~/.cco/cortex/<repo-id>/memories/` as individual Markdown files with
YAML frontmatter. Each file carries a `kind` (decision, bug, pattern, constraint,
gotcha, reference), a `title`, `tags`, `confidence`, and a `utility_tier` (hot, warm,
cold). The local `~/.cco/cortex/<repo-id>/cache/` directory holds the LanceDB vector
index and an SQLite typed graph derived from these files. A `manifest.json` tracks
content hashes so rebuilds only reprocess what changed. `cco cortex reindex` forces a
full rebuild.

Because each memory is a separate file named with a content-derived hash suffix,
concurrent extraction and consolidation work without file conflicts.

### Capture, extract, and redact

On task completion (success or error), at `pre_compaction`, and at `Stop`/`SubagentStop`
events, CCO enqueues a job into `~/.cco/cortex/<repo-id>/cache/queue.db`. An extraction
worker drains the queue:

1. **Redact** the payload - credentials, tokens, and keys are scrubbed via regex and
   entropy detection; PHI heuristics cover SSN, email, phone, date-of-birth, and ICD-10
   patterns. Projects on the `cortex_denylist` are dropped entirely.
2. A **haiku** subagent receives the redacted content and returns zero or more candidate
   memories in structured output.
3. Candidates are deduplicated against existing memories by embedding similarity.
4. Surviving candidates are written as new `.md` files in the local store.

All work happens in your local machine. Memories never leave your machine automatically.

### Bootstrap from transcript history

`cco cortex bootstrap` performs a machine-wide sweep of `~/.claude/projects/` to seed
each project's local cortex from its own Claude Code transcripts. It resolves encoded
project paths back to their real directories, maps worktrees to their canonical repo
root via `git rev-parse --git-common-dir`, and skips projects that cannot be resolved,
are non-repos, or appear in the denylist. Each resolved repo's transcripts populate only
that repo's local `~/.cco/cortex/<repo-id>/` - there is no cross-project memory pool.
A single machine-wide summary reports populated vs. skipped project counts.

### Auto-prime: inject relevant memories at session start

At `session_start`, Cortex runs a hybrid search (BM25 + vector) over the committed
memories using the repo context and the initial user prompt as the query. The top-K
results within the configured `cortex_token_budget` are injected as a context block
before the session proceeds. Which memories were injected is logged to `.cache/stats.db`
for use by the utility-tracking signal.

### Nightly consolidation: cluster, merge, decay, and promote

A daemon background worker runs daily (enabled by default, gated on
`cortex_consolidate` not being set to `false`). It:

- Clusters near-duplicate memories by embedding similarity across all repos.
- Spawns haiku subagents to merge each cluster into a single canonical memory.
- Decays stale low-utility memories: hot to warm to cold to archive.
- Promotes memories that appear consistently in successful sessions.
- Rebuilds `INDEX.md` from the current set of local memories.
- Only acts on repos that already have local memories.

All changes happen in the local store. Live runtime statistics (`access_count`,
`injected_count`, `last_used`) are kept in `~/.cco/cortex/<repo-id>/cache/stats.db`
and are never committed.

### Incremental daily capture

The maintenance worker persists a "last successful capture" timestamp at
`~/.cco/cortex_last_capture`. On the first run, the capture window falls back to the
last `CORTEX_CAPTURE_LOOKBACK_DAYS` (7 days). Every subsequent run captures only
transcript content modified since the previous cycle, keeping per-cycle LLM cost
proportional to new activity rather than total history.

---

## CLI Reference

All `cco cortex` subcommands operate on the repository identified by `cortex_repo_root`
in your config, or on the current directory's repo when `cortex_repo_root` is unset.
Machine-wide operations like `status`, `bootstrap`, and consolidation discover and
iterate across all repos.

### `cco cortex status`

Show memory counts by kind and utility tier, index freshness, queue depth, last
consolidation timestamp, and the current config values.

```bash
cco cortex status
```

### `cco cortex search <query>`

Run a hybrid search over committed memories in the current repo.

```bash
cco cortex search "how to handle rate limit retries"
cco cortex search "postgres migration pattern" --limit 5
```

Flags:
- `--limit <N>` - maximum results to return (default: 10)

### `cco cortex export`

Write local memories into the repo's `.cco/cortex/memories/` for git tracking.
Regenerates `INDEX.md`, syncs deletions, and stages changes with `git add`.
You review, commit, and push yourself.

```bash
# Export current repo's local memories
cco cortex export

# Export a specific repo's local memories
cco cortex export --repo /path/to/repo
```

Output shows how many files were written, removed, and staged for commit.

### `cco cortex import`

Load memories from a repo's committed `.cco/cortex/memories/` into your local store.
Also runs automatically on first recall when the local store is empty but the repo
has committed memories (auto-seed).

```bash
# Import current repo's committed memories into local store
cco cortex import

# Import a specific repo's committed memories
cco cortex import --repo /path/to/repo
```

### `cco cortex reindex`

Force a full rebuild of `~/.cco/cortex/<repo-id>/cache/` from the local Markdown files.
Use this after modifying local memory files or to refresh the vector index.

```bash
cco cortex reindex
```

### `cco cortex bootstrap`

Seed this repo's cortex (or all discovered repos) from Claude Code transcript history.

```bash
# Default: current repo only
cco cortex bootstrap

# Estimate scope without writing anything
cco cortex bootstrap --dry-run

# All discovered projects (potentially expensive; estimate with --dry-run first)
cco cortex bootstrap --scope all

# Only transcripts from the past 30 days
cco cortex bootstrap --since 2026-05-19

# Skip additional repos beyond what is in cortex_denylist
cco cortex bootstrap --deny my-private-repo another-repo
```

Flags:
- `--scope <value>` - `this-repo` (default) or `all`
- `--since <ISO-8601-date>` - only process sessions modified after this date
- `--dry-run` - count candidates and report scope; write nothing
- `--deny <repo> ...` - additional repo names or paths to skip (appended to
  `cortex_denylist` from config)

---

## Configuration Reference

All cortex keys are managed via `cco config set/get/unset`. They live in
`~/.cco/config.toml`.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `cortex_enabled` | bool | true | Controls all Cortex features. Unset or `true` = enabled (full loop runs automatically); `false` = fully disabled. To turn off: `cco config set cortex_enabled false`. |
| `cortex_repo_root` | string | unset | Pin Cortex to a single absolute-path git repository instead of all discovered repos. Denylisted repos are skipped even if pinned. When unset, the daily worker discovers all your Claude Code repos automatically. |
| `cortex_autoprime` | bool | unset (on by default) | Inject the most relevant memories at session start within the token budget. Runs automatically when `cortex_enabled` is not false; set to `false` to disable only recall. |
| `cortex_consolidate` | bool | true | Run the nightly consolidation worker (cluster, merge, decay, promote). Enabled by default; set to `false` to disable. Requires `cortex_enabled = true`. |
| `cortex_token_budget` | u32 | unset | Maximum tokens the auto-prime injector may use per session. |
| `cortex_denylist` | string list | unset | Comma-separated repo names or paths the cortex worker must skip entirely. Honored everywhere: discovery, pinned-root, and runtime. |

### Examples

```bash
# Pin Cortex to a single repo (recall still runs on current-cwd repos)
cco config set cortex_repo_root /Users/you/projects/myrepo

# Enable auto-prime with a 2000-token budget
cco config set cortex_autoprime true
cco config set cortex_token_budget 2000

# Disable nightly consolidation
cco config set cortex_consolidate false

# Exclude a sensitive repo from all cortex operations
cco config set cortex_denylist phi-data-repo,customer-data

# Check the current cortex config
cco config get cortex_enabled
cco config show

# Disable Cortex
cco config set cortex_enabled false
```

---

## Storage Layout

```
~/.cco/
  cortex/
    <repo-id>/                          # stable hash of repo root path
      memories/                         # local source of truth (never committed)
        <kind>-<slug>-<shorthash>.md   #   one memory per file
      cache/                            # derived, rebuildable (gitignored)
        vectors.lance/                  #   LanceDB vector index
        graph.db                        #   SQLite typed graph
        stats.db                        #   access/inject counters
        queue.db                        #   pending extraction jobs
        manifest.json                   #   content-hash to index staleness map
        watermarks.json                 #   last-processed transcript offsets
  cortex_last_capture                   # RFC 3339 timestamp of last successful capture

<repo>/
  .cco/
    cortex/
      memories/                         # committed: exported from local store
        <kind>-<slug>-<shorthash>.md   #   one memory per file
      INDEX.md                          # committed: generated table of contents
```

Local memories in `~/.cco/cortex/<repo-id>/memories/` are your machine's source of
truth. The repo's `.cco/cortex/memories/` is written only by the export command and is
meant for sharing with your team via git. Everything in `cache/` is derived from local
`.md` files and can be rebuilt at any time with `cco cortex reindex`.

---

## Safety and Privacy

**Recall is read-only; all work is local.** The auto-prime/recall path reads locally
stored, already-redacted memory files and makes no LLM call. Extraction and consolidation
run locally in your machine-local store. Memories never reach git or leave your machine
automatically. To fully disable all Cortex activity, set `cortex_enabled = false`.

**Layered redaction before the LLM:** The extraction worker runs a mandatory redaction
pass on all input content before any subagent sees it. This pass covers:

- Credentials, API keys, and tokens (regex + entropy-based detection).
- PHI heuristics: SSN, email addresses, phone numbers, dates of birth, and ICD-10 codes.
- Denylisted projects are dropped entirely without any extraction.

**Deliberate export:** When you run `cco cortex export`, memories are written to git
and staged with `git add`. You control when and whether to commit and push. This keeps
sharing explicit and reviewable.

**Residual risk (accepted by the operator):** PHI redaction is best-effort and
pattern-based. It is not a guarantee. The org policy is "Never Process PHI." Enabling
the extraction path on a PHI-bearing repository is the operator's explicit opt-in risk.
Recall alone reads only locally stored, already-redacted memory files and makes no LLM
call.

### Privacy architecture summary

1. Pre-extraction redaction (always, before any API call).
2. Schema discipline: the extractor is instructed to record lessons, never raw payloads.
3. Commit-time scan gate: blocks the PR commit if any pattern is flagged.
4. Reviewed merge: a human sees the diff before it reaches a shared branch.

---

## Health Check

`cco doctor` includes a Cortex check that verifies:

- Is `cortex_enabled` set?
- Is `cortex_repo_root` set and does it exist?
- Is the index fresh?
- Is `.cco/cortex/.cache/` gitignored?
- Is the denylist sane?
- Is there a pending cortex PR?

Run `cco doctor --fix` to auto-repair common issues.
