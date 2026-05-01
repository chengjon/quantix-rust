#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_FILE:-$LOG_DIR/check_market_cli_prereqs_$(date +%Y%m%d_%H%M%S).log}"
ENV_TEMPLATE_PATH="${ENV_TEMPLATE_PATH:-$ROOT_DIR/scripts/dev/market_cli_env.example.sh}"
LOCAL_ENV_PATH="${LOCAL_ENV_PATH:-$ROOT_DIR/.env.market.local}"
ROOT_ENV_PATH="${ROOT_ENV_PATH:-$ROOT_DIR/.env}"

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

load_env_file "$LOCAL_ENV_PATH"
load_tdx_env_fallback_from_root "$ROOT_ENV_PATH"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Market CLI prerequisite log: $LOG_FILE"

PASS=0
WARN=0
FAIL=0
WARNINGS=()
QUANTIX_BIN="${QUANTIX_BIN:-$ROOT_DIR/target/debug/quantix}"
RISK_DIR="${QUANTIX_RISK_DIR:-$HOME/.quantix/risk}"
INDUSTRY_DB_PATH="${QUANTIX_INDUSTRY_DB_PATH:-$RISK_DIR/industry_reference.db}"
TDX_ROOT="${QUANTIX_TDX_ROOT:-${TDX_ROOT:-}}"
TDX_MARKET="${QUANTIX_TDX_MARKET:-${TDX_MARKET:-}}"
MARKET_SNAPSHOT_SOURCE="${QUANTIX_MARKET_SNAPSHOT_SOURCE:-auto}"
CLICKHOUSE_URL="${CLICKHOUSE_URL:-http://localhost:8123}"
CLICKHOUSE_DB="${CLICKHOUSE_DB:-quantix}"
CLICKHOUSE_USER="${CLICKHOUSE_USER:-default}"
CLICKHOUSE_PASSWORD="${CLICKHOUSE_PASSWORD:-}"
UPSTREAM_MYSQL_URL="${QUANTIX_UPSTREAM_MYSQL_URL:-}"
UPSTREAM_MYSQL_DB="${QUANTIX_UPSTREAM_MYSQL_DB:-}"
UPSTREAM_MYSQL_USER="${QUANTIX_UPSTREAM_MYSQL_USER:-}"
MARKET_SNAPSHOT_PROBE_URL="${MARKET_SNAPSHOT_PROBE_URL:-https://push2.eastmoney.com/api/qt/clist/get?pn=1&pz=5&po=1&np=1&fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23&fields=f12,f14,f2,f3,f5,f6}"
MARKET_SNAPSHOT_PROBE_CMD="${MARKET_SNAPSHOT_PROBE_CMD:-curl -sS \"$MARKET_SNAPSHOT_PROBE_URL\" -H \"Referer: https://data.eastmoney.com/\" -H \"User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36\" >/dev/null}"
MARKET_FUNDAMENTALS_INPUT="${MARKET_FUNDAMENTALS_INPUT:-}"
MARKET_FUNDAMENTALS_STATUS_CMD="${MARKET_FUNDAMENTALS_STATUS_CMD:-}"

record_result() {
  local outcome="$1"
  case "$outcome" in
    pass) PASS=$((PASS + 1)) ;;
    warn) WARN=$((WARN + 1)) ;;
    fail) FAIL=$((FAIL + 1)) ;;
  esac
}

record_warning_detail() {
  local detail="$1"
  WARNINGS+=("$detail")
}

check_pass() {
  local name="$1"
  local check_cmd="$2"
  echo "\n[CHECK] $name"
  if bash -lc "$check_cmd"; then
    echo "[PASS] $name"
    record_result pass
  else
    echo "[FAIL] $name"
    record_result fail
  fi
}

check_warn_if_missing() {
  local name="$1"
  local check_cmd="$2"
  local hint="$3"
  echo "\n[CHECK] $name"
  set +e
  local output
  output=$(bash -lc "$check_cmd" 2>&1)
  local code=$?
  set -e

  echo "$output"
  if [[ $code -eq 0 ]]; then
    echo "[PASS] $name"
    record_result pass
  elif echo "$output" | grep -Eiq "$hint"; then
    echo "[WARN] $name"
    record_result warn
    case "$name" in
      "Shenwan SQLite reference DB present")
        record_warning_detail "缺少本地行业 SQLite：先 source $ENV_TEMPLATE_PATH，再运行 quantix risk sync industry --standard shenwan"
        ;;
      "Upstream MySQL env configured for risk sync")
        record_warning_detail "缺少上游 MySQL 环境变量：请 source $ENV_TEMPLATE_PATH，并填写 QUANTIX_UPSTREAM_MYSQL_URL / QUANTIX_UPSTREAM_MYSQL_DB / QUANTIX_UPSTREAM_MYSQL_USER / QUANTIX_UPSTREAM_MYSQL_PASSWORD"
        ;;
      "ClickHouse env resolved for market strength")
        record_warning_detail "缺少 ClickHouse 配置：请 source $ENV_TEMPLATE_PATH，并填写 CLICKHOUSE_URL / CLICKHOUSE_DB"
        ;;
      "EastMoney A-share snapshot upstream reachable")
        record_warning_detail "A股全市场快照上游当前不可达：market foundation 与 market strength 会尝试退回 TDX 实时行情；若 TDX 也不可用则运行失败。若同时缺少本地 market_fundamentals_daily 且 EastMoney 单股基本面不可达，strength / strength-stocks 的总市值、净利润 TopN 可能为空"
        ;;
      "Optional market fundamentals import input present")
        record_warning_detail "如已准备 MarketFundamentalSyncRecord JSON，请确认 MARKET_FUNDAMENTALS_INPUT 指向有效文件；formal sequence 会自动先运行 quantix data validate-fundamentals --input \"\$MARKET_FUNDAMENTALS_INPUT\"，再运行 quantix data import-fundamentals --input \"\$MARKET_FUNDAMENTALS_INPUT\"，以补齐本地 market_fundamentals_daily"
        ;;
    esac
  else
    echo "[FAIL] $name"
    record_result fail
  fi
}

clickhouse_query() {
  local sql="$1"
  local auth_args=()
  if [[ -n "$CLICKHOUSE_USER" || -n "$CLICKHOUSE_PASSWORD" ]]; then
    auth_args+=(--user "${CLICKHOUSE_USER}:${CLICKHOUSE_PASSWORD}")
  fi

  curl -sS "${auth_args[@]}" \
    --get \
    --data-urlencode "database=$CLICKHOUSE_DB" \
    --data-urlencode "query=$sql" \
    "$CLICKHOUSE_URL"
}

default_market_fundamentals_status_probe() {
  local exists_sql="SELECT count() FROM system.tables WHERE database = currentDatabase() AND name = 'market_fundamentals_daily' FORMAT TabSeparatedRaw"
  local exists=""
  exists="$(clickhouse_query "$exists_sql" | tr -d '\r' | tail -n 1 | tr -d '[:space:]')"

  if [[ "$exists" != "1" ]]; then
    echo "state=missing"
    echo "rows=0"
    echo "latest_snapshot=N/A"
    return 0
  fi

  local summary_sql="SELECT count(), max(snapshot_date) FROM market_fundamentals_daily FORMAT TabSeparatedRaw"
  local summary=""
  summary="$(clickhouse_query "$summary_sql" | tr -d '\r' | tail -n 1)"
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

probe_market_fundamentals_status() {
  if [[ -n "$MARKET_FUNDAMENTALS_STATUS_CMD" ]]; then
    bash -lc "$MARKET_FUNDAMENTALS_STATUS_CMD"
    return
  fi

  default_market_fundamentals_status_probe
}

extract_probe_field() {
  local key="$1"
  local output="$2"
  grep -E "^${key}=" <<<"$output" | tail -n 1 | cut -d= -f2- || true
}

check_market_fundamentals_table_status() {
  echo "\n[CHECK] Local market fundamentals table ready for TopN ranking"
  set +e
  local output
  output="$(probe_market_fundamentals_status 2>&1)"
  local code=$?
  set -e

  if [[ -n "$output" ]]; then
    echo "$output"
  fi

  local state rows latest_snapshot
  state="$(extract_probe_field "state" "$output" | tr -d '\r')"
  rows="$(extract_probe_field "rows" "$output" | tr -d '\r')"
  latest_snapshot="$(extract_probe_field "latest_snapshot" "$output" | tr -d '\r')"

  state="${state:-unavailable}"
  rows="${rows:-N/A}"
  latest_snapshot="${latest_snapshot:-N/A}"

  if [[ "$state" == "missing" ]]; then
    rows="0"
    latest_snapshot="N/A"
  elif [[ "$state" == "empty" || "$rows" == "0" ]]; then
    latest_snapshot="N/A"
  fi

  echo "[FIELD] precheck_market_fundamentals_state=$state"
  echo "[FIELD] precheck_market_fundamentals_rows=$rows"
  echo "[FIELD] precheck_market_fundamentals_latest_snapshot=$latest_snapshot"

  case "$state" in
    populated)
      echo "[PASS] Local market fundamentals table ready for TopN ranking"
      record_result pass
      ;;
    empty)
      echo "[WARN] Local market fundamentals table ready for TopN ranking"
      record_result warn
      record_warning_detail "本地 market_fundamentals_daily 已建表但仍为空：建议先运行 scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh 做 scratch DB 导入演练；确认 JSON 与 ClickHouse 链路正常后，先运行 quantix data validate-fundamentals --input /abs/path/market_fundamentals.json，再运行 quantix data import-fundamentals --input /abs/path/market_fundamentals.json 补齐总市值/净利润 TopN"
      ;;
    missing)
      echo "[WARN] Local market fundamentals table ready for TopN ranking"
      record_result warn
      record_warning_detail "本地 market_fundamentals_daily 尚未建表：建议先运行 scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh 做 scratch DB 导入演练；确认 JSON 与 ClickHouse 链路正常后，先运行 quantix data validate-fundamentals --input /abs/path/market_fundamentals.json，再运行 quantix data import-fundamentals --input /abs/path/market_fundamentals.json 补齐总市值/净利润 TopN"
      ;;
    *)
      echo "[INFO] Local market fundamentals table status probe unavailable (exit=$code); keeping this as non-blocking diagnostics."
      ;;
  esac
}

check_pass "Quantix binary exists" "test -x \"$QUANTIX_BIN\""
check_pass "Market command tree reachable" "\"$QUANTIX_BIN\" market --help >/dev/null"
check_pass "Risk command tree reachable" "\"$QUANTIX_BIN\" risk --help >/dev/null"

check_warn_if_missing \
  "Shenwan SQLite reference DB present" \
  "ls \"$INDUSTRY_DB_PATH\" >/dev/null && test -s \"$INDUSTRY_DB_PATH\"" \
  "No such file|cannot access|not found"

check_warn_if_missing \
  "Upstream MySQL env configured for risk sync" \
  "test -n \"$UPSTREAM_MYSQL_URL\" && test -n \"$UPSTREAM_MYSQL_DB\" && test -n \"$UPSTREAM_MYSQL_USER\"" \
  "^$"

check_warn_if_missing \
  "ClickHouse env resolved for market strength" \
  "test -n \"$CLICKHOUSE_URL\" && test -n \"$CLICKHOUSE_DB\"" \
  "^$"

check_market_fundamentals_table_status

if [[ "${MARKET_SNAPSHOT_SOURCE,,}" == "tdx" ]]; then
  echo "\n[CHECK] EastMoney A-share snapshot upstream reachable"
  echo "[INFO] QUANTIX_MARKET_SNAPSHOT_SOURCE=tdx：跳过 EastMoney 连通性探测，运行时将直接使用 TDX 快照"
  echo "[PASS] EastMoney A-share snapshot upstream reachable"
  record_result pass
else
  check_warn_if_missing \
    "EastMoney A-share snapshot upstream reachable" \
    "$MARKET_SNAPSHOT_PROBE_CMD" \
    "curl: \\(52\\)|Empty reply from server|Connection refused|Could not resolve host|timed out|SSL connect error"
fi

if [[ -n "$MARKET_FUNDAMENTALS_INPUT" ]]; then
  check_warn_if_missing \
    "Optional market fundamentals import input present" \
    "test -f \"$MARKET_FUNDAMENTALS_INPUT\" && test -s \"$MARKET_FUNDAMENTALS_INPUT\"" \
    "No such file|cannot access|not found|^$"
fi

echo "\n[INFO] Expected runtime endpoints"
echo "  ClickHouse URL : $CLICKHOUSE_URL"
echo "  ClickHouse DB  : $CLICKHOUSE_DB"
echo "  ClickHouse user: $CLICKHOUSE_USER"
echo "  Market snapshot mode : $MARKET_SNAPSHOT_SOURCE"
echo "  Snapshot probe : $MARKET_SNAPSHOT_PROBE_URL"
echo "  Fundamentals   : ${MARKET_FUNDAMENTALS_INPUT:-unset}"
echo "  Industry SQLite: $INDUSTRY_DB_PATH"
if [[ -n "$TDX_ROOT" ]]; then
  if [[ -d "$TDX_ROOT" ]]; then
    echo "  TDX root       : $TDX_ROOT"
  else
    echo "  TDX root       : $TDX_ROOT (missing on this machine)"
  fi
else
  echo "  TDX root       : unset"
fi
echo "  TDX market     : ${TDX_MARKET:-unset}"
echo "  TDX quote mode : direct TDX quote-server fallback for market foundation/strength; GUI not required"
echo "  Env template   : $ENV_TEMPLATE_PATH"
echo "  Local env file : $LOCAL_ENV_PATH"
if [[ -n "$UPSTREAM_MYSQL_URL" ]]; then
  echo "  Upstream MySQL : configured"
else
echo "  Upstream MySQL : missing env"
fi
echo "  Scratch rehearsal: scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh"
echo "  TopN bootstrap : quantix data validate-fundamentals --input /abs/path/market_fundamentals.json && quantix data import-fundamentals --input /abs/path/market_fundamentals.json"

if [[ ${#WARNINGS[@]} -gt 0 ]]; then
  echo "\n[REMEDIATION]"
  for detail in "${WARNINGS[@]}"; do
    echo "  - $detail"
  done
fi

echo "\n================ SUMMARY ================"
echo "PASS : $PASS"
echo "WARN : $WARN"
echo "FAIL : $FAIL"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi

echo "Market CLI prerequisite checks passed (or warned on missing external prerequisites)."
