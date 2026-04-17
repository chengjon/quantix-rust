#!/usr/bin/env bash
set -euo pipefail

# Quantix feature smoke verification script
# Run in repo root (recommended under root dev environment)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="$ROOT_DIR/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_FILE:-$LOG_DIR/verify_features_$(date +%Y%m%d_%H%M%S).log}"

# Mirror all output to terminal + log file for full audit trail
exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Feature verification log: $LOG_FILE"

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
    pass)
      PASS=$((PASS + 1))
      ;;
    warn)
      WARN=$((WARN + 1))
      ;;
    fail)
      FAIL=$((FAIL + 1))
      ;;
  esac

  case "$category:$outcome" in
    local:pass)
      LOCAL_PASS=$((LOCAL_PASS + 1))
      ;;
    local:warn)
      LOCAL_WARN=$((LOCAL_WARN + 1))
      ;;
    local:fail)
      LOCAL_FAIL=$((LOCAL_FAIL + 1))
      ;;
    external:pass)
      EXTERNAL_PASS=$((EXTERNAL_PASS + 1))
      ;;
    external:warn)
      EXTERNAL_WARN=$((EXTERNAL_WARN + 1))
      ;;
    external:fail)
      EXTERNAL_FAIL=$((EXTERNAL_FAIL + 1))
      ;;
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
    echo "[PASS] $name (feature may be implemented now)"
    record_result "$category" pass
  else
    echo "[FAIL] $name"
    record_result "$category" fail
  fi
}

# 1) Basic health
run_expect_pass "Cargo version" "cargo --version"
run_expect_pass "Compile check" "cargo check -q"
run_expect_pass "Build quantix binary" "cargo build -q --bin quantix"

# 2) Command tree reachability
run_expect_pass "CLI help" "\"$QUANTIX_BIN\" --help"
run_expect_pass "Strategy help" "\"$QUANTIX_BIN\" strategy --help"
run_expect_pass "Execution help" "\"$QUANTIX_BIN\" execution --help"
run_expect_pass "Risk help" "\"$QUANTIX_BIN\" risk --help"
run_expect_pass "Fundamental help" "\"$QUANTIX_BIN\" fundamental --help"

# 3) Local binary smoke checks
run_expect_pass "Strategy list" "\"$QUANTIX_BIN\" strategy list"
run_expect_pass "Signal list" "\"$QUANTIX_BIN\" strategy signal list"
run_expect_pass "Request list stats" "\"$QUANTIX_BIN\" strategy request list --stats"
run_expect_pass "Execution config show" "\"$QUANTIX_BIN\" execution config show"
run_expect_pass "Execution daemon run once" "\"$QUANTIX_BIN\" execution daemon run --once"

# 4) External dependency smoke checks
run_expect_warn "Execution bridge status (external dependency)" "\"$QUANTIX_BIN\" execution bridge status" "Connection refused|bridge request failed|timeout|timed out" external
run_expect_pass "Fundamental valuation" "\"$QUANTIX_BIN\" fundamental valuation --code 600519" external
run_expect_pass "Fundamental earnings" "\"$QUANTIX_BIN\" fundamental earnings --code 600519 --years 1" external
run_expect_pass "Fundamental institution" "\"$QUANTIX_BIN\" fundamental institution --code 600519" external

# 5) Expected-limited features
run_expect_warn "TUI placeholder" "\"$QUANTIX_BIN\" menu --tui" "开发中|提示|todo|暂未"
run_expect_warn "Parquet export placeholder" "\"$QUANTIX_BIN\" data export --code 000001 --format parquet --output ./tmp" "暂未实现|未实现|unsupported|未找到数据|无数据|empty|no data|not found"
run_expect_warn "Dividend placeholder" "\"$QUANTIX_BIN\" fundamental dividend --code 600519 --years 3" "开发中|敬请期待|未实现"
run_expect_warn "Task add P0 limitation" "\"$QUANTIX_BIN\" task add --name demo --cron '0 9 * * *' --command 'echo hi'" "P0|暂不支持|unsupported"

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

echo "All checks passed (or expected-warn)."
