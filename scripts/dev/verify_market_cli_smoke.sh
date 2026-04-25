#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="$ROOT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_FILE:-$LOG_DIR/verify_market_cli_smoke_$(date +%Y%m%d_%H%M%S).log}"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Market CLI smoke log: $LOG_FILE"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] cargo not found. Please install Rust toolchain first." >&2
  exit 127
fi

PASS=0
WARN=0
FAIL=0
LOCAL_PASS=0
LOCAL_WARN=0
LOCAL_FAIL=0
EXTERNAL_PASS=0
EXTERNAL_WARN=0
EXTERNAL_FAIL=0
QUANTIX_BIN="$ROOT_DIR/target/debug/quantix"

record_result() {
  local category="$1"
  local outcome="$2"

  case "$outcome" in
    pass) PASS=$((PASS + 1)) ;;
    warn) WARN=$((WARN + 1)) ;;
    fail) FAIL=$((FAIL + 1)) ;;
  esac

  case "$category:$outcome" in
    local:pass) LOCAL_PASS=$((LOCAL_PASS + 1)) ;;
    local:warn) LOCAL_WARN=$((LOCAL_WARN + 1)) ;;
    local:fail) LOCAL_FAIL=$((LOCAL_FAIL + 1)) ;;
    external:pass) EXTERNAL_PASS=$((EXTERNAL_PASS + 1)) ;;
    external:warn) EXTERNAL_WARN=$((EXTERNAL_WARN + 1)) ;;
    external:fail) EXTERNAL_FAIL=$((EXTERNAL_FAIL + 1)) ;;
  esac
}

run_expect_pass() {
  local name="$1"
  local cmd="$2"
  local category="${3:-local}"
  echo "\n[CHECK] $name"
  if bash -lc "$cmd"; then
    echo "[PASS] $name"
    record_result "$category" pass
  else
    echo "[FAIL] $name"
    record_result "$category" fail
  fi
}

run_expect_warn() {
  local name="$1"
  local cmd="$2"
  local hint="$3"
  local category="${4:-local}"
  echo "\n[CHECK] $name"
  set +e
  local output
  output=$(bash -lc "$cmd" 2>&1)
  local code=$?
  set -e

  echo "$output"
  if echo "$output" | grep -Eiq "$hint"; then
    echo "[WARN-EXPECTED] $name"
    record_result "$category" warn
  elif [[ $code -eq 0 ]]; then
    echo "[PASS] $name"
    record_result "$category" pass
  else
    echo "[FAIL] $name"
    record_result "$category" fail
  fi
}

# 1) Local reachability
run_expect_pass "Build quantix binary" "cargo build -q --bin quantix"
run_expect_pass "Market help" "\"$QUANTIX_BIN\" market --help"
run_expect_pass "Market foundation help" "\"$QUANTIX_BIN\" market foundation --help"
run_expect_pass "Market strength help" "\"$QUANTIX_BIN\" market strength --help"
run_expect_pass "Market strength-stocks help" "\"$QUANTIX_BIN\" market strength-stocks --help"

# 2) External dependency checks
run_expect_warn \
  "Risk sync industry Shenwan (external dependency)" \
  "\"$QUANTIX_BIN\" risk sync industry --standard shenwan" \
  "Connection refused|timed out|timeout|Access denied|access denied|MySQL|mysql|database|db error|network|No such file|refused|unreachable" \
  external

run_expect_warn \
  "Market foundation (external dependency)" \
  "\"$QUANTIX_BIN\" market foundation" \
  "Connection refused|timed out|timeout|industry|Industry|shenwan|SQLite|sqlite|未同步|sync industry|empty|no data|not found|网络|失败" \
  external

run_expect_warn \
  "Market strength (external dependency)" \
  "\"$QUANTIX_BIN\" market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10" \
  "Connection refused|timed out|timeout|industry|Industry|shenwan|SQLite|sqlite|未同步|sync industry|empty|no data|not found|网络|失败" \
  external

run_expect_warn \
  "Market strength-stocks (external dependency)" \
  "\"$QUANTIX_BIN\" market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10" \
  "Connection refused|timed out|timeout|industry|Industry|shenwan|SQLite|sqlite|未同步|sync industry|empty|no data|not found|网络|失败" \
  external

echo "\n================ SUMMARY ================"
echo "PASS : $PASS"
echo "WARN : $WARN"
echo "FAIL : $FAIL"
echo "LOCAL PASS : $LOCAL_PASS"
echo "LOCAL WARN : $LOCAL_WARN"
echo "LOCAL FAIL : $LOCAL_FAIL"
echo "EXTERNAL PASS : $EXTERNAL_PASS"
echo "EXTERNAL WARN : $EXTERNAL_WARN"
echo "EXTERNAL FAIL : $EXTERNAL_FAIL"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi

echo "Market CLI smoke checks passed (or expected-warn)."
