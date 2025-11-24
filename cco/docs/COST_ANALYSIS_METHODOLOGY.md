# Cost Analysis Methodology

## Overview

This document provides a comprehensive methodology for calculating costs across all supported models and caching mechanisms in the CCO Proxy system.

## Cost Components

### 1. Base Token Costs

Token costs are calculated per million tokens for input and output:

```
token_cost = (token_count / 1,000,000) × price_per_million
```

### 2. Cache Multipliers

Different cache operations have different cost multipliers:

| Operation | Multiplier | Description |
|-----------|------------|-------------|
| Regular Input | 1.0x | Standard pricing |
| Cache Write | 1.25x | Creating cache entry (Anthropic) |
| Cache Read (Opus/Sonnet) | 0.1x | 90% discount |
| Cache Read (Haiku) | 0.2x | 80% discount |
| Moka Cache | 0.0x | Free (local) |

## Model Pricing Matrix

### Anthropic Models

| Model | Input ($/1M) | Output ($/1M) | Cache Write | Cache Read |
|-------|-------------|---------------|-------------|------------|
| claude-opus-4 | $15.00 | $75.00 | $18.75 | $1.50 |
| claude-sonnet-3.5 | $3.00 | $15.00 | $3.75 | $0.30 |
| claude-haiku | $0.25 | $1.25 | $0.30 | $0.05 |

### OpenAI Models

| Model | Input ($/1M) | Output ($/1M) | Notes |
|-------|-------------|---------------|--------|
| gpt-4 | $30.00 | $60.00 | No cache pricing |
| gpt-4-turbo | $10.00 | $30.00 | No cache pricing |
| gpt-3.5-turbo | $0.50 | $1.50 | No cache pricing |
| o1-preview | $15.00 | $60.00 | Reasoning model |

### Self-Hosted Models

| Model | Input | Output | Comparison Model | Est. Savings |
|-------|-------|--------|------------------|--------------|
| ollama/llama3-70b | $0.00 | $0.00 | claude-sonnet-3.5 | ~$3.00/1M tokens |
| ollama/llama3-8b | $0.00 | $0.00 | claude-haiku | ~$0.25/1M tokens |
| ollama/mixtral-8x7b | $0.00 | $0.00 | gpt-3.5-turbo | ~$0.50/1M tokens |
| vllm/llama2-70b | $0.00 | $0.00 | claude-sonnet-3.5 | ~$3.00/1M tokens |

### Cloud-Hosted Alternatives

| Provider | Model | Input ($/1M) | Output ($/1M) |
|----------|-------|-------------|---------------|
| Groq | llama3-70b | $0.59 | $0.79 |
| Together | llama3-70b | $0.90 | $0.90 |
| Anyscale | llama2-70b | $1.00 | $1.00 |
| Replicate | llama3-70b | $0.65 | $2.75 |

## Calculation Formulas

### Standard API Call

```python
def calculate_standard_cost(model, input_tokens, output_tokens):
    pricing = get_model_pricing(model)

    input_cost = (input_tokens / 1_000_000) * pricing.input
    output_cost = (output_tokens / 1_000_000) * pricing.output

    return input_cost + output_cost
```

### Moka Cache Hit

```python
def calculate_moka_savings(model, input_tokens, output_tokens):
    would_be_cost = calculate_standard_cost(model, input_tokens, output_tokens)
    actual_cost = 0.00  # Served from local cache
    savings = would_be_cost

    return {
        'would_be_cost': would_be_cost,
        'actual_cost': actual_cost,
        'savings': savings,
        'savings_percent': 100.0
    }
```

### Claude Cache Read

```python
def calculate_claude_cache_savings(model, cached_tokens, new_tokens, output_tokens):
    pricing = get_model_pricing(model)

    # Actual cost with cache
    cache_discount = 0.1 if model in ['opus', 'sonnet'] else 0.2
    cache_read_cost = (cached_tokens / 1_000_000) * pricing.input * cache_discount
    new_input_cost = (new_tokens / 1_000_000) * pricing.input
    output_cost = (output_tokens / 1_000_000) * pricing.output
    actual_cost = cache_read_cost + new_input_cost + output_cost

    # Would-be cost without cache
    total_input = cached_tokens + new_tokens
    would_be_input = (total_input / 1_000_000) * pricing.input
    would_be_cost = would_be_input + output_cost

    savings = would_be_cost - actual_cost
    savings_percent = (savings / would_be_cost) * 100

    return {
        'would_be_cost': would_be_cost,
        'actual_cost': actual_cost,
        'savings': savings,
        'savings_percent': savings_percent
    }
```

### Self-Hosted Model Savings

```python
def calculate_self_hosted_savings(model, input_tokens, output_tokens):
    # Get comparison model pricing
    config = get_model_config(model)
    comparison_model = config.savings_comparison

    if comparison_model:
        # Calculate what it would cost with comparison model
        would_be_cost = calculate_standard_cost(
            comparison_model,
            input_tokens,
            output_tokens
        )
        actual_cost = 0.00  # Self-hosted
        savings = would_be_cost

        return {
            'would_be_cost': would_be_cost,
            'actual_cost': actual_cost,
            'savings': savings,
            'comparison_model': comparison_model,
            'savings_percent': 100.0
        }
    else:
        # No comparison available
        return {
            'would_be_cost': 0.00,
            'actual_cost': 0.00,
            'savings': 0.00,
            'comparison_model': None,
            'savings_percent': 0.0
        }
```

## Complex Scenarios

### Scenario 1: Multi-Turn Conversation with Cache

```python
# First turn - no cache
turn1_cost = calculate_standard_cost('claude-sonnet-3.5', 2000, 500)
# Cost: $0.0135

# Second turn - partial cache
turn2_cost = calculate_claude_cache_savings(
    model='claude-sonnet-3.5',
    cached_tokens=1800,  # Previous context
    new_tokens=200,      # New user input
    output_tokens=400
)
# Actual: $0.00654, Would-be: $0.012, Savings: $0.00546

# Total conversation
total_actual = turn1_cost + turn2_cost['actual_cost']  # $0.02004
total_would_be = turn1_cost + turn2_cost['would_be_cost']  # $0.0255
total_savings = turn2_cost['savings']  # $0.00546
```

### Scenario 2: Fallback Chain

```python
def calculate_fallback_chain_cost(primary_model, fallback_models, tokens):
    costs = []

    # Primary attempt (failed, but still costs)
    primary_cost = calculate_standard_cost(
        primary_model,
        tokens['input'],
        0  # Failed, no output
    )
    costs.append(primary_cost)

    # Fallback attempts
    for model in fallback_models:
        if model == successful_model:
            # This one succeeded
            fallback_cost = calculate_standard_cost(
                model,
                tokens['input'],
                tokens['output']
            )
            costs.append(fallback_cost)
            break
        else:
            # Failed attempt
            failed_cost = calculate_standard_cost(
                model,
                tokens['input'],
                0
            )
            costs.append(failed_cost)

    return sum(costs)
```

### Scenario 3: Batch Processing with Cache

```python
def calculate_batch_savings(requests):
    total_actual = 0
    total_would_be = 0
    cache_hits = 0

    for request in requests:
        cache_key = generate_cache_key(request)

        if cache.has(cache_key):
            # Cache hit
            cache_hits += 1
            savings = calculate_moka_savings(
                request.model,
                request.input_tokens,
                request.output_tokens
            )
            total_would_be += savings['would_be_cost']
            # actual_cost is 0 for cache hit
        else:
            # Cache miss
            cost = calculate_standard_cost(
                request.model,
                request.input_tokens,
                request.output_tokens
            )
            total_actual += cost
            total_would_be += cost

    return {
        'total_actual': total_actual,
        'total_would_be': total_would_be,
        'total_savings': total_would_be - total_actual,
        'cache_hit_rate': (cache_hits / len(requests)) * 100
    }
```

## Cost Optimization Strategies

### 1. Model Selection Optimization

```python
def select_optimal_model(task_complexity, budget):
    """Select the most cost-effective model for a task."""

    models = [
        {'name': 'claude-haiku', 'cost_factor': 1, 'capability': 3},
        {'name': 'claude-sonnet-3.5', 'cost_factor': 12, 'capability': 7},
        {'name': 'claude-opus-4', 'cost_factor': 60, 'capability': 10},
        {'name': 'ollama/llama3-70b', 'cost_factor': 0, 'capability': 6},
    ]

    # Filter by capability
    capable_models = [m for m in models if m['capability'] >= task_complexity]

    # Sort by cost
    capable_models.sort(key=lambda x: x['cost_factor'])

    # Consider budget
    for model in capable_models:
        if model['cost_factor'] <= budget or model['cost_factor'] == 0:
            return model['name']

    return capable_models[0]['name']  # Best available
```

### 2. Cache Efficiency Maximization

```python
def optimize_cache_strategy(prompt_patterns):
    """Determine optimal cache configuration."""

    analysis = {
        'high_frequency': [],  # Cache for 2 hours
        'medium_frequency': [],  # Cache for 1 hour
        'low_frequency': [],  # Cache for 30 minutes
        'no_cache': []  # Don't cache
    }

    for pattern in prompt_patterns:
        frequency = pattern['daily_requests']
        cost_per_request = pattern['avg_cost']

        cache_benefit = frequency * cost_per_request

        if cache_benefit > 10.0:  # >$10/day savings
            analysis['high_frequency'].append(pattern)
        elif cache_benefit > 1.0:  # >$1/day savings
            analysis['medium_frequency'].append(pattern)
        elif cache_benefit > 0.1:  # >$0.10/day savings
            analysis['low_frequency'].append(pattern)
        else:
            analysis['no_cache'].append(pattern)

    return analysis
```

### 3. Batch Processing Optimization

```python
def optimize_batch_size(model, requests):
    """Find optimal batch size for cost/performance."""

    # Model-specific limits
    limits = {
        'claude-opus-4': {'max_batch': 10, 'max_tokens': 100_000},
        'claude-sonnet-3.5': {'max_batch': 20, 'max_tokens': 150_000},
        'claude-haiku': {'max_batch': 50, 'max_tokens': 200_000},
    }

    model_limit = limits.get(model)

    # Calculate optimal batch
    total_tokens = sum(r['tokens'] for r in requests)
    batch_count = min(
        len(requests),
        model_limit['max_batch'],
        model_limit['max_tokens'] // avg_tokens_per_request
    )

    return batch_count
```

## Reporting Metrics

### Daily Cost Report

```sql
SELECT
    DATE(timestamp) as date,
    SUM(actual_cost) as total_cost,
    SUM(would_be_cost) as would_be_cost,
    SUM(savings) as total_savings,
    COUNT(*) as total_requests,
    AVG(cache_hit) * 100 as cache_hit_rate,
    SUM(input_tokens + output_tokens) as total_tokens
FROM api_calls
WHERE timestamp > datetime('now', '-30 days')
GROUP BY DATE(timestamp)
ORDER BY date DESC;
```

### Model Cost Distribution

```sql
SELECT
    model,
    COUNT(*) as requests,
    SUM(actual_cost) as total_cost,
    AVG(actual_cost) as avg_cost_per_request,
    SUM(savings) as total_savings,
    SUM(input_tokens) as total_input_tokens,
    SUM(output_tokens) as total_output_tokens
FROM api_calls
WHERE timestamp > datetime('now', '-7 days')
GROUP BY model
ORDER BY total_cost DESC;
```

### Cost per Project

```sql
SELECT
    p.name as project,
    COUNT(ac.id) as requests,
    SUM(ac.actual_cost) as project_cost,
    SUM(ac.savings) as project_savings,
    AVG(ac.cache_hit) * 100 as cache_hit_rate,
    SUM(ac.actual_cost + ac.savings) as would_be_cost
FROM api_calls ac
JOIN conversations c ON ac.conversation_id = c.id
JOIN projects p ON c.project_id = p.id
WHERE ac.timestamp > datetime('now', '-30 days')
GROUP BY p.id
ORDER BY project_cost DESC;
```

## Alert Thresholds

### Cost Alerts

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| Hourly spend | >$10 | >$25 | Review usage patterns |
| Daily spend | >$100 | >$250 | Implement rate limiting |
| Cost per request | >$0.50 | >$1.00 | Check for inefficient prompts |
| Cache hit rate | <50% | <30% | Optimize cache strategy |

### Anomaly Detection

```python
def detect_cost_anomalies(current_hour_cost, historical_costs):
    """Detect unusual cost patterns."""

    mean = statistics.mean(historical_costs)
    stdev = statistics.stdev(historical_costs)

    z_score = (current_hour_cost - mean) / stdev

    if z_score > 3:
        return 'CRITICAL: Cost spike detected'
    elif z_score > 2:
        return 'WARNING: Elevated costs'
    else:
        return 'Normal'
```

## Best Practices

1. **Always track both actual and would-be costs** for accurate savings calculation
2. **Use integer cents** for internal storage to avoid floating-point errors
3. **Include all cost components** (input, output, cache operations)
4. **Consider time-of-day patterns** when setting cache TTLs
5. **Monitor cost trends** to detect issues early
6. **Set project-level budgets** to prevent overruns
7. **Use model fallback chains** strategically based on cost/capability trade-offs
8. **Implement cost approval** for expensive operations
9. **Regular cost audits** to identify optimization opportunities
10. **Document all pricing changes** for historical analysis