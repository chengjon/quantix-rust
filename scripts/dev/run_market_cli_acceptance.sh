#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs}"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_FILE:-$LOG_DIR/run_market_cli_acceptance_$(date +%Y%m%d_%H%M%S).log}"
ENV_TEMPLATE_PATH="${ENV_TEMPLATE_PATH:-$ROOT_DIR/scripts/dev/market_cli_env.example.sh}"
LOCAL_ENV_PATH="${LOCAL_ENV_PATH:-$ROOT_DIR/.env.market.local}"
INIT_LOCAL_ENV_SCRIPT="${INIT_LOCAL_ENV_SCRIPT:-$ROOT_DIR/scripts/dev/init_market_cli_local_env.sh}"
PRECHECK_SCRIPT="${PRECHECK_SCRIPT:-$ROOT_DIR/scripts/dev/check_market_cli_prereqs.sh}"
SMOKE_SCRIPT="${SMOKE_SCRIPT:-$ROOT_DIR/scripts/dev/verify_market_cli_smoke.sh}"

exec > >(tee -a "$LOG_FILE") 2>&1

echo "[INFO] Market CLI acceptance log: $LOG_FILE"
echo "[INFO] Suggested first step: source $ENV_TEMPLATE_PATH"
echo "[INFO] Local-only override file: $LOCAL_ENV_PATH"

"$INIT_LOCAL_ENV_SCRIPT"

if [[ -f "$LOCAL_ENV_PATH" ]]; then
  echo "[INFO] Loading local market env overrides from $LOCAL_ENV_PATH"
  set -a
  # shellcheck disable=SC1090
  source "$LOCAL_ENV_PATH"
  set +a
fi

run_step() {
  local name="$1"
  local cmd="$2"
  echo "\n[STEP] $name"
  bash -lc "$cmd"
}

run_step "Environment precheck" "\"$PRECHECK_SCRIPT\""
run_step "Smoke verification" "\"$SMOKE_SCRIPT\""

echo "\n[NEXT]"
echo "  - 若 precheck / smoke 只出现 expected-warn，先按 [REMEDIATION] 补环境"
echo "  - 然后重跑本脚本，直到环境 warning 收敛到可接受范围"
echo "  - 如需本机持久复用成功配置，可复制 .env.market.local.example 为 .env.market.local 并填写真实值"
echo "  - 最后再执行正式命令链路："
echo "      quantix risk sync industry --standard shenwan"
echo "      quantix market foundation"
echo "      quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10"
echo "      quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10"

echo "\nMarket CLI acceptance orchestration completed."
