#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

LOG_DIR="$ROOT_DIR/logs"
REPORT_DIR="${REPORT_DIR:-$LOG_DIR}"
mkdir -p "$REPORT_DIR"

latest_log() {
  local pattern="$1"
  local latest
  latest=$(ls -1t "$LOG_DIR"/$pattern 2>/dev/null | head -n 1 || true)
  if [[ -n "$latest" ]]; then
    printf '%s' "$latest"
  fi
}

extract_last_field() {
  local pattern="$1"
  local file="$2"
  if [[ -f "$file" ]]; then
    grep -E "^$pattern" "$file" | tail -n 1 | sed "s/^$pattern//" || true
  fi
}

ACCEPTANCE_LOG="${ACCEPTANCE_LOG:-$(latest_log 'run_market_cli_acceptance_*.log')}"
PRECHECK_LOG="${PRECHECK_LOG:-$(latest_log 'check_market_cli_prereqs_*.log')}"
SMOKE_LOG="${SMOKE_LOG:-$(latest_log 'verify_market_cli_smoke_*.log')}"
FORMAL_LOG="${FORMAL_LOG:-$(latest_log 'market_cli_formal_sequence_*.log')}"
STAMP="$(date +%Y%m%d_%H%M%S)"
REPORT_PATH="${REPORT_PATH:-$REPORT_DIR/market_cli_acceptance_report_$STAMP.md}"
REHEARSAL_CMD="${REHEARSAL_CMD:-scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh}"

PRECHECK_PASS="$(extract_last_field 'PASS : ' "${PRECHECK_LOG:-}")"
PRECHECK_WARN="$(extract_last_field 'WARN : ' "${PRECHECK_LOG:-}")"
PRECHECK_FAIL="$(extract_last_field 'FAIL : ' "${PRECHECK_LOG:-}")"
PRECHECK_FUNDAMENTALS_STATE="$(extract_last_field '\[FIELD\] precheck_market_fundamentals_state=' "${PRECHECK_LOG:-}")"
PRECHECK_FUNDAMENTALS_ROWS="$(extract_last_field '\[FIELD\] precheck_market_fundamentals_rows=' "${PRECHECK_LOG:-}")"
PRECHECK_FUNDAMENTALS_LATEST_SNAPSHOT="$(extract_last_field '\[FIELD\] precheck_market_fundamentals_latest_snapshot=' "${PRECHECK_LOG:-}")"
SMOKE_PASS="$(extract_last_field 'PASS : ' "${SMOKE_LOG:-}")"
SMOKE_WARN="$(extract_last_field 'WARN : ' "${SMOKE_LOG:-}")"
SMOKE_FAIL="$(extract_last_field 'FAIL : ' "${SMOKE_LOG:-}")"
SYNC_EXIT="$(extract_last_field '\[RESULT\] sync_industry_exit=' "${FORMAL_LOG:-}")"
VALIDATE_EXIT="$(extract_last_field '\[RESULT\] market_fundamentals_validate_exit=' "${FORMAL_LOG:-}")"
FOUNDATION_EXIT="$(extract_last_field '\[RESULT\] market_foundation_exit=' "${FORMAL_LOG:-}")"
STRENGTH_EXIT="$(extract_last_field '\[RESULT\] market_strength_exit=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_EXIT="$(extract_last_field '\[RESULT\] market_strength_stocks_exit=' "${FORMAL_LOG:-}")"
SYNC_LOG_PATH="$(extract_last_field '\[LOG\] sync_industry_log=' "${FORMAL_LOG:-}")"
VALIDATE_LOG_PATH="$(extract_last_field '\[LOG\] market_fundamentals_validate_log=' "${FORMAL_LOG:-}")"
FOUNDATION_LOG_PATH="$(extract_last_field '\[LOG\] market_foundation_log=' "${FORMAL_LOG:-}")"
STRENGTH_LOG_PATH="$(extract_last_field '\[LOG\] market_strength_log=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_LOG_PATH="$(extract_last_field '\[LOG\] market_strength_stocks_log=' "${FORMAL_LOG:-}")"
SYNC_SUMMARY="$(extract_last_field '\[SUMMARY\] sync_industry_summary=' "${FORMAL_LOG:-}")"
VALIDATE_SUMMARY="$(extract_last_field '\[SUMMARY\] market_fundamentals_validate_summary=' "${FORMAL_LOG:-}")"
FOUNDATION_SUMMARY="$(extract_last_field '\[SUMMARY\] market_foundation_summary=' "${FORMAL_LOG:-}")"
STRENGTH_SUMMARY="$(extract_last_field '\[SUMMARY\] market_strength_summary=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_SUMMARY="$(extract_last_field '\[SUMMARY\] market_strength_stocks_summary=' "${FORMAL_LOG:-}")"
FORMAL_MARKET_DATE="$(extract_last_field '\[INFO\] Using market date for formal sequence: ' "${FORMAL_LOG:-}")"
VALIDATE_TOTAL_RECORDS="$(extract_last_field '\[FIELD\] market_fundamentals_validate_total_records=' "${FORMAL_LOG:-}")"
VALIDATE_UNIQUE_CODES="$(extract_last_field '\[FIELD\] market_fundamentals_validate_unique_codes=' "${FORMAL_LOG:-}")"
VALIDATE_SNAPSHOT_MIN="$(extract_last_field '\[FIELD\] market_fundamentals_validate_snapshot_min=' "${FORMAL_LOG:-}")"
VALIDATE_SNAPSHOT_MAX="$(extract_last_field '\[FIELD\] market_fundamentals_validate_snapshot_max=' "${FORMAL_LOG:-}")"
VALIDATE_MARKET_CAP_COVERAGE="$(extract_last_field '\[FIELD\] market_fundamentals_validate_market_cap_coverage=' "${FORMAL_LOG:-}")"
VALIDATE_PROFIT_COVERAGE="$(extract_last_field '\[FIELD\] market_fundamentals_validate_latest_report_profit_coverage=' "${FORMAL_LOG:-}")"
VALIDATE_WARNING_COUNT="$(extract_last_field '\[FIELD\] market_fundamentals_validate_warning_count=' "${FORMAL_LOG:-}")"
FOUNDATION_TOTAL="$(extract_last_field '\[FIELD\] market_foundation_total_stocks=' "${FORMAL_LOG:-}")"
FOUNDATION_CLASSIFIED="$(extract_last_field '\[FIELD\] market_foundation_classified_stocks=' "${FORMAL_LOG:-}")"
FOUNDATION_UNCLASSIFIED="$(extract_last_field '\[FIELD\] market_foundation_unclassified_stocks=' "${FORMAL_LOG:-}")"
FOUNDATION_SECTOR_COUNT="$(extract_last_field '\[FIELD\] market_foundation_sector_count=' "${FORMAL_LOG:-}")"
FOUNDATION_TOP_SECTOR="$(extract_last_field '\[FIELD\] market_foundation_top_sector=' "${FORMAL_LOG:-}")"
STRENGTH_BASE="$(extract_last_field '\[FIELD\] market_strength_base=' "${FORMAL_LOG:-}")"
STRENGTH_CANDIDATES="$(extract_last_field '\[FIELD\] market_strength_candidate_stock_count=' "${FORMAL_LOG:-}")"
STRENGTH_SNAPSHOT_SOURCE="$(extract_last_field '\[FIELD\] market_strength_snapshot_source=' "${FORMAL_LOG:-}")"
STRENGTH_TDX_COVERAGE="$(extract_last_field '\[FIELD\] market_strength_tdx_coverage=' "${FORMAL_LOG:-}")"
STRENGTH_TOP_STRONG="$(extract_last_field '\[FIELD\] market_strength_top_strong_sector=' "${FORMAL_LOG:-}")"
STRENGTH_TOP_WEAK="$(extract_last_field '\[FIELD\] market_strength_top_weak_sector=' "${FORMAL_LOG:-}")"
STRENGTH_TOP_CAP="$(extract_last_field '\[FIELD\] market_strength_top_market_cap_stock=' "${FORMAL_LOG:-}")"
STRENGTH_TOP_PROFIT="$(extract_last_field '\[FIELD\] market_strength_top_profit_stock=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_SECTOR="$(extract_last_field '\[FIELD\] market_strength_stocks_sector_filter=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_METRIC="$(extract_last_field '\[FIELD\] market_strength_stocks_metric=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_COVERAGE="$(extract_last_field '\[FIELD\] market_strength_stocks_coverage=' "${FORMAL_LOG:-}")"
STRENGTH_STOCKS_TOP_ROW="$(extract_last_field '\[FIELD\] market_strength_stocks_top_row=' "${FORMAL_LOG:-}")"

STRENGTH_CMD_DATE="${FORMAL_MARKET_DATE:-2026-03-09}"
STRENGTH_CMD="quantix market strength --date ${STRENGTH_CMD_DATE} --strong-top 3 --weak-top 3 --stock-top 10"
STRENGTH_STOCKS_METRIC_FLAG="profit"
case "${STRENGTH_STOCKS_METRIC:-}" in
  *市值*)
    STRENGTH_STOCKS_METRIC_FLAG="market-cap"
    ;;
esac
STRENGTH_STOCKS_CMD="quantix market strength-stocks --date ${STRENGTH_CMD_DATE} --strong-top 3"
if [[ -n "${STRENGTH_STOCKS_SECTOR:-}" && "${STRENGTH_STOCKS_SECTOR}" != "N/A" ]]; then
  STRENGTH_STOCKS_CMD+=" --sector ${STRENGTH_STOCKS_SECTOR}"
fi
STRENGTH_STOCKS_CMD+=" --metric ${STRENGTH_STOCKS_METRIC_FLAG} --top 10"

cat > "$REPORT_PATH" <<EOF
# Market CLI Acceptance Report

生成时间: $(date '+%Y-%m-%d %H:%M:%S %Z')

## 日志来源

- acceptance orchestrator: ${ACCEPTANCE_LOG:-未找到}
- precheck: ${PRECHECK_LOG:-未找到}
- smoke: ${SMOKE_LOG:-未找到}
- formal sequence: ${FORMAL_LOG:-未找到}

## 摘要

- precheck: PASS=${PRECHECK_PASS:-N/A} WARN=${PRECHECK_WARN:-N/A} FAIL=${PRECHECK_FAIL:-N/A}
  - fundamentals_state: ${PRECHECK_FUNDAMENTALS_STATE:-N/A}
  - fundamentals_rows: ${PRECHECK_FUNDAMENTALS_ROWS:-N/A}
  - fundamentals_latest_snapshot: ${PRECHECK_FUNDAMENTALS_LATEST_SNAPSHOT:-N/A}
- smoke: PASS=${SMOKE_PASS:-N/A} WARN=${SMOKE_WARN:-N/A} FAIL=${SMOKE_FAIL:-N/A}
- formal:
  - sync industry exit=${SYNC_EXIT:-N/A} log=${SYNC_LOG_PATH:-未找到}
    - summary: ${SYNC_SUMMARY:-N/A}
  - market fundamentals validate exit=${VALIDATE_EXIT:-N/A} log=${VALIDATE_LOG_PATH:-未找到}
    - summary: ${VALIDATE_SUMMARY:-N/A}
    - total_records: ${VALIDATE_TOTAL_RECORDS:-N/A}
    - unique_codes: ${VALIDATE_UNIQUE_CODES:-N/A}
    - snapshot_min: ${VALIDATE_SNAPSHOT_MIN:-N/A}
    - snapshot_max: ${VALIDATE_SNAPSHOT_MAX:-N/A}
    - market_cap_coverage: ${VALIDATE_MARKET_CAP_COVERAGE:-N/A}
    - latest_report_profit_coverage: ${VALIDATE_PROFIT_COVERAGE:-N/A}
    - warning_count: ${VALIDATE_WARNING_COUNT:-N/A}
  - market foundation exit=${FOUNDATION_EXIT:-N/A} log=${FOUNDATION_LOG_PATH:-未找到}
    - summary: ${FOUNDATION_SUMMARY:-N/A}
    - total_stocks: ${FOUNDATION_TOTAL:-N/A}
    - classified_stocks: ${FOUNDATION_CLASSIFIED:-N/A}
    - unclassified_stocks: ${FOUNDATION_UNCLASSIFIED:-N/A}
    - sector_count: ${FOUNDATION_SECTOR_COUNT:-N/A}
    - top_sector: ${FOUNDATION_TOP_SECTOR:-N/A}
  - market strength exit=${STRENGTH_EXIT:-N/A} log=${STRENGTH_LOG_PATH:-未找到}
    - summary: ${STRENGTH_SUMMARY:-N/A}
    - base: ${STRENGTH_BASE:-N/A}
    - candidate_stock_count: ${STRENGTH_CANDIDATES:-N/A}
    - snapshot_source: ${STRENGTH_SNAPSHOT_SOURCE:-N/A}
    - tdx_coverage: ${STRENGTH_TDX_COVERAGE:-N/A}
    - top_strong_sector: ${STRENGTH_TOP_STRONG:-N/A}
    - top_weak_sector: ${STRENGTH_TOP_WEAK:-N/A}
    - top_market_cap_stock: ${STRENGTH_TOP_CAP:-N/A}
    - top_profit_stock: ${STRENGTH_TOP_PROFIT:-N/A}
  - market strength-stocks exit=${STRENGTH_STOCKS_EXIT:-N/A} log=${STRENGTH_STOCKS_LOG_PATH:-未找到}
    - summary: ${STRENGTH_STOCKS_SUMMARY:-N/A}
    - sector_filter: ${STRENGTH_STOCKS_SECTOR:-N/A}
    - metric: ${STRENGTH_STOCKS_METRIC:-N/A}
    - coverage: ${STRENGTH_STOCKS_COVERAGE:-N/A}
    - top_row: ${STRENGTH_STOCKS_TOP_ROW:-N/A}

## 当前判定

- 如果 precheck 或 smoke 的 FAIL 大于 0，应先修复 CLI/脚本问题后再继续。
- 如果只有 WARN，请优先查看 precheck 日志中的 \`[REMEDIATION]\` 段落。
- 如果 \`fundamentals_state\` 为 \`missing\` 或 \`empty\`，即使 TDX 快照链路可用，\`market strength\` / \`market strength-stocks\` 的总市值、净利润 TopN 仍可能输出为空。
- 当 \`fundamentals_state\` 为 \`missing\` 或 \`empty\` 时，建议先运行 \`${REHEARSAL_CMD}\` 做 scratch ClickHouse 导入演练；确认 JSON 与导入链路正常后，先执行 \`quantix data validate-fundamentals --input /abs/path/market_fundamentals.json\`，再执行 \`quantix data import-fundamentals --input /abs/path/market_fundamentals.json\`。
- 当 warning 收敛到可接受范围后，再执行正式命令链路：
  - \`quantix risk sync industry --standard shenwan\`
  - \`quantix market foundation\`
  - \`quantix data validate-fundamentals --input /abs/path/market_fundamentals.json\`
  - \`quantix data import-fundamentals --input /abs/path/market_fundamentals.json\`
  - \`${STRENGTH_CMD}\`
  - \`${STRENGTH_STOCKS_CMD}\`

## 建议补充记录

- 环境模板是否已加载：\`source scripts/dev/market_cli_env.example.sh\`
- precheck 主要 warning:
- smoke 主要 warning:
- 正式命令执行结果:
  - sync industry:
  - market foundation:
  - market strength:
  - market strength-stocks:
- 结论:
EOF

echo "Generated report: $REPORT_PATH"
