#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
STAMP="$(date +%Y%m%d_%H%M%S)"
SUMMARY_LOG="${SUMMARY_LOG:-$LOG_DIR/market_cli_formal_sequence_$STAMP.log}"
QUANTIX_BIN="${QUANTIX_BIN:-$ROOT_DIR/target/debug/quantix}"
LOCAL_ENV_PATH="${LOCAL_ENV_PATH:-$ROOT_DIR/.env.market.local}"
ROOT_ENV_PATH="${ROOT_ENV_PATH:-$ROOT_DIR/.env}"
INIT_LOCAL_ENV_SCRIPT="${INIT_LOCAL_ENV_SCRIPT:-$ROOT_DIR/scripts/dev/init_market_cli_local_env.sh}"
MARKET_FUNDAMENTALS_INPUT="${MARKET_FUNDAMENTALS_INPUT:-}"
REHEARSAL_SCRIPT="${REHEARSAL_SCRIPT:-$ROOT_DIR/scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh}"
MARKET_SNAPSHOT_SOURCE="${QUANTIX_MARKET_SNAPSHOT_SOURCE:-auto}"
CLICKHOUSE_URL="${CLICKHOUSE_URL:-http://localhost:8123}"
CLICKHOUSE_DB="${CLICKHOUSE_DB:-quantix}"
CLICKHOUSE_USER="${CLICKHOUSE_USER:-default}"
CLICKHOUSE_PASSWORD="${CLICKHOUSE_PASSWORD:-}"
MARKET_DATE_QUERY_CMD="${MARKET_DATE_QUERY_CMD:-}"
LAST_STEP_EXIT_CODE=0

exec > >(tee -a "$SUMMARY_LOG") 2>&1

echo "[INFO] Market CLI formal sequence log: $SUMMARY_LOG"

load_env_file() {
  local path="$1"
  if [[ -f "$path" ]]; then
    set -a
    # shellcheck disable=SC1090
    source "$path"
    set +a
  fi
}

read_env_value() {
  local key="$1"
  local path="$2"
  [[ -f "$path" ]] || return 0
  local line=""
  line="$(grep -E "^${key}=" "$path" | tail -n 1 || true)"
  [[ -n "$line" ]] || return 0
  printf '%s\n' "${line#*=}"
}

load_tdx_env_fallback_from_root() {
  local path="$1"
  [[ -f "$path" ]] || return 0

  if [[ -z "${QUANTIX_TDX_ROOT:-}" && -z "${TDX_ROOT:-}" ]]; then
    local root_tdx=""
    root_tdx="$(read_env_value "QUANTIX_TDX_ROOT" "$path")"
    [[ -n "$root_tdx" ]] && export QUANTIX_TDX_ROOT="$root_tdx"
  fi

  if [[ -z "${QUANTIX_TDX_MARKET:-}" && -z "${TDX_MARKET:-}" ]]; then
    local root_market=""
    root_market="$(read_env_value "QUANTIX_TDX_MARKET" "$path")"
    [[ -n "$root_market" ]] && export QUANTIX_TDX_MARKET="$root_market"
  fi
}

"$INIT_LOCAL_ENV_SCRIPT"

if [[ -f "$LOCAL_ENV_PATH" ]]; then
  echo "[INFO] Loading local market env overrides from $LOCAL_ENV_PATH"
  load_env_file "$LOCAL_ENV_PATH"
fi
load_tdx_env_fallback_from_root "$ROOT_ENV_PATH"
MARKET_SNAPSHOT_SOURCE="${QUANTIX_MARKET_SNAPSHOT_SOURCE:-${MARKET_SNAPSHOT_SOURCE:-auto}}"
echo "[INFO] Market snapshot source mode: $MARKET_SNAPSHOT_SOURCE"

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

extract_snapshot_source() {
  local file="$1"
  if grep -q 'QUANTIX_MARKET_SNAPSHOT_SOURCE=tdx' "$file"; then
    printf 'tdx_configured\n'
  elif grep -q '开始尝试 TDX fallback' "$file"; then
    printf 'tdx_fallback\n'
  else
    printf 'primary\n'
  fi
}

extract_tdx_coverage() {
  local file="$1"
  local coverage=""
  coverage="$(grep -E 'TDX fallback 仅返回部分 A 股实时行情:' "$file" | tail -n 1 | sed 's/^.*: //')"
  if [[ -n "$coverage" ]]; then
    printf '%s\n' "$coverage"
  else
    printf 'N/A\n'
  fi
}

resolve_strength_stocks_sector_from_log() {
  local file="$1"
  local row=""
  row="$(extract_first_row_after_heading '强势板块:' "$file")"
  [[ -z "$row" ]] && return 0
  [[ "$row" == 📭* ]] && return 0
  awk '{print $3}' <<<"$row"
}

resolve_market_date() {
  if [[ -n "${MARKET_DATE:-}" ]]; then
    printf '%s\n' "$MARKET_DATE"
    return 0
  fi

  local query_cmd="$MARKET_DATE_QUERY_CMD"
  if [[ -z "$query_cmd" ]]; then
    local auth_args=()
    if [[ -n "$CLICKHOUSE_USER" || -n "$CLICKHOUSE_PASSWORD" ]]; then
      auth_args+=(--user "${CLICKHOUSE_USER}:${CLICKHOUSE_PASSWORD}")
    fi
    query_cmd="$(printf "curl -sS %s '%s/?database=%s&query=SELECT%%20max(trade_date)%%20FROM%%20sector_daily%%20WHERE%%20sector_type%%3D%%27industry%%27%%20FORMAT%%20TabSeparatedRaw'" \
      "${auth_args[*]}" "$CLICKHOUSE_URL" "$CLICKHOUSE_DB")"
  fi

  local resolved=""
  set +e
  resolved="$(bash -lc "$query_cmd" 2>/dev/null | tr -d '\r' | tail -n 1 | tr -d '[:space:]')"
  set -e
  if [[ -n "$resolved" && "$resolved" != "\\N" ]]; then
    printf '%s\n' "$resolved"
    return 0
  fi

  printf '%s\n' "2026-03-09"
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
    market_fundamentals_validate)
      local input total unique snapshot_min snapshot_max cap_coverage profit_coverage warning_count
      input="$(grep -E '^  文件:' "$step_log" | head -n 1 | sed 's/^  文件: //')"
      total="$(grep -E '^\[FIELD\] validation_total_records=' "$step_log" | tail -n 1 | sed 's/^\[FIELD\] validation_total_records=//')"
      unique="$(grep -E '^\[FIELD\] validation_unique_codes=' "$step_log" | tail -n 1 | sed 's/^\[FIELD\] validation_unique_codes=//')"
      snapshot_min="$(grep -E '^\[FIELD\] validation_snapshot_min=' "$step_log" | tail -n 1 | sed 's/^\[FIELD\] validation_snapshot_min=//')"
      snapshot_max="$(grep -E '^\[FIELD\] validation_snapshot_max=' "$step_log" | tail -n 1 | sed 's/^\[FIELD\] validation_snapshot_max=//')"
      cap_coverage="$(grep -E '^\[FIELD\] validation_market_cap_coverage=' "$step_log" | tail -n 1 | sed 's/^\[FIELD\] validation_market_cap_coverage=//')"
      profit_coverage="$(grep -E '^\[FIELD\] validation_latest_report_profit_coverage=' "$step_log" | tail -n 1 | sed 's/^\[FIELD\] validation_latest_report_profit_coverage=//')"
      warning_count="$(grep -c '^\[WARN\]' "$step_log" || true)"
      echo "[FIELD] market_fundamentals_validate_input=${input:-N/A}"
      echo "[FIELD] market_fundamentals_validate_total_records=${total:-N/A}"
      echo "[FIELD] market_fundamentals_validate_unique_codes=${unique:-N/A}"
      echo "[FIELD] market_fundamentals_validate_snapshot_min=${snapshot_min:-N/A}"
      echo "[FIELD] market_fundamentals_validate_snapshot_max=${snapshot_max:-N/A}"
      echo "[FIELD] market_fundamentals_validate_market_cap_coverage=${cap_coverage:-N/A}"
      echo "[FIELD] market_fundamentals_validate_latest_report_profit_coverage=${profit_coverage:-N/A}"
      echo "[FIELD] market_fundamentals_validate_warning_count=${warning_count:-N/A}"
      summary="记录数=${total:-N/A} 唯一股票=${unique:-N/A} 快照区间=${snapshot_min:-N/A}~${snapshot_max:-N/A} 总市值覆盖=${cap_coverage:-N/A} 净利润覆盖=${profit_coverage:-N/A} warnings=${warning_count:-N/A}"
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
    market_fundamentals_import)
      local input records written elapsed
      input="$(grep -E '^  文件:' "$step_log" | head -n 1 | sed 's/^  文件: //')"
      records="$(grep -E '^  记录数:' "$step_log" | head -n 1 | sed 's/^  记录数: //')"
      written="$(grep -E '^  已写入:' "$step_log" | head -n 1 | sed 's/^  已写入: //')"
      elapsed="$(grep -E '^  耗时\(秒\):' "$step_log" | head -n 1 | sed 's/^  耗时(秒): //')"
      echo "[FIELD] market_fundamentals_import_input=${input:-N/A}"
      echo "[FIELD] market_fundamentals_import_records=${records:-N/A}"
      echo "[FIELD] market_fundamentals_import_written=${written:-N/A}"
      summary="输入=${input:-N/A} 记录数=${records:-N/A} 已写入=${written:-N/A} 耗时(秒)=${elapsed:-N/A}"
      ;;
    market_strength)
      local base candidates top_strong top_weak top_cap top_profit snapshot_source tdx_coverage
      base="$(grep -E '^基础数据:' "$step_log" | head -n 1 | sed 's/^基础数据: //')"
      candidates="$(grep -E '^强势板块候选股数:' "$step_log" | head -n 1 | sed 's/^强势板块候选股数: //')"
      top_strong="$(extract_first_row_after_heading '强势板块:' "$step_log")"
      top_weak="$(extract_first_row_after_heading '弱势板块:' "$step_log")"
      top_cap="$(extract_first_row_after_heading '强势板块个股 Top10 总市值:' "$step_log")"
      top_profit="$(extract_first_row_after_heading '强势板块个股 Top10 推算净利润:' "$step_log")"
      snapshot_source="$(extract_snapshot_source "$step_log")"
      tdx_coverage="$(extract_tdx_coverage "$step_log")"
      echo "[FIELD] market_strength_base=${base:-N/A}"
      echo "[FIELD] market_strength_candidate_stock_count=${candidates:-N/A}"
      echo "[FIELD] market_strength_snapshot_source=${snapshot_source:-N/A}"
      echo "[FIELD] market_strength_tdx_coverage=${tdx_coverage:-N/A}"
      echo "[FIELD] market_strength_top_strong_sector=${top_strong:-N/A}"
      echo "[FIELD] market_strength_top_weak_sector=${top_weak:-N/A}"
      echo "[FIELD] market_strength_top_market_cap_stock=${top_cap:-N/A}"
      echo "[FIELD] market_strength_top_profit_stock=${top_profit:-N/A}"
      summary="基础数据=${base:-N/A}; 候选股数=${candidates:-N/A}; 快照来源=${snapshot_source:-N/A}; TDX覆盖=${tdx_coverage:-N/A}; 强势首行=${top_strong:-N/A}; 弱势首行=${top_weak:-N/A}; 总市值首行=${top_cap:-N/A}; 净利润首行=${top_profit:-N/A}"
      ;;
    market_strength_stocks)
      local sector metric covered top_row metric_heading
      sector="$(grep -E '^行业过滤:' "$step_log" | head -n 1 | sed 's/^行业过滤: //')"
      covered="$(grep -E '覆盖:' "$step_log" | head -n 1 | sed 's/^[^:]*: //')"
      metric_heading="$(grep -E '^按.*从大到小 Top[0-9]+:' "$step_log" | head -n 1)"
      metric="$(sed 's/^按//; s/从大到小 Top.*$//' <<<"$metric_heading")"
      if [[ -n "$metric_heading" ]]; then
        top_row="$(extract_first_row_after_heading "$metric_heading" "$step_log")"
      else
        top_row=""
      fi
      echo "[FIELD] market_strength_stocks_sector_filter=${sector:-N/A}"
      echo "[FIELD] market_strength_stocks_metric=${metric:-N/A}"
      echo "[FIELD] market_strength_stocks_coverage=${covered:-N/A}"
      echo "[FIELD] market_strength_stocks_top_row=${top_row:-N/A}"
      summary="行业过滤=${sector:-N/A}; 指标=${metric:-N/A}; 覆盖=${covered:-N/A}; 首行=${top_row:-N/A}"
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
  LAST_STEP_EXIT_CODE="$code"
}

run_formal_step \
  "sync_industry" \
  "Risk sync industry Shenwan" \
  "\"$QUANTIX_BIN\" risk sync industry --standard shenwan"

if [[ -n "$MARKET_FUNDAMENTALS_INPUT" ]]; then
  echo "[INFO] Using market fundamentals input: $MARKET_FUNDAMENTALS_INPUT"
  run_formal_step \
    "market_fundamentals_validate" \
    "Validate market fundamentals" \
    "\"$QUANTIX_BIN\" data validate-fundamentals --input \"$MARKET_FUNDAMENTALS_INPUT\""
  if [[ "$LAST_STEP_EXIT_CODE" -eq 0 ]]; then
  run_formal_step \
    "market_fundamentals_import" \
    "Import market fundamentals" \
    "\"$QUANTIX_BIN\" data import-fundamentals --input \"$MARKET_FUNDAMENTALS_INPUT\""
  else
    echo "[INFO] market fundamentals validation failed; skipping import-fundamentals step"
  fi
else
  echo "[INFO] No market fundamentals input configured; skipping validate-fundamentals / import-fundamentals steps"
  echo "[INFO] To validate a fundamentals JSON against scratch ClickHouse first, run: $REHEARSAL_SCRIPT"
fi

run_formal_step \
  "market_foundation" \
  "Market foundation" \
  "\"$QUANTIX_BIN\" market foundation"

MARKET_DATE="$(resolve_market_date)"
echo "[INFO] Using market date for formal sequence: $MARKET_DATE"

run_formal_step \
  "market_strength" \
  "Market strength" \
  "\"$QUANTIX_BIN\" market strength --date $MARKET_DATE --strong-top 3 --weak-top 3 --stock-top 10"

MARKET_STRENGTH_LOG="$LOG_DIR/market_cli_market_strength_$STAMP.log"
MARKET_STRENGTH_STOCKS_SECTOR="$(resolve_strength_stocks_sector_from_log "$MARKET_STRENGTH_LOG")"
MARKET_STRENGTH_STOCKS_CMD="\"$QUANTIX_BIN\" market strength-stocks --date $MARKET_DATE --strong-top 3"
if [[ -n "$MARKET_STRENGTH_STOCKS_SECTOR" ]]; then
  echo "[INFO] Using dynamic strong sector for market strength-stocks: $MARKET_STRENGTH_STOCKS_SECTOR"
  MARKET_STRENGTH_STOCKS_CMD+=" --sector \"$MARKET_STRENGTH_STOCKS_SECTOR\""
else
  echo "[INFO] No strong sector extracted from market strength output; running market strength-stocks without sector filter"
fi
MARKET_STRENGTH_STOCKS_CMD+=" --metric profit --top 10"

run_formal_step \
  "market_strength_stocks" \
  "Market strength-stocks" \
  "$MARKET_STRENGTH_STOCKS_CMD"

echo "\nMarket CLI formal sequence completed."
