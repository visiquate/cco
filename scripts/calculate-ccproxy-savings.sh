#!/bin/bash

# Calculate Anthropic API costs vs local Ollama savings from ccproxy logs

echo "📊 ccproxy Cost & Savings Calculator"
echo ""

# Check if we can SSH to Mac mini
if ! ssh -o ConnectTimeout=5 brent@192.168.9.123 "echo ''" 2>/dev/null; then
    echo "❌ Cannot reach Mac mini"
    exit 1
fi

echo "📁 Analyzing ccproxy logs..."
echo ""

# Get log analysis
ssh brent@192.168.9.123 << 'ENDSCRIPT'
cd /Users/brent/ccproxy/logs

# Count requests by model
echo "📈 Request Distribution:"
echo ""

OPUS_COUNT=$(grep -c "claude-opus-4" litellm.log 2>/dev/null || echo "0")
SONNET_COUNT=$(grep -c "claude-sonnet-4-5" litellm.log 2>/dev/null || echo "0")
QWEN_CODER_COUNT=$(grep -c "qwen2.5-coder" litellm.log 2>/dev/null || echo "0")
QWEN_FAST_COUNT=$(grep -c "qwen-fast" litellm.log 2>/dev/null || echo "0")
QWEN_QUALITY_COUNT=$(grep -c "qwen-quality-128k" litellm.log 2>/dev/null || echo "0")

TOTAL_ANTHROPIC=$((OPUS_COUNT + SONNET_COUNT))
TOTAL_LOCAL=$((QWEN_CODER_COUNT + QWEN_FAST_COUNT + QWEN_QUALITY_COUNT))
TOTAL_REQUESTS=$((TOTAL_ANTHROPIC + TOTAL_LOCAL))

echo "  Anthropic API (Architect):"
echo "    - Claude Opus 4:    $OPUS_COUNT requests"
echo "    - Claude Sonnet 4:  $SONNET_COUNT requests"
echo "    - Total:            $TOTAL_ANTHROPIC requests"
echo ""
echo "  Local Ollama (All other agents):"
echo "    - qwen2.5-coder:    $QWEN_CODER_COUNT requests"
echo "    - qwen-fast:        $QWEN_FAST_COUNT requests"
echo "    - qwen-quality-128k: $QWEN_QUALITY_COUNT requests"
echo "    - Total:            $TOTAL_LOCAL requests"
echo ""
echo "  📊 Total requests:   $TOTAL_REQUESTS"
echo ""

# Estimate costs (rough averages)
# Opus 4: ~$15 per 1M input tokens, ~$75 per 1M output tokens (avg ~$0.02 per request)
# Sonnet 4: ~$3 per 1M input tokens, ~$15 per 1M output tokens (avg ~$0.004 per request)
# If ALL were Anthropic API: ~$0.01 per request average

OPUS_COST=$(echo "scale=2; $OPUS_COUNT * 0.02" | bc)
SONNET_COST=$(echo "scale=2; $SONNET_COUNT * 0.004" | bc)
ACTUAL_COST=$(echo "scale=2; $OPUS_COST + $SONNET_COST" | bc)

# If ALL requests used Anthropic API
HYPOTHETICAL_COST=$(echo "scale=2; $TOTAL_REQUESTS * 0.01" | bc)

SAVINGS=$(echo "scale=2; $HYPOTHETICAL_COST - $ACTUAL_COST" | bc)
SAVINGS_PERCENT=$(echo "scale=1; 100 * (1 - $ACTUAL_COST / $HYPOTHETICAL_COST)" | bc 2>/dev/null || echo "0")

echo "💰 Cost Analysis:"
echo ""
echo "  Actual cost (Hybrid):"
echo "    - Opus requests:     \$$OPUS_COST"
echo "    - Sonnet requests:   \$$SONNET_COST"
echo "    - Local requests:    \$0.00"
echo "    - TOTAL:             \$$ACTUAL_COST"
echo ""
echo "  If ALL used Anthropic API:"
echo "    - TOTAL:             \$$HYPOTHETICAL_COST"
echo ""
echo "  💸 Savings:            \$$SAVINGS"
echo "  📉 Cost reduction:     ${SAVINGS_PERCENT}%"
echo ""

ENDSCRIPT

echo "✅ Analysis complete"
echo ""
echo "📝 Note: Cost estimates are based on average request sizes."
echo "   For exact costs, check Anthropic Console: https://console.anthropic.com/settings/usage"
