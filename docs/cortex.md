# CCO Cortex: Repo-Shared Self-Improving Memory

Cortex is CCO's repo-scoped memory system. Memories are plain Markdown files
committed under `.cco/cortex/memories/` (one file per memory), so every developer,
worktree, and agent that has the repo can benefit from them, and improvements propagate
through normal `git push`/`pull`. A local, gitignored `.cco/cortex/.cache/` directory
holds the derived vector index (LanceDB) and SQLite graph built from those files.

**Cortex is enabled by default.** The full self-improving loop runs automatically
across ALL your Claude Code repos: recall/auto-prime at session start; a daily
maintenance worker that incrementally captures new transcript content into per-repo
memories and consolidates near-duplicates; each repo that accumulates memories gets a
reviewable `cortex/updates` PR (never a direct write to your main/working branch).

To fully disable Cortex (including recall): `cco config set cortex_enabled false`.

---

## Quickstart

Cortex runs automatically with no configuration required. The daily worker discovers
your Claude Code repos, captures new transcript content incrementally, and consolidates
memories. Each batch of new memories surfaces as a `cortex/updates` pull request for
your review.

```bash
# Check status across discovered repos
cco cortex status

# Seed from existing Claude Code transcript history now (dry-run first)
cco cortex bootstrap --dry-run
cco cortex bootstrap

# Pin Cortex to a single repo instead of all discovered repos
cco config set cortex_repo_root /absolute/path/to/your/repo

# Exclude a sensitive repo from all Cortex operations
cco config set cortex_denylist secret-repo,another-repo

# To fully disable Cortex (including recall):
cco config set cortex_enabled false
```

After bootstrap completes, Cortex opens a `cortex/bootstrap` pull request on your repo
with the extracted memories. Review and merge to make them active. After the PR is merged,
run `cco cortex reindex` once to rebuild the local cache.

---

## How It Works

Cortex implements five core capabilities. The sections below describe each.

### Foundation: committed memory store and semantic recall

Memories live in `.cco/cortex/memories/` as individual Markdown files with YAML
frontmatter. Each file carries a `kind` (decision, bug, pattern, constraint, gotcha,
reference), a `title`, `tags`, `confidence`, and a `utility_tier` (hot, warm, cold).
The local `.cache/` directory holds the LanceDB vector index and an SQLite typed graph
derived from these files. A `manifest.json` tracks content hashes so a fresh clone or
worktree rebuilds only what changed. `cco cortex reindex` forces a full rebuild.

Because each memory is a separate file named with a content-derived hash suffix,
concurrent agents can add memories without touching the same file, and git auto-merges
additions cleanly.

### Capture, extract, redact, and PR

On task completion (success or error), at `pre_compaction`, and at `Stop`/`SubagentStop`
events, CCO enqueues a job into `.cache/queue.db`. An extraction worker drains the queue:

1. **Redact** the payload - credentials, tokens, and keys are scrubbed via regex and
   entropy detection; PHI heuristics cover SSN, email, phone, date-of-birth, and ICD-10
   patterns. Projects on the `cortex_denylist` are dropped entirely.
2. A **haiku** subagent receives the redacted content and returns zero or more candidate
   memories in structured output.
3. Candidates are deduplicated against existing memories by embedding similarity.
4. Surviving candidates are written as new `.md` files in the working tree.

A **secret/PHI scan gate** runs before any commit. New files are batched onto a
`cortex/updates` branch and opened (or appended) as a single rolling pull request. A
reviewer-tier agent pre-screens the PR and posts its assessment. A human makes the final
merge. Merge is when the memory becomes shared across all checkouts.

### Bootstrap from transcript history

`cco cortex bootstrap` performs a machine-wide sweep of `~/.claude/projects/` to seed
each project's cortex from its own Claude Code transcripts. It resolves encoded project
paths back to their real directories, maps worktrees to their canonical repo root via
`git rev-parse --git-common-dir`, and skips projects that cannot be resolved, are
non-repos, or appear in the denylist. Each resolved repo's transcripts populate only
that repo's `.cco/cortex/` - there is no cross-project memory pool. One
`cortex/bootstrap` PR is opened per resolved repo, and a single machine-wide summary
reports populated vs. skipped project counts.

### Auto-prime: inject relevant memories at session start

At `session_start`, Cortex runs a hybrid search (BM25 + vector) over the committed
memories using the repo context and the initial user prompt as the query. The top-K
results within the configured `cortex_token_budget` are injected as a context block
before the session proceeds. Which memories were injected is logged to `.cache/stats.db`
for use by the utility-tracking signal.

### Nightly consolidation: cluster, merge, decay, and promote

A daemon background worker runs daily (enabled by default, gated on
`cortex_consolidate` not being set to `false`). It:

- Clusters near-duplicate memories by embedding similarity.
- Spawns haiku subagents to merge each cluster into a single canonical memory.
- Decays stale low-utility memories: hot to warm to cold to archive.
- Promotes memories that appear consistently in successful sessions.
- Rebuilds `INDEX.md` from the current set of committed memories.
- Only acts on repos that already have a `.cco/cortex/memories/` directory (no cost for
  repos with no memories yet).

All changes surface as a reviewable diff on the `cortex/updates` PR. Live runtime
statistics (`access_count`, `injected_count`, `last_used`) are kept in `.cache/stats.db`
and are never committed, so the git history stays clean.

### Incremental daily capture

The maintenance worker persists a "last successful capture" timestamp at
`~/.cco/cortex_last_capture`. On the first run, the capture window falls back to the
last `CORTEX_CAPTURE_LOOKBACK_DAYS` (7 days). Every subsequent run captures only
transcript content modified since the previous cycle, keeping per-cycle LLM cost
proportional to new activity rather than total history.

---

## CLI Reference

All `cco cortex` subcommands operate on the repository identified by `cortex_repo_root`
in your config, or on all discovered repos when `cortex_repo_root` is unset.

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

### `cco cortex reindex`

Force a full rebuild of `.cco/cortex/.cache/` from the committed Markdown files.
Use this after merging a cortex PR or after a fresh clone.

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
| `cortex_autoprime` | bool | unset (on by default) | Inject the most relevant memories at session start within the token budget. Runs automatically when `cortex_enabled` is not false; set to `false` to disable only recall while leaving other write paths active. |
| `cortex_consolidate` | bool | true | Run the nightly consolidation worker (cluster, merge, decay, promote). Enabled by default; set to `false` to disable. Requires `cortex_enabled = true`. |
| `cortex_scope` | string | `"repo"` | Scope for cortex operations: `"repo"` or `"all"`. |
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
<repo>/
  .cco/
    cortex/
      memories/
        <kind>-<slug>-<shorthash>.md   # committed: one memory per file
      INDEX.md                          # committed: generated table of contents
      .cache/                           # gitignored: derived, rebuildable
        vectors.lance/                  #   LanceDB vector index
        graph.db                        #   SQLite typed graph
        stats.db                        #   access/inject counters (never committed)
        queue.db                        #   pending extraction jobs
        manifest.json                   #   content-hash to index staleness map
        watermarks.json                 #   last-processed transcript offsets

~/.cco/
  cortex_last_capture                   # RFC 3339 timestamp of last successful capture
```

The `.cache/` directory is self-gitignoring: a `.gitignore` file containing `*` is
committed inside it so the cache contents are never accidentally staged. Everything in
`.cache/` is derived from the committed `.md` files and can be rebuilt at any time with
`cco cortex reindex`.

---

## Safety and Privacy

**Recall is read-only; write paths run automatically but are fully auditable.** The
auto-prime/recall path reads already-committed, already-redacted memory files and makes
no LLM call and commits nothing. Every LLM extraction call and every write/commit/
consolidate action surfaces as a reviewable pull request before any memory reaches a
shared branch. To fully disable all Cortex activity, set `cortex_enabled = false`.

**Layered redaction before the LLM:** The extraction worker runs a mandatory redaction
pass on all input content before any subagent sees it. This pass covers:

- Credentials, API keys, and tokens (regex + entropy-based detection).
- PHI heuristics: SSN, email addresses, phone numbers, dates of birth, and ICD-10 codes.
- Denylisted projects are dropped entirely without any extraction.

**Secret-scan gate before any commit:** The PR commit is blocked if the scanner detects
any credential or flagged pattern in the candidate memory files.

**Reviewed merge:** A reviewer-tier agent pre-screens every cortex PR and posts its
assessment; a human makes the final merge before memories reach a shared branch.

**Residual risk (accepted by the operator):** PHI redaction is best-effort and
pattern-based. It is not a guarantee. The org policy is "Never Process PHI." Enabling
the extraction write path on a PHI-bearing repository is the operator's explicit opt-in
risk. Recall alone reads only already-committed, already-redacted memory files and makes
no LLM call.

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
