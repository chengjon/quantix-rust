#!/usr/bin/env bash
# Daily market data update via OpenStock (P0.12 rewrite).
#
# Replaces the legacy tdx-api based daily-update.sh removed in P0.11c.
# OpenStock is the canonical market data source; this script:
#   1. Fetches the trading calendar for the current year (read-only)
#   2. Performs kline imports for the symbols passed as args
#      (dry-run by default; pass APPLY=1 to write to ClickHouse)
#
# Usage:
#   scripts/daily-update.sh [CODE1 CODE2 ...]
#   APPLY=1 scripts/daily-update.sh 600000 600001   # write to ClickHouse
#   YEAR=2026 scripts/daily-update.sh               # override year
#
# Environment:
#   QUANTIX                quantix binary (default: `quantix`)
#   YEAR                   calendar year (default: current year)
#   OPENSTOCK_BASE_URL     required for live OpenStock (e.g. http://192.168.123.104:8040)
#   OPENSTOCK_API_KEY      required for live OpenStock
#   APPLY                  set to `1` to pass --apply (requires QUANTIX_OPENSTOCK_KLINE_APPLY=yes)
#
# Exit codes:
#   0 on success (individual fetch failures are reported but non-fatal)
#   1 if OpenStock env is missing or quantix binary not found

set -euo pipefail

YEAR="${YEAR:-$(date +%Y)}"
QUANTIX="${QUANTIX:-quantix}"
APPLY_FLAG=""
if [[ "${APPLY:-0}" == "1" ]]; then
  APPLY_FLAG="--apply"
fi

if ! command -v "$QUANTIX" >/dev/null 2>&1; then
  echo "ERROR: quantix binary not found (QUANTIX=$QUANTIX)" >&2
  exit 1
fi

if [[ -z "${OPENSTOCK_BASE_URL:-}" || -z "${OPENSTOCK_API_KEY:-}" ]]; then
  echo "ERROR: OPENSTOCK_BASE_URL and OPENSTOCK_API_KEY must be set for live OpenStock access" >&2
  exit 1
fi

CODES=("$@")
if [[ ${#CODES[@]} -eq 0 ]]; then
  echo "WARN: no codes passed; defaulting to 600000 (pass codes as args for multi-symbol update)" >&2
  CODES=(600000)
fi

echo "=== OpenStock daily update $(date '+%Y-%m-%d %H:%M:%S') ==="
echo "Endpoint: $OPENSTOCK_BASE_URL"
echo "Year:     $YEAR"
echo "Codes:    ${CODES[*]} (${#CODES[@]} symbols)"
echo "Apply:    ${APPLY_FLAG:-dry-run}"
echo

echo "[1/2] Fetching ${YEAR} trading calendar (read-only)..."
$QUANTIX data openstock fetch-calendar --year "$YEAR" 2>&1 || echo "  Calendar fetch failed, continuing..."

echo
echo "[2/2] Importing daily klines (${#CODES[@]} symbols)..."
for code in "${CODES[@]}"; do
  echo "  -> $code"
  $QUANTIX data import-klines \
    --code "$code" \
    --type day \
    --start "${YEAR}-01-01" \
    --end "${YEAR}-12-31" \
    $APPLY_FLAG 2>&1 || echo "    import failed for $code, continuing..."
done

echo
echo "=== Update complete $(date '+%Y-%m-%d %H:%M:%S') ==="
