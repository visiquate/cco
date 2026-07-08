# CCO Budget Configuration

## Trigger
`/cco-budget`

Show current budget configuration and spending status relative to thresholds.

## Purpose
This command provides an overview of budget settings and current spending:
- Daily and weekly budget thresholds
- Current period spending vs budget limits
- Budget burn rate and time-to-limit projection
- Instructions for configuring budgets

## Usage

```
You: /cco-budget

💳 Budget Configuration & Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Current Settings (from ~/.cco/config.toml):

Daily Budget:   $20.00
Weekly Budget:  $100.00
Enforcement:    Alert on exceed (soft limit)

Spending Status:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Today:
  Spent: $2.47 / $20.00
  Remaining: $17.53
  Status: ✅ OK (12%)

This Week:
  Spent: $12.34 / $100.00
  Remaining: $87.66
  Status: ✅ OK (12%)

Burn Rate Analysis:
  Avg daily: $2.05 (last 6 days)
  Projection: $13.97 / week
  Trend: ↓ 3% (within normal)

When Budget Limits Trigger:
  Alert: When spending exceeds threshold
  Action: Haiku agents pause, use Opus for critical decisions only
  Manual override: Possible via 'cco cost gate --reset'
```

## When to Use

Use `/cco-budget` when you want to:
- Check budget configuration
- Monitor spending vs limits
- Adjust daily/weekly thresholds
- Understand budget enforcement behavior
- See burn rate and projections

## Configuring Budgets

Edit `~/.cco/config.toml`:

```toml
[budget]
# Daily spending limit (in USD)
daily_budget_usd = 20.0

# Weekly spending limit (in USD)
weekly_budget_usd = 100.0

# Action when budget exceeded: "alert" (warn) or "block" (halt)
on_exceed = "alert"
```

Restart daemon after editing:
```bash
launchctl kickstart -kp gui/$(id -u)/com.anthropic.cco.daemon
```

## Implementation

To use this command, ask Claude to:
1. Run `cco cost dashboard` to show current spending
2. Display the budget configuration from the config file
3. Show current period totals and remaining budget
4. Calculate and show burn rate / time-to-limit projection
5. Explain how to update budget settings in the config file

## Notes

- Budget enforcement is per-user (separate config per machine)
- Daily budget resets at midnight UTC
- Weekly budget resets every Sunday at 00:00 UTC
- Alerts are non-blocking; can continue working
- "block" mode will pause non-critical agent spawning
- Budget can be temporarily bypassed with `cco cost gate --reset`
