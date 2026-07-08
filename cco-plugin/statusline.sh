#!/bin/bash
# CCO statusline — shipped with the cco plugin so it is only active when
# claude is launched through the cco wrapper. Plain `claude` invocations
# fall back to whatever the user has configured (or none).
input=$(cat)

# Billing cross-check log: append one JSONL line per render so the cco TUI
# can compare Claude Code's authoritative session cost against our locally
# aggregated number. Readers dedupe to the latest entry per session_id.
BILLING_LOG="$HOME/.cco/statusline-billing.jsonl"
if mkdir -p "$HOME/.cco" 2>/dev/null; then
  echo "$input" | jq -c --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '{
    ts: $ts,
    session_id: .session_id,
    transcript_path: .transcript_path,
    model_id: .model.id,
    model_display: .model.display_name,
    total_cost_usd: (.cost.total_cost_usd // 0),
    total_duration_ms: (.cost.total_duration_ms // 0),
    total_api_duration_ms: (.cost.total_api_duration_ms // 0),
    lines_added: (.cost.total_lines_added // 0),
    lines_removed: (.cost.total_lines_removed // 0)
  }' >> "$BILLING_LOG" 2>/dev/null
fi

MODEL=$(echo "$input" | jq -r '.model.display_name // "claude"')
DIR=$(echo "$input" | jq -r '.workspace.current_dir // ""')
COST=$(echo "$input" | jq -r '.cost.total_cost_usd // 0')
PCT=$(echo "$input" | jq -r '.context_window.used_percentage // 0' | cut -d. -f1)
DURATION_MS=$(echo "$input" | jq -r '.cost.total_duration_ms // 0')
EFFORT=$(echo "$input" | jq -r '.effort.level // empty')
FIVE_H=$(echo "$input" | jq -r '.rate_limits.five_hour.used_percentage // empty')
FIVE_H_RESET=$(echo "$input" | jq -r '.rate_limits.five_hour.resets_at // empty')
WEEK=$(echo "$input" | jq -r '.rate_limits.seven_day.used_percentage // empty')

CYAN='\033[36m'; GREEN='\033[32m'; YELLOW='\033[33m'; RED='\033[31m'
MAGENTA='\033[35m'; DIM='\033[2m'; RESET='\033[0m'

# Color the context bar by usage
if [ "$PCT" -ge 90 ]; then BAR_COLOR="$RED"
elif [ "$PCT" -ge 70 ]; then BAR_COLOR="$YELLOW"
else BAR_COLOR="$GREEN"; fi

FILLED=$((PCT / 10)); EMPTY=$((10 - FILLED))
printf -v FILL "%${FILLED}s"; printf -v PAD "%${EMPTY}s"
BAR="${FILL// /█}${PAD// /░}"

MINS=$((DURATION_MS / 60000)); SECS=$(((DURATION_MS % 60000) / 1000))
COST_FMT=$(printf '$%.2f' "$COST")

# Color rate limits
rl_color() {
  local v=$1
  v=${v%.*}
  if [ "$v" -ge 90 ]; then printf "%s" "$RED"
  elif [ "$v" -ge 70 ]; then printf "%s" "$YELLOW"
  else printf "%s" "$GREEN"; fi
}

mini_bar() {
  local pct=${1%.*}
  local f=$((pct / 20)); local e=$((5 - f))
  printf -v fl "%${f}s"; printf -v pd "%${e}s"
  printf "%s" "${fl// /█}${pd// /░}"
}

LIMITS=""
if [ -n "$FIVE_H" ]; then
  C=$(rl_color "$FIVE_H")
  LIMITS="${C}5h $(mini_bar "$FIVE_H") $(printf '%.0f' "$FIVE_H")%${RESET}"
  # Add window-time progress: how far into the 5h window we are
  if [ -n "$FIVE_H_RESET" ]; then
    # Validate FIVE_H_RESET to prevent command injection
    if ! { [[ "$FIVE_H_RESET" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}([.][0-9]+)?(Z|[+-][0-9:]+)?$ ]] || [[ "$FIVE_H_RESET" =~ ^[0-9]+$ ]]; }; then
      FIVE_H_RESET=""
    fi

    NOW_S=$(date -u +%s)
    # resets_at can be epoch seconds (integer) or ISO 8601 string
    if [[ "$FIVE_H_RESET" =~ ^[0-9]+$ ]]; then
      RESET_S="$FIVE_H_RESET"
    else
      RESET_CLEAN="${FIVE_H_RESET%Z}"
      RESET_CLEAN="${RESET_CLEAN%%.*}"
      RESET_S=$(date -u -j -f "%Y-%m-%dT%H:%M:%S" "$RESET_CLEAN" +%s 2>/dev/null \
                || date -u -d "$FIVE_H_RESET" +%s 2>/dev/null)
    fi
    if [ -n "$RESET_S" ] && [ "$RESET_S" -gt "$NOW_S" ]; then
      MINS_LEFT=$(( (RESET_S - NOW_S) / 60 ))
      MINS_ELAPSED=$(( 300 - MINS_LEFT ))
      [ "$MINS_ELAPSED" -lt 0 ] && MINS_ELAPSED=0
      [ "$MINS_ELAPSED" -gt 300 ] && MINS_ELAPSED=300
      TIME_PCT=$(( MINS_ELAPSED * 100 / 300 ))
      TC=$(rl_color "$TIME_PCT")
      RESET_LOCAL=$(date -r "$RESET_S" "+%-I:%M%p %Z" 2>/dev/null | tr '[:upper:]' '[:lower:]')
      LIMITS="$LIMITS ${DIM}·${RESET} ${TC}⏳ $(mini_bar "$TIME_PCT") ${TIME_PCT}%${RESET}${RESET_LOCAL:+ ${DIM}(resets $RESET_LOCAL)${RESET}}"
    fi
  fi
fi
if [ -n "$WEEK" ]; then
  C=$(rl_color "$WEEK")
  LIMITS="${LIMITS:+$LIMITS ${DIM}·${RESET} }${C}7d $(mini_bar "$WEEK") $(printf '%.0f' "$WEEK")%${RESET}"
fi

BRANCH=""
if git rev-parse --git-dir > /dev/null 2>&1; then
  B=$(git branch --show-current 2>/dev/null)
  [ -n "$B" ] && BRANCH=" ${DIM}·${RESET} 🌿 $B"
fi

EFFORT_FMT=""
[ -n "$EFFORT" ] && EFFORT_FMT=" ${MAGENTA}[$EFFORT]${RESET}"

USER=""
if [ -f "$HOME/.claude.json" ]; then
  U=$(jq -r '.oauthAccount.displayName // .oauthAccount.emailAddress // empty' "$HOME/.claude.json" 2>/dev/null)
  [ -n "$U" ] && USER=" ${DIM}·${RESET} 👤 $U"
fi

echo -e "${CYAN}[$MODEL]${RESET}${EFFORT_FMT}${USER} 📁 ${DIR##*/}$BRANCH"
echo -e "${BAR_COLOR}${BAR}${RESET} ${PCT}% ${DIM}·${RESET} ${YELLOW}${COST_FMT}${RESET} ${DIM}·${RESET} ⏱️ ${MINS}m ${SECS}s${LIMITS:+ ${DIM}·${RESET} $LIMITS}"
