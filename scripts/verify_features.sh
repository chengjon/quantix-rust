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

run_expect_pass() {
  local name="$1"
  local cmd="$2"
  echo "\n[CHECK] $name"
  if bash -lc "$cmd"; then
    echo "[PASS] $name"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $name"
    FAIL=$((FAIL + 1))
  fi
}

run_expect_warn() {
  local name="$1"
  local cmd="$2"
  local hint="$3"
  echo "\n[CHECK] $name"
  set +e
  local output
  output=$(bash -lc "$cmd" 2>&1)
  local code=$?
  set -e

  echo "$output"
  if echo "$output" | grep -Eiq "$hint"; then
    echo "[WARN-EXPECTED] $name"
    WARN=$((WARN + 1))
  elif [[ $code -eq 0 ]]; then
    echo "[PASS] $name (feature may be implemented now)"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $name"
    FAIL=$((FAIL + 1))
  fi
}

# 1) Basic health
run_expect_pass "Cargo version" "cargo --version"
run_expect_pass "Compile check" "cargo check -q"

# 2) Command tree reachability
run_expect_pass "CLI help" "cargo run -- --help"
run_expect_pass "Strategy help" "cargo run -- strategy --help"
run_expect_pass "Execution help" "cargo run -- execution --help"
run_expect_pass "Risk help" "cargo run -- risk --help"
run_expect_pass "Fundamental help" "cargo run -- fundamental --help"

# 3) Core smoke checks
run_expect_pass "Strategy list" "cargo run -- strategy list"
run_expect_pass "Signal list" "cargo run -- strategy signal list"
run_expect_pass "Request list stats" "cargo run -- strategy request list --stats"
run_expect_warn "Execution bridge status (external dependency)" "cargo run -- execution bridge status" "Connection refused|bridge request failed|timeout|timed out"
run_expect_pass "Fundamental valuation" "cargo run -- fundamental valuation --code 600519"
run_expect_pass "Fundamental earnings" "cargo run -- fundamental earnings --code 600519 --years 1"
run_expect_pass "Fundamental institution" "cargo run -- fundamental institution --code 600519"

# 4) Expected-limited features
run_expect_warn "TUI placeholder" "cargo run -- menu --tui" "开发中|提示|todo|暂未"
run_expect_warn "Parquet export placeholder" "cargo run -- data export --code 000001 --format parquet --output ./tmp" "暂未实现|未实现|unsupported|未找到数据|无数据|empty|no data|not found"
run_expect_warn "Dividend placeholder" "cargo run -- fundamental dividend --code 600519 --years 3" "开发中|敬请期待|未实现"
run_expect_warn "Task add P0 limitation" "cargo run -- task add --name demo --cron '0 9 * * *' --command 'echo hi'" "P0|暂不支持|unsupported"

echo "\n================ SUMMARY ================"
echo "PASS : $PASS"
echo "WARN : $WARN"
echo "FAIL : $FAIL"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi

echo "All checks passed (or expected-warn)."
