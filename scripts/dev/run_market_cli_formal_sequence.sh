#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="$ROOT_DIR/logs"
mkdir -p "$LOG_DIR"
STAMP="$(date +%Y%m%d_%H%M%S)"
SUMMARY_LOG="${SUMMARY_LOG:-$LOG_DIR/market_cli_formal_sequence_$STAMP.log}"
QUANTIX_BIN="$ROOT_DIR/target/debug/quantix"
LOCAL_ENV_PATH="$ROOT_DIR/.env.market.local"
INIT_LOCAL_ENV_SCRIPT="$ROOT_DIR/scripts/dev/init_market_cli_local_env.sh"

exec > >(tee -a "$SUMMARY_LOG") 2>&1

echo "[INFO] Market CLI formal sequence log: $SUMMARY_LOG"

"$INIT_LOCAL_ENV_SCRIPT"

if [[ -f "$LOCAL_ENV_PATH" ]]; then
  echo "[INFO] Loading local market env overrides from $LOCAL_ENV_PATH"
  set -a
  # shellcheck disable=SC1090
  source "$LOCAL_ENV_PATH"
  set +a
fi

compress_ws() {
  tr -s '[:space:]' ' ' | sed 's/^ //; s/ $//'
}

extract_first_error() {
  local file="$1"
  grep -E '^Error:' "$file" | head -n 1 | compress_ws
}

extract_first_row_after_heading() {
  local heading="$1"
  local file="$2"
  awk -v heading="$heading" '
    $0 == heading { capture=1; next }
    capture && NF == 0 { next }
    capture && $0 ~ /^📭/ { print; exit }
    capture && $0 ~ /^排名/ { next }
    capture && $0 ~ /^-+$/ { next }
    capture && $0 ~ /板块:$|总市值:$|净利润:$/ { exit }
    capture { print; exit }
  ' "$file" | compress_ws
}

summarize_step() {
  local key="$1"
  local code="$2"
  local step_log="$3"
  local summary=""

  if [[ "$code" -ne 0 ]]; then
    summary="$(extract_first_error "$step_log")"
    [[ -z "$summary" ]] && summary="exit=$code see log"
    echo "[SUMMARY] ${key}_summary=$summary"
    return
  fi

  case "$key" in
    sync_industry)
      summary="exit=0 completed; see log for refreshed industry reference details"
      ;;
    market_foundation)
      local total classified unclassified sectors
      total="$(grep -E '^A股总数:' "$step_log" | head -n 1 | sed 's/^A股总数: //')"
      classified="$(grep -E '^已匹配行业:' "$step_log" | head -n 1 | sed 's/^已匹配行业: //')"
      unclassified="$(grep -E '^未匹配行业:' "$step_log" | head -n 1 | sed 's/^未匹配行业: //')"
      sectors="$(grep -E '^行业数:' "$step_log" | head -n 1 | sed 's/^行业数: //')"
      echo "[FIELD] market_foundation_total_stocks=${total:-N/A}"
      echo "[FIELD] market_foundation_classified_stocks=${classified:-N/A}"
      echo "[FIELD] market_foundation_unclassified_stocks=${unclassified:-N/A}"
      echo "[FIELD] market_foundation_sector_count=${sectors:-N/A}"
      echo "[FIELD] market_foundation_top_sector=$(extract_first_row_after_heading '行业覆盖 Top10:' "$step_log")"
      summary="A股总数=${total:-N/A} 已匹配行业=${classified:-N/A} 未匹配行业=${unclassified:-N/A} 行业数=${sectors:-N/A}"
      ;;
    market_strength)
      local base candidates top_strong top_weak top_cap top_profit
      base="$(grep -E '^基础数据:' "$step_log" | head -n 1 | sed 's/^基础数据: //')"
      candidates="$(grep -E '^强势板块候选股数:' "$step_log" | head -n 1 | sed 's/^强势板块候选股数: //')"
      top_strong="$(extract_first_row_after_heading '强势板块:' "$step_log")"
      top_weak="$(extract_first_row_after_heading '弱势板块:' "$step_log")"
      top_cap="$(extract_first_row_after_heading '强势板块个股 Top10 总市值:' "$step_log")"
      top_profit="$(extract_first_row_after_heading '强势板块个股 Top10 最新净利润:' "$step_log")"
      echo "[FIELD] market_strength_base=${base:-N/A}"
      echo "[FIELD] market_strength_candidate_stock_count=${candidates:-N/A}"
      echo "[FIELD] market_strength_top_strong_sector=${top_strong:-N/A}"
      echo "[FIELD] market_strength_top_weak_sector=${top_weak:-N/A}"
      echo "[FIELD] market_strength_top_market_cap_stock=${top_cap:-N/A}"
      echo "[FIELD] market_strength_top_profit_stock=${top_profit:-N/A}"
      summary="基础数据=${base:-N/A}; 候选股数=${candidates:-N/A}; 强势首行=${top_strong:-N/A}; 弱势首行=${top_weak:-N/A}; 总市值首行=${top_cap:-N/A}; 净利润首行=${top_profit:-N/A}"
      ;;
  esac

  echo "[SUMMARY] ${key}_summary=$summary"
}

run_formal_step() {
  local key="$1"
  local title="$2"
  local cmd="$3"
  local step_log="$LOG_DIR/market_cli_${key}_$STAMP.log"

  echo "\n[STEP] $title"
  echo "[LOG] ${key}_log=$step_log"

  set +e
  bash -lc "$cmd" > >(tee "$step_log") 2>&1
  local code=$?
  set -e

  echo "[RESULT] ${key}_exit=$code"
  summarize_step "$key" "$code" "$step_log"
}

run_formal_step \
  "sync_industry" \
  "Risk sync industry Shenwan" \
  "\"$QUANTIX_BIN\" risk sync industry --standard shenwan"

run_formal_step \
  "market_foundation" \
  "Market foundation" \
  "\"$QUANTIX_BIN\" market foundation"

run_formal_step \
  "market_strength" \
  "Market strength" \
  "\"$QUANTIX_BIN\" market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10"

echo "\nMarket CLI formal sequence completed."
