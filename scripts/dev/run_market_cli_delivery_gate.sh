#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
STAMP="$(date +%Y%m%d_%H%M%S)"
LOG_FILE="${LOG_FILE:-$LOG_DIR/run_market_cli_delivery_gate_$STAMP.log}"

ACCEPTANCE_SCRIPT="${ACCEPTANCE_SCRIPT:-$ROOT_DIR/scripts/dev/run_market_cli_acceptance.sh}"
FORMAL_SEQUENCE_SCRIPT="${FORMAL_SEQUENCE_SCRIPT:-$ROOT_DIR/scripts/dev/run_market_cli_formal_sequence.sh}"
REPORT_SCRIPT="${REPORT_SCRIPT:-$ROOT_DIR/scripts/dev/generate_market_cli_acceptance_report.sh}"
FORMAL_LOG="${FORMAL_LOG:-$LOG_DIR/market_cli_formal_sequence_$STAMP.log}"
SUMMARY_LOG="${SUMMARY_LOG:-$FORMAL_LOG}"
REPORT_PATH="${REPORT_PATH:-$LOG_DIR/market_cli_delivery_gate_report_$STAMP.md}"

export LOG_DIR
export ACCEPTANCE_SCRIPT
export FORMAL_SEQUENCE_SCRIPT
export REPORT_SCRIPT
export FORMAL_LOG
export REPORT_PATH
export SUMMARY_LOG

exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Market CLI delivery gate log: $LOG_FILE"
echo "[INFO] Acceptance script: $ACCEPTANCE_SCRIPT"
echo "[INFO] Formal sequence script: $FORMAL_SEQUENCE_SCRIPT"
echo "[INFO] Report script: $REPORT_SCRIPT"
echo "[INFO] Report path: $REPORT_PATH"

run_step() {
  local name="$1"
  local cmd="$2"
  echo "\n[STEP] $name"
  bash -lc "$cmd"
}

assert_formal_success() {
  local formal_log="$1"

  if [[ ! -f "$formal_log" ]]; then
    echo "[GATE-FAIL] formal sequence log missing: $formal_log"
    return 1
  fi

  local failed_steps
  failed_steps="$(grep -E '^\[RESULT\] .+_exit=[1-9][0-9]*$' "$formal_log" || true)"
  if [[ -n "$failed_steps" ]]; then
    echo "[GATE-FAIL] Formal sequence contains non-zero step exits:"
    echo "$failed_steps"
    return 1
  fi

  echo "[PASS] Formal sequence gate verdict: all recorded step exits are zero"
}

run_step "Acceptance orchestration" "\"$ACCEPTANCE_SCRIPT\""
run_step "Formal sequence" "\"$FORMAL_SEQUENCE_SCRIPT\""
assert_formal_success "$FORMAL_LOG"
run_step "Acceptance report generation" "\"$REPORT_SCRIPT\""

echo "\n[NEXT]"
echo "  - 打开最终报告确认 precheck / smoke / formal 三层结果是否一致"
echo "  - 若 formal sequence 中 sync industry 或 market 命令 exit 非 0，先修复环境或依赖后再重跑"
echo "  - 若报告显示 fundamentals_state=missing 或 empty，先运行 scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh 验证 JSON 与 ClickHouse scratch 导入链路，再执行 quantix data validate-fundamentals --input /abs/path/market_fundamentals.json 与 quantix data import-fundamentals --input /abs/path/market_fundamentals.json"
echo "  - 若只有 warning，结合 acceptance report 中的 remediation 和 formal 摘要做收口判断"
echo "  - 默认报告位置: $REPORT_PATH"

echo "\nMarket CLI delivery gate completed."
