# Historic Metrics Testing Guide

## Pre-Testing Verification

### 1. Check Database Exists
```bash
ls -lh ~/.claude/cco_metrics.db
```

### 2. View Database Schema
```bash
sqlite3 ~/.claude/cco_metrics.db ".schema"
```

### 3. Check Migration Status
```bash
sqlite3 ~/.claude/cco_metrics.db "SELECT * FROM claude_history_migration_status;"
```

### 4. View Sample Data
```bash
sqlite3 ~/.claude/cco_metrics.db "SELECT date, model, cost, message_count FROM claude_history_metrics ORDER BY date DESC LIMIT 10;"
```

### 5. Count Records
```bash
sqlite3 ~/.claude/cco_metrics.db "SELECT COUNT(*) as total_records FROM claude_history_metrics;"
sqlite3 ~/.claude/cco_metrics.db "SELECT COUNT(DISTINCT date) as unique_dates FROM claude_history_metrics;"
sqlite3 ~/.claude/cco_metrics.db "SELECT COUNT(DISTINCT model) as unique_models FROM claude_history_metrics;"
```

## API Testing

### 1. Health Check
```bash
curl -s http://localhost:8302/ready | jq '.'
```

### 2. Get Stats API
```bash
curl -s http://localhost:8302/api/stats | jq '{
  total_cost: .project.cost,
  days_of_data: (.chart_data.cost_over_time | length),
  first_day: .chart_data.cost_over_time[0],
  last_day: .chart_data.cost_over_time[-1]
}'
```

### 3. Full Cost Over Time Data
```bash
curl -s http://localhost:8302/api/stats | jq '.chart_data.cost_over_time'
```

### 4. Model Distribution
```bash
curl -s http://localhost:8302/api/stats | jq '.chart_data.model_distribution'
```

## Dashboard Visual Testing

### 1. Open Dashboard
```bash
open http://localhost:8302/
```

### 2. Verify Chart Display
- Check "Cost Over Time" chart shows varying costs (not flat line)
- Verify dates are sequential
- Confirm model distribution matches expectations

### 3. Check for Real Trends
- Look for days with higher/lower costs
- Verify trend line shows actual usage patterns
- Compare with known usage dates

## SQL Queries for Validation

### Daily Totals by Date
```sql
SELECT
    date,
    COUNT(*) as models,
    SUM(cost) as total_cost,
    SUM(input_tokens + output_tokens) as total_tokens,
    SUM(message_count) as total_messages
FROM claude_history_metrics
GROUP BY date
ORDER BY date DESC
LIMIT 30;
```

### Model Breakdown for a Specific Date
```sql
SELECT
    model,
    cost,
    input_tokens,
    output_tokens,
    message_count
FROM claude_history_metrics
WHERE date = '2025-11-17'
ORDER BY cost DESC;
```

### Top 10 Most Expensive Days
```sql
SELECT
    date,
    SUM(cost) as daily_cost,
    SUM(message_count) as messages
FROM claude_history_metrics
GROUP BY date
ORDER BY daily_cost DESC
LIMIT 10;
```

### Model Usage Distribution
```sql
SELECT
    model,
    COUNT(DISTINCT date) as days_used,
    SUM(cost) as total_cost,
    SUM(message_count) as total_messages,
    ROUND(AVG(cost), 4) as avg_daily_cost
FROM claude_history_metrics
GROUP BY model
ORDER BY total_cost DESC;
```

## Expected Results

### Migration Success
```bash
$ sqlite3 ~/.claude/cco_metrics.db "SELECT * FROM claude_history_migration_status;"
1|1|2025-11-18 00:45:00|2025-11-18 00:45:05|530|530|15420|
```

### Sample Data Row
```bash
$ sqlite3 ~/.claude/cco_metrics.db "SELECT * FROM claude_history_metrics WHERE date='2025-11-17';"
2025-11-17|claude-opus-4|15000|7500|2000|1000|2.45|5|25
2025-11-17|claude-sonnet-4-5|10000|5000|0|0|0.75|3|18
2025-11-17|claude-haiku-4-5|5000|2500|0|0|0.125|2|12
```

### API Response
```json
{
  "total_cost": 125.45,
  "days_of_data": 30,
  "first_day": {
    "date": "2025-10-19",
    "cost": 3.22
  },
  "last_day": {
    "date": "2025-11-17",
    "cost": 5.67
  }
}
```

## Troubleshooting

### No Data in Database
```bash
# Check migration status
sqlite3 ~/.claude/cco_metrics.db "SELECT migrated FROM claude_history_migration_status WHERE id=1;"

# If migrated=0, check error_message
sqlite3 ~/.claude/cco_metrics.db "SELECT error_message FROM claude_history_migration_status WHERE id=1;"

# Re-run migration manually
sqlite3 ~/.claude/cco_metrics.db "UPDATE claude_history_migration_status SET migrated=0 WHERE id=1;"
# Restart server
```

### Flat Line in Dashboard
```bash
# Check if data has variation
sqlite3 ~/.claude/cco_metrics.db "SELECT date, SUM(cost) FROM claude_history_metrics GROUP BY date ORDER BY date DESC LIMIT 10;"

# If all costs are same, check JSONL files
ls -la ~/.claude/projects/-Users-brent-git-cc-orchestra/ | wc -l
```

### API Returns Mock Data
```bash
# Check server logs for database errors
tail -50 /tmp/cco_test2.log | grep -E "(ERROR|Failed|database)"

# Verify persistence layer initialized
curl -s http://localhost:8302/health | jq '.version'
```

## Performance Metrics

### Migration Performance
- **530 JSONL files**: ~5-10 seconds
- **Database size**: ~500KB - 2MB
- **Query performance**: <50ms for 30-day range

### API Response Times
- `/api/stats` with database: ~100-200ms
- `/api/stats` without database (mock): ~50-100ms
- Chart data with 30 days: ~150-250ms

## Verification Checklist

- [ ] Database file exists at `~/.claude/cco_metrics.db`
- [ ] Migration status shows `migrated=1`
- [ ] At least 30 unique dates in database
- [ ] Multiple models represented in data
- [ ] Cost values vary by date (not all same)
- [ ] API returns non-flat cost_over_time array
- [ ] Dashboard chart shows trend line
- [ ] Model distribution matches known usage
- [ ] Dates are sequential without gaps
- [ ] Total cost matches sum of daily costs
