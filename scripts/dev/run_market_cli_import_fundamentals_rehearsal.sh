#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
STAMP="$(date +%Y%m%d_%H%M%S)"
LOG_FILE="${LOG_FILE:-$LOG_DIR/market_cli_import_fundamentals_rehearsal_$STAMP.log}"
IMPORT_LOG="${IMPORT_LOG:-$LOG_DIR/market_cli_import_fundamentals_step_$STAMP.log}"
QUANTIX_BIN="${QUANTIX_BIN:-$ROOT_DIR/target/debug/quantix}"
LOCAL_ENV_PATH="${LOCAL_ENV_PATH:-$ROOT_DIR/.env.market.local}"
ROOT_ENV_PATH="${ROOT_ENV_PATH:-$ROOT_DIR/.env}"
INIT_LOCAL_ENV_SCRIPT="${INIT_LOCAL_ENV_SCRIPT:-$ROOT_DIR/scripts/dev/init_market_cli_local_env.sh}"
MARKET_FUNDAMENTALS_SMOKE_INPUT="${MARKET_FUNDAMENTALS_SMOKE_INPUT:-/tmp/quantix_market_fundamentals_smoke.json}"

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

load_env_file "$LOCAL_ENV_PATH"
load_tdx_env_fallback_from_root "$ROOT_ENV_PATH"

CLICKHOUSE_URL="${CLICKHOUSE_URL:-http://localhost:8123}"
CLICKHOUSE_DB="${CLICKHOUSE_DB:-quantix}"
CLICKHOUSE_USER="${CLICKHOUSE_USER:-default}"
CLICKHOUSE_PASSWORD="${CLICKHOUSE_PASSWORD:-}"
MARKET_FUNDAMENTALS_REHEARSAL_INPUT="${MARKET_FUNDAMENTALS_REHEARSAL_INPUT:-${MARKET_FUNDAMENTALS_INPUT:-}}"
MARKET_FUNDAMENTALS_REHEARSAL_STATUS_CMD="${MARKET_FUNDAMENTALS_REHEARSAL_STATUS_CMD:-}"
MARKET_FUNDAMENTALS_REHEARSAL_DB="${MARKET_FUNDAMENTALS_REHEARSAL_DB:-${CLICKHOUSE_DB}_mf_rehearsal_$STAMP}"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Market fundamentals import rehearsal log: $LOG_FILE"

validate_clickhouse_db_name() {
  local db="$1"
  if [[ ! "$db" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
    echo "[FAIL] Invalid scratch ClickHouse database name: $db"
    echo "[HINT] Use only letters, digits, and underscores, starting with a letter or underscore."
    exit 1
  fi
}

resolve_rehearsal_input() {
  if [[ -n "$MARKET_FUNDAMENTALS_REHEARSAL_INPUT" ]]; then
    REHEARSAL_INPUT_MODE="explicit"
    return 0
  fi

  if [[ -f "$MARKET_FUNDAMENTALS_SMOKE_INPUT" && -s "$MARKET_FUNDAMENTALS_SMOKE_INPUT" ]]; then
    MARKET_FUNDAMENTALS_REHEARSAL_INPUT="$MARKET_FUNDAMENTALS_SMOKE_INPUT"
    REHEARSAL_INPUT_MODE="smoke_fixture"
    return 0
  fi

  REHEARSAL_INPUT_MODE="missing"
}

clickhouse_query() {
  local database="$1"
  local sql="$2"
  local auth_args=()
  if [[ -n "$CLICKHOUSE_USER" || -n "$CLICKHOUSE_PASSWORD" ]]; then
    auth_args+=(--user "${CLICKHOUSE_USER}:${CLICKHOUSE_PASSWORD}")
  fi

  curl -sS "${auth_args[@]}" \
    --get \
    --data-urlencode "database=$database" \
    --data-urlencode "query=$sql" \
    "$CLICKHOUSE_URL"
}

default_rehearsal_status_probe() {
  local exists_sql="SELECT count() FROM system.tables WHERE database = '${MARKET_FUNDAMENTALS_REHEARSAL_DB}' AND name = 'market_fundamentals_daily' FORMAT TabSeparatedRaw"
  local exists=""
  exists="$(clickhouse_query "$MARKET_FUNDAMENTALS_REHEARSAL_DB" "$exists_sql" | tr -d '\r' | tail -n 1 | tr -d '[:space:]')"

  if [[ "$exists" != "1" ]]; then
    echo "state=missing"
    echo "rows=0"
    echo "latest_snapshot=N/A"
    return 0
  fi

  local summary_sql="SELECT count(), max(snapshot_date) FROM ${MARKET_FUNDAMENTALS_REHEARSAL_DB}.market_fundamentals_daily FORMAT TabSeparatedRaw"
  local summary=""
  summary="$(clickhouse_query "$MARKET_FUNDAMENTALS_REHEARSAL_DB" "$summary_sql" | tr -d '\r' | tail -n 1)"
  [[ -n "$summary" ]] || return 1

  local rows latest_snapshot
  rows="$(awk -F'\t' 'NR==1 { print $1 }' <<<"$summary" | tr -d '[:space:]')"
  latest_snapshot="$(awk -F'\t' 'NR==1 { print $2 }' <<<"$summary" | tr -d '[:space:]')"
  [[ -n "$rows" ]] || return 1

  if [[ "$rows" == "0" ]]; then
    echo "state=empty"
  else
    echo "state=populated"
  fi
  echo "rows=$rows"
  if [[ -n "$latest_snapshot" && "$latest_snapshot" != "\\N" ]]; then
    echo "latest_snapshot=$latest_snapshot"
  else
    echo "latest_snapshot=N/A"
  fi
}

probe_rehearsal_status() {
  if [[ -n "$MARKET_FUNDAMENTALS_REHEARSAL_STATUS_CMD" ]]; then
    bash -lc "$MARKET_FUNDAMENTALS_REHEARSAL_STATUS_CMD"
    return
  fi

  default_rehearsal_status_probe
}

extract_probe_field() {
  local key="$1"
  local output="$2"
  grep -E "^${key}=" <<<"$output" | tail -n 1 | cut -d= -f2- || true
}

REHEARSAL_INPUT_MODE=""
resolve_rehearsal_input

case "$REHEARSAL_INPUT_MODE" in
  explicit)
    ;;
  smoke_fixture)
    echo "[WARN] No explicit fundamentals JSON configured; using smoke fixture in scratch DB only: $MARKET_FUNDAMENTALS_REHEARSAL_INPUT"
    ;;
  missing)
    echo "[FAIL] No rehearsal input available."
    echo "[HINT] Set MARKET_FUNDAMENTALS_REHEARSAL_INPUT or MARKET_FUNDAMENTALS_INPUT to a MarketFundamentalSyncRecord JSON file."
    echo "[HINT] Optional smoke fallback path checked: $MARKET_FUNDAMENTALS_SMOKE_INPUT"
    exit 1
    ;;
esac

if [[ ! -f "$MARKET_FUNDAMENTALS_REHEARSAL_INPUT" || ! -s "$MARKET_FUNDAMENTALS_REHEARSAL_INPUT" ]]; then
  echo "[FAIL] Rehearsal input file missing or empty: $MARKET_FUNDAMENTALS_REHEARSAL_INPUT"
  exit 1
fi

validate_clickhouse_db_name "$MARKET_FUNDAMENTALS_REHEARSAL_DB"

echo "[INFO] Using scratch ClickHouse database: $MARKET_FUNDAMENTALS_REHEARSAL_DB"
echo "[INFO] Using rehearsal input mode: $REHEARSAL_INPUT_MODE"
echo "[INFO] Using rehearsal input file: $MARKET_FUNDAMENTALS_REHEARSAL_INPUT"
echo "[INFO] Import step log: $IMPORT_LOG"

set +e
env \
  CLICKHOUSE_URL="$CLICKHOUSE_URL" \
  CLICKHOUSE_DB="$MARKET_FUNDAMENTALS_REHEARSAL_DB" \
  CLICKHOUSE_USER="$CLICKHOUSE_USER" \
  CLICKHOUSE_PASSWORD="$CLICKHOUSE_PASSWORD" \
  "$QUANTIX_BIN" data import-fundamentals --input "$MARKET_FUNDAMENTALS_REHEARSAL_INPUT" \
  > >(tee "$IMPORT_LOG") 2>&1
IMPORT_EXIT=$?
set -e

echo "[RESULT] import_exit=$IMPORT_EXIT"
if [[ $IMPORT_EXIT -ne 0 ]]; then
  echo "[FAIL] Scratch fundamentals import rehearsal failed."
  exit "$IMPORT_EXIT"
fi

IMPORT_RECORDS="$(grep -E '^  记录数:' "$IMPORT_LOG" | head -n 1 | sed 's/^  记录数: //')"
IMPORT_WRITTEN="$(grep -E '^  已写入:' "$IMPORT_LOG" | head -n 1 | sed 's/^  已写入: //')"
IMPORT_ELAPSED="$(grep -E '^  耗时\(秒\):' "$IMPORT_LOG" | head -n 1 | sed 's/^  耗时(秒): //')"

set +e
STATUS_OUTPUT="$(probe_rehearsal_status 2>&1)"
STATUS_EXIT=$?
set -e

if [[ -n "$STATUS_OUTPUT" ]]; then
  echo "$STATUS_OUTPUT"
fi

if [[ $STATUS_EXIT -ne 0 ]]; then
  echo "[FAIL] Scratch fundamentals status probe failed."
  exit "$STATUS_EXIT"
fi

TABLE_STATE="$(extract_probe_field "state" "$STATUS_OUTPUT" | tr -d '\r')"
TABLE_ROWS="$(extract_probe_field "rows" "$STATUS_OUTPUT" | tr -d '\r')"
LATEST_SNAPSHOT="$(extract_probe_field "latest_snapshot" "$STATUS_OUTPUT" | tr -d '\r')"

TABLE_STATE="${TABLE_STATE:-unavailable}"
TABLE_ROWS="${TABLE_ROWS:-N/A}"
LATEST_SNAPSHOT="${LATEST_SNAPSHOT:-N/A}"

echo "[FIELD] rehearsal_input=$MARKET_FUNDAMENTALS_REHEARSAL_INPUT"
echo "[FIELD] rehearsal_input_mode=$REHEARSAL_INPUT_MODE"
echo "[FIELD] rehearsal_clickhouse_url=$CLICKHOUSE_URL"
echo "[FIELD] rehearsal_clickhouse_db=$MARKET_FUNDAMENTALS_REHEARSAL_DB"
echo "[FIELD] rehearsal_import_records=${IMPORT_RECORDS:-N/A}"
echo "[FIELD] rehearsal_import_written=${IMPORT_WRITTEN:-N/A}"
echo "[FIELD] rehearsal_import_elapsed_seconds=${IMPORT_ELAPSED:-N/A}"
echo "[FIELD] rehearsal_table_state=$TABLE_STATE"
echo "[FIELD] rehearsal_table_rows=$TABLE_ROWS"
echo "[FIELD] rehearsal_latest_snapshot=$LATEST_SNAPSHOT"

if [[ "$TABLE_STATE" != "populated" || "$TABLE_ROWS" == "0" ]]; then
  echo "[FAIL] Scratch DB fundamentals table is not populated after import."
  exit 1
fi

echo "[INFO] Scratch DB retained for inspection: $MARKET_FUNDAMENTALS_REHEARSAL_DB"
echo "[INFO] Production quantix DB was not modified."
echo "Market fundamentals import rehearsal completed."
