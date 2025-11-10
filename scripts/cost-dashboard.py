#!/usr/bin/env python3
"""
ccproxy Cost Tracking Dashboard
Parses LiteLLM logs and displays cost/usage statistics via web interface
"""

from flask import Flask, render_template_string, jsonify
import re
import json
from datetime import datetime
from collections import defaultdict
import os

app = Flask(__name__)

# Pricing (per 1M tokens)
PRICING = {
    'claude-opus-4': {'input': 15.00, 'output': 75.00},
    'claude-sonnet-4-5': {'input': 3.00, 'output': 15.00},
    'claude-3-5-sonnet': {'input': 0.00, 'output': 0.00},  # Local Ollama - Free
    'claude-3-haiku': {'input': 0.00, 'output': 0.00},      # Local Ollama - Free
    'gpt-4': {'input': 0.00, 'output': 0.00}                 # Local Ollama - Free
}

HTML_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <title>ccproxy Cost Dashboard</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        :root {
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --text-primary: #e2e8f0;
            --text-secondary: #94a3b8;
            --border-color: #334155;
            --accent-blue: #116df8;
            --accent-orange: #ff5100;
            --success-green: #22c55e;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            margin: 0;
            padding: 20px;
            line-height: 1.6;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
        }

        h1 {
            color: var(--accent-blue);
            margin-bottom: 10px;
        }

        .subtitle {
            color: var(--text-secondary);
            margin-bottom: 30px;
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }

        .card {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 20px;
        }

        .card h2 {
            color: var(--accent-blue);
            font-size: 1.2em;
            margin-top: 0;
            margin-bottom: 15px;
        }

        .metric {
            display: flex;
            justify-content: space-between;
            margin-bottom: 10px;
            padding-bottom: 10px;
            border-bottom: 1px solid var(--border-color);
        }

        .metric:last-child {
            border-bottom: none;
            margin-bottom: 0;
        }

        .metric-label {
            color: var(--text-secondary);
        }

        .metric-value {
            color: var(--text-primary);
            font-weight: 600;
        }

        .cost-paid {
            color: var(--accent-orange);
        }

        .cost-saved {
            color: var(--success-green);
        }

        .savings-highlight {
            background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
            border: 2px solid var(--success-green);
            padding: 25px;
            border-radius: 12px;
            text-align: center;
            margin-bottom: 30px;
        }

        .savings-highlight h2 {
            color: var(--success-green);
            font-size: 1.8em;
            margin: 0 0 10px 0;
        }

        .savings-highlight .amount {
            font-size: 3em;
            font-weight: bold;
            color: var(--success-green);
        }

        .savings-highlight .subtitle {
            color: var(--text-secondary);
            font-size: 1.1em;
        }

        .model-breakdown {
            margin-top: 15px;
        }

        .model-row {
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid var(--border-color);
        }

        .model-name {
            font-weight: 500;
        }

        .badge {
            display: inline-block;
            padding: 3px 8px;
            border-radius: 4px;
            font-size: 0.85em;
            font-weight: 600;
            margin-left: 8px;
        }

        .badge-paid {
            background: #7c2d12;
            color: var(--accent-orange);
        }

        .badge-free {
            background: #14532d;
            color: var(--success-green);
        }

        .refresh-info {
            text-align: center;
            color: var(--text-secondary);
            margin-top: 20px;
            font-size: 0.9em;
        }

        @media (max-width: 768px) {
            .grid {
                grid-template-columns: 1fr;
            }

            .savings-highlight .amount {
                font-size: 2em;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>💰 ccproxy Cost Dashboard</h1>
        <p class="subtitle">Hybrid routing savings tracker</p>

        <div class="savings-highlight">
            <h2>💸 Total Savings</h2>
            <div class="amount">${{ "%.2f"|format(stats.total_saved) }}</div>
            <p class="subtitle">{{ stats.savings_percent }}% cost reduction vs all-Anthropic</p>
        </div>

        <div class="grid">
            <div class="card">
                <h2>📊 Request Distribution</h2>
                <div class="metric">
                    <span class="metric-label">Anthropic API</span>
                    <span class="metric-value cost-paid">{{ stats.anthropic_requests }} requests</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Local Ollama</span>
                    <span class="metric-value cost-saved">{{ stats.local_requests }} requests</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Total Requests</span>
                    <span class="metric-value">{{ stats.total_requests }}</span>
                </div>
            </div>

            <div class="card">
                <h2>💵 Cost Breakdown</h2>
                <div class="metric">
                    <span class="metric-label">Actual Cost</span>
                    <span class="metric-value cost-paid">${{ "%.2f"|format(stats.actual_cost) }}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">If All Anthropic</span>
                    <span class="metric-value">${{ "%.2f"|format(stats.hypothetical_cost) }}</span>
                </div>
                <div class="metric">
                    <span class="metric-label">Savings</span>
                    <span class="metric-value cost-saved">${{ "%.2f"|format(stats.total_saved) }}</span>
                </div>
            </div>
        </div>

        <div class="card">
            <h2>🤖 Model Usage</h2>
            <div class="model-breakdown">
                {% for model, data in stats.models.items() %}
                <div class="model-row">
                    <div>
                        <span class="model-name">{{ model }}</span>
                        {% if data.is_paid %}
                        <span class="badge badge-paid">PAID</span>
                        {% else %}
                        <span class="badge badge-free">FREE</span>
                        {% endif %}
                    </div>
                    <div>
                        <span class="metric-value">{{ data.count }} requests</span>
                        {% if data.is_paid %}
                        <span class="cost-paid"> (${{ "%.2f"|format(data.cost) }})</span>
                        {% endif %}
                    </div>
                </div>
                {% endfor %}
            </div>
        </div>

        <div class="refresh-info">
            Last updated: {{ stats.last_updated }}<br>
            Auto-refresh every 30 seconds
        </div>
    </div>

    <script>
        // Auto-refresh every 30 seconds
        setTimeout(() => location.reload(), 30000);
    </script>
</body>
</html>
"""

def parse_logs(log_file):
    """Parse LiteLLM logs to extract usage statistics"""
    stats = {
        'models': defaultdict(lambda: {'count': 0, 'cost': 0.0, 'is_paid': False}),
        'total_requests': 0,
        'anthropic_requests': 0,
        'local_requests': 0,
        'actual_cost': 0.0,
        'hypothetical_cost': 0.0,
        'total_saved': 0.0,
        'savings_percent': 0.0,
        'last_updated': datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    }

    if not os.path.exists(log_file):
        return stats

    try:
        with open(log_file, 'r') as f:
            for line in f:
                # Look for model usage in logs
                # Pattern: "model": "claude-opus-4" or similar
                model_match = re.search(r'"model":\s*"([^"]+)"', line)
                if model_match:
                    model = model_match.group(1)

                    # Count request
                    if model in PRICING:
                        stats['models'][model]['count'] += 1
                        stats['total_requests'] += 1

                        # Track if paid or free
                        is_paid = PRICING[model]['input'] > 0 or PRICING[model]['output'] > 0
                        stats['models'][model]['is_paid'] = is_paid

                        if is_paid:
                            stats['anthropic_requests'] += 1
                            # Estimate cost (avg ~$0.02 per Opus request, ~$0.004 per Sonnet)
                            avg_cost = 0.02 if 'opus' in model else 0.004
                            stats['models'][model]['cost'] += avg_cost
                            stats['actual_cost'] += avg_cost
                        else:
                            stats['local_requests'] += 1

        # Calculate hypothetical cost if all requests used Anthropic API
        # Average of ~$0.01 per request
        stats['hypothetical_cost'] = stats['total_requests'] * 0.01

        # Calculate savings
        stats['total_saved'] = stats['hypothetical_cost'] - stats['actual_cost']

        # Calculate savings percentage
        if stats['hypothetical_cost'] > 0:
            stats['savings_percent'] = round(100 * (1 - stats['actual_cost'] / stats['hypothetical_cost']), 1)

    except Exception as e:
        print(f"Error parsing logs: {e}")

    return stats

@app.route('/')
def dashboard():
    """Main dashboard page"""
    log_file = '/Users/brent/ccproxy/logs/litellm.log'
    stats = parse_logs(log_file)
    return render_template_string(HTML_TEMPLATE, stats=stats)

@app.route('/api/stats')
def api_stats():
    """API endpoint for stats (JSON)"""
    log_file = '/Users/brent/ccproxy/logs/litellm.log'
    stats = parse_logs(log_file)
    return jsonify(stats)

if __name__ == '__main__':
    # Run on port 8082 (ccproxy is on 8081)
    app.run(host='127.0.0.1', port=8082, debug=False)
