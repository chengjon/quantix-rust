#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="$ROOT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_FILE:-$LOG_DIR/check_market_cli_prereqs_$(date +%Y%m%d_%H%M%S).log}"
ENV_TEMPLATE_PATH="$ROOT_DIR/scripts/dev/market_cli_env.example.sh"
LOCAL_ENV_PATH="$ROOT_DIR/.env.market.local"

if [[ -f "$LOCAL_ENV_PATH" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$LOCAL_ENV_PATH"
  set +a
fi

exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Market CLI prerequisite log: $LOG_FILE"

PASS=0
WARN=0
FAIL=0
WARNINGS=()
QUANTIX_BIN="$ROOT_DIR/target/debug/quantix"
RISK_DIR="${QUANTIX_RISK_DIR:-$HOME/.quantix/risk}"
INDUSTRY_DB_PATH="${QUANTIX_INDUSTRY_DB_PATH:-$RISK_DIR/industry_reference.db}"
CLICKHOUSE_URL="${CLICKHOUSE_URL:-http://localhost:8123}"
CLICKHOUSE_DB="${CLICKHOUSE_DB:-quantix}"
UPSTREAM_MYSQL_URL="${QUANTIX_UPSTREAM_MYSQL_URL:-}"
UPSTREAM_MYSQL_DB="${QUANTIX_UPSTREAM_MYSQL_DB:-}"
UPSTREAM_MYSQL_USER="${QUANTIX_UPSTREAM_MYSQL_USER:-}"

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
    esac
  else
    echo "[FAIL] $name"
    record_result fail
  fi
}

check_pass "Quantix binary exists" "test -x \"$QUANTIX_BIN\""
check_pass "Market command tree reachable" "\"$QUANTIX_BIN\" market --help >/dev/null"
check_pass "Risk command tree reachable" "\"$QUANTIX_BIN\" risk --help >/dev/null"

check_warn_if_missing \
  "Shenwan SQLite reference DB present" \
  "test -f \"$INDUSTRY_DB_PATH\" && test -s \"$INDUSTRY_DB_PATH\"" \
  "No such file|cannot access|not found"

check_warn_if_missing \
  "Upstream MySQL env configured for risk sync" \
  "test -n \"$UPSTREAM_MYSQL_URL\" && test -n \"$UPSTREAM_MYSQL_DB\" && test -n \"$UPSTREAM_MYSQL_USER\"" \
  "^$"

check_warn_if_missing \
  "ClickHouse env resolved for market strength" \
  "test -n \"$CLICKHOUSE_URL\" && test -n \"$CLICKHOUSE_DB\"" \
  "^$"

echo "\n[INFO] Expected runtime endpoints"
echo "  ClickHouse URL : $CLICKHOUSE_URL"
echo "  ClickHouse DB  : $CLICKHOUSE_DB"
echo "  Industry SQLite: $INDUSTRY_DB_PATH"
echo "  Env template   : $ENV_TEMPLATE_PATH"
echo "  Local env file : $LOCAL_ENV_PATH"
if [[ -n "$UPSTREAM_MYSQL_URL" ]]; then
  echo "  Upstream MySQL : configured"
else
  echo "  Upstream MySQL : missing env"
fi

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
