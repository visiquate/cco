#!/bin/bash
# knowledge-query.sh - Query CCO knowledge store and return formatted results
# Usage: knowledge-query.sh --query "search text" [--limit N]

set -euo pipefail

SOCKET_PATH="${HOME}/.cco/daemon.sock"
QUERY=""
LIMIT=5

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --query) QUERY="$2"; shift 2 ;;
        --limit) LIMIT="$2"; shift 2 ;;
        *) shift ;;
    esac
done

# Validate LIMIT is a non-negative integer; reset to default if not
[[ "$LIMIT" =~ ^[0-9]+$ ]] || LIMIT=5

# Exit silently if no query or daemon unavailable
if [ -z "$QUERY" ] || [ ! -S "$SOCKET_PATH" ]; then
    exit 0
fi

# Escape query for JSON (handle quotes and special chars)
ESCAPED_QUERY=$(printf '%s' "$QUERY" | jq -Rs '.')

# Build payload
PAYLOAD="{\"query\": ${ESCAPED_QUERY}, \"limit\": ${LIMIT}}"

# Query the daemon
RESULT=$(curl -sf --max-time 3 --unix-socket "$SOCKET_PATH" \
    -X POST http://localhost/api/knowledge/search \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" 2>/dev/null) || exit 0

# Format results - skip if empty or invalid
echo "$RESULT" | jq -r '.[] | "[\(.type)] \(.text) (\(.agent), \(.score))"' 2>/dev/null || true
