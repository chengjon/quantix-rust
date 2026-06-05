#!/usr/bin/env bash
# 每日数据更新脚本
# 用法: ./scripts/daily-update.sh [--all]
# 添加到 crontab: 0 18 * * 1-5 /opt/claude/quantix-rust/scripts/daily-update.sh --all

set -euo pipefail

YEAR="${1:-$(date +%Y)}"
QUANTIX="${QUANTIX:-quantix}"

echo "=== 每日数据更新 $(date '+%Y-%m-%d %H:%M:%S') ==="

# 1. 同步交易日历
echo "[1/2] 同步 ${YEAR} 年交易日历..."
$QUANTIX data tdx-api sync-calendar --year "$YEAR" 2>&1 || echo "  日历同步失败，继续..."

# 2. 增量导入 K 线
echo "[2/2] 增量导入日线..."
$QUANTIX data tdx-api import-klines --all --type day 2>&1 || echo "  日线导入失败"

echo "=== 更新完成 $(date '+%Y-%m-%d %H:%M:%S') ==="
