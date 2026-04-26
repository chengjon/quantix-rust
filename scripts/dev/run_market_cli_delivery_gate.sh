#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_FILE:-$LOG_DIR/run_market_cli_delivery_gate_$(date +%Y%m%d_%H%M%S).log}"

ACCEPTANCE_SCRIPT="${ACCEPTANCE_SCRIPT:-$ROOT_DIR/scripts/dev/run_market_cli_acceptance.sh}"
FORMAL_SEQUENCE_SCRIPT="${FORMAL_SEQUENCE_SCRIPT:-$ROOT_DIR/scripts/dev/run_market_cli_formal_sequence.sh}"
REPORT_SCRIPT="${REPORT_SCRIPT:-$ROOT_DIR/scripts/dev/generate_market_cli_acceptance_report.sh}"
REPORT_PATH="${REPORT_PATH:-$LOG_DIR/market_cli_delivery_gate_report_$(date +%Y%m%d_%H%M%S).md}"

export LOG_DIR
export ACCEPTANCE_SCRIPT
export FORMAL_SEQUENCE_SCRIPT
export REPORT_SCRIPT
export REPORT_PATH

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

run_step "Acceptance orchestration" "\"$ACCEPTANCE_SCRIPT\""
run_step "Formal sequence" "\"$FORMAL_SEQUENCE_SCRIPT\""
run_step "Acceptance report generation" "\"$REPORT_SCRIPT\""

echo "\n[NEXT]"
echo "  - 打开最终报告确认 precheck / smoke / formal 三层结果是否一致"
echo "  - 若 formal sequence 中 sync industry 或 market 命令 exit 非 0，先修复环境或依赖后再重跑"
echo "  - 若只有 warning，结合 acceptance report 中的 remediation 和 formal 摘要做收口判断"
echo "  - 默认报告位置: $REPORT_PATH"

echo "\nMarket CLI delivery gate completed."
