Hey there, welcome to Claude Orchestra!

If you're here, you probably code with Claude Code and have ever wondered: *How much is this costing me?* or *Is my context efficient?* or *Could I automate more of this work with specialized agents?* If any of that rings a bell, you're in the right place.

---

## What is CCO?

Claude Orchestra (CCO) is your local sidekick for Claude Code. It's a small command-line tool that runs quietly in the background and does three things really well:

1. **Tracks your AI spending in real time** — Every token you use is logged locally (on your machine, never uploaded unless you opt in). You get instant visibility into what things cost.
2. **Gives you a fleet of 117 specialist agents** — Rather than doing everything yourself at the top tier, CCO lets you delegate work to cheaper, focused agents that excel at specific tasks (documentation, code review, security, testing, etc.). Plus a live dashboard to manage them.
3. **Shrinks your token bills quietly** — CCO bundles compression tools (RTK, lean-ctx, Headroom) that cut token use 60–90% on typical tasks. You see the savings tracked in real time.

All of this runs locally. Your data, your machine, your control.

---

## Why you'd want this

**Surprise AI bills:** Claude Code sessions add up fast, and without visibility, you don't know where the money's going. CCO shows you exactly.

**No cache insight:** Prompt caching can save a fortune, but you don't know if it's working. CCO shows you hit rates and estimated savings per model tier.

**Manual delegation:** If you're doing all the coding yourself with Opus, you're paying premium rates for tasks that could be handled by a specialist. CCO's delegation nudge reminds you to use the right tool for the job.

**Token bloat:** Bash output, logs, and verbose responses eat tokens. CCO's compression tools trim the noise automatically.

---

## Getting started (one command)

```bash
curl -fsSL https://raw.githubusercontent.com/visiquate/cco/main/install.sh | bash
```

That's it. The binary lands in `~/.local/bin/cco` and updates itself with `cco update`.

Next, run the health check (it'll fix things automatically):

```bash
cco doctor --fix
```

This verifies Claude Code is installed, starts the daemon, and optionally sets up compression tools.

---

## Day-to-day commands

Here are the handful of commands you'll actually use:

**`cco`** — Launch Claude Code through CCO. Everything gets tracked automatically.

**`cco tui`** — The live dashboard. Shows your spend, cache hit rate, active agents, and more. Press `?` for keybindings.

**`cco cost dashboard`** — Text summary: total spend, breakdown by model, trends.

**`cco cost cache --period today`** — Cache efficiency: how much you saved with prompt caching.

**`cco cost gate --max-usd 10.00`** — Budget guard for CI/CD: exits with code 1 if you're over budget.

**`cco doctor --fix`** — Health check. Repairs issues automatically.

---

## What to expect

**It runs quietly in the background.** Once you start the daemon (`cco daemon start`), it stays out of your way.

**Cost tracking is instant and offline.** No cloud services, no data uploaded (unless you opt in for telemetry). Everything lives in a local DuckDB file.

**First run might be slower.** The daemon loads a small semantic embedding model for code search and context compression. After that, it's fast.

**Privacy by default.** Prompts, responses, and code never leave your machine. Aggregate telemetry (token counts and costs — no text) is opt-out. Transcript upload is off by default and must be explicitly turned on.

---

## Cool stuff to explore

**The TUI cockpit** (`cco tui`) is your command center. Five pages: Dashboard (spend + sparklines), Activity (live event tail), Cache (efficiency), Delegation (agent work distribution), and Agents (spawn/kill agents with a form). Press `p` to filter by project, `t` to change time range, `n` to spawn an agent. Full guide at [`docs/tui.md`](docs/tui.md).

**Cache buster detection** (`cco cost cache`) finds cache writes that were never read — wasted money. This is hard to spot manually but CCO finds it automatically.

**Token compression** — RTK is included by default and cuts Bash output 60–90%. You can also opt into lean-ctx (semantic compression) or Headroom (API-boundary caching). See `docs/integrations.md` for the full tour.

**Delegation nudges** — When you're doing implementation work inline with Opus, the system nudges you to use a Sonnet or Haiku specialist instead. Non-blocking, rate-limited, and it learns from your preferences.

**MCP server** — Register CCO as an MCP server in Claude Desktop or Cursor. Other AI tools can then query your costs, agents, and knowledge graph without the daemon being online. See [`docs/mcp-and-control-plane.md`](docs/mcp-and-control-plane.md).

**Multi-tool tracking** — `cco codex` launcher lets you use Codex for code review with CCO's cost tracking. All the savings and metrics flow back into your dashboard.

---

## Getting help

**`cco doctor`** — Diagnoses problems and suggests fixes. Usually solves issues automatically with `--fix`.

**`docs/` folder** — Detailed guides for [TUI usage](docs/tui.md), [integrations](docs/integrations.md), [cost metrics](docs/cost-metrics.md), and [telemetry](docs/telemetry.md).

**GitHub Issues** — Found a bug? Have a feature idea? GitHub issues for the main repo: https://github.com/langstons/cco/issues

---

## You've got this

You're now in control of your AI coding spend, have a toolkit for smarter delegation, and can watch your token compression happen in real time. No surprise bills. No guessing. Just you, Claude, and visibility.

Questions? Start with `cco doctor`. Want to go deeper? Read `docs/tui.md` and `docs/integrations.md`.

Happy coding.

---

## License

MIT — see [LICENSE](LICENSE).
