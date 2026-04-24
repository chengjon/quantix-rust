#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

EXAMPLE_PATH="$ROOT_DIR/.env.market.local.example"
LOCAL_PATH="$ROOT_DIR/.env.market.local"

if [[ ! -f "$EXAMPLE_PATH" ]]; then
  echo "[FAIL] missing example file: $EXAMPLE_PATH" >&2
  exit 1
fi

if [[ ! -f "$LOCAL_PATH" ]]; then
  cp "$EXAMPLE_PATH" "$LOCAL_PATH"
  echo "[INFO] created $LOCAL_PATH from $EXAMPLE_PATH"
else
  echo "[INFO] local env already exists: $LOCAL_PATH"
fi

if rg -n 'replace-me' "$LOCAL_PATH" >/dev/null 2>&1; then
  echo "[WARN] placeholder values still present in $LOCAL_PATH" >&2
  echo "[NEXT] edit $LOCAL_PATH and replace all 'replace-me' values before rerunning market acceptance" >&2
  exit 2
fi

echo "[PASS] local market env ready: $LOCAL_PATH"
