#!/usr/bin/env bash
# guard_target_size.sh — Monitor and optionally clean target/ directory bloat
#
# Usage:
#   ./guard_target_size.sh              # Check only, exit 1 if over threshold
#   ./guard_target_size.sh --clean      # Auto-clean if over threshold
#   ./guard_target_size.sh --status     # Always show stats, exit 0
#
# Thresholds (configurable via env):
#   TARGET_WARN_GB   — warn threshold in GB (default: 8)
#   TARGET_MAX_GB    — force clean threshold in GB (default: 15)
#   TARGET_BASE      — path to project root (default: script's grandparent)

set -euo pipefail

# --- Config ---
WARN_GB="${TARGET_WARN_GB:-8}"
MAX_GB="${TARGET_MAX_GB:-15}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BASE_DIR="${TARGET_BASE:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
TARGET_DIR="$BASE_DIR/target"

# --- Helpers ---
gb() { echo "$1" | awk '{printf "%.1f", $1 / 1024 / 1024}'; }

size_kb() {
    if [ ! -d "$1" ]; then echo 0; return; fi
    du -sk "$1" 2>/dev/null | cut -f1
}

# --- Main ---
ACTION="${1:---check}"
CURRENT_KB=$(size_kb "$TARGET_DIR")
CURRENT_GB=$(gb "$CURRENT_KB")

if [ "$ACTION" = "--status" ]; then
    echo "target/: ${CURRENT_GB} GB  (warn: ${WARN_GB} GB, max: ${MAX_GB} GB)"

    if [ -d "$TARGET_DIR/debug/deps" ]; then
        DEPS_KB=$(size_kb "$TARGET_DIR/debug/deps")
        DEPS_GB=$(gb "$DEPS_KB")
        echo "  deps/: ${DEPS_GB} GB"

        # Show top 5 largest files
        echo "  top 5 files:"
        top5=$(find "$TARGET_DIR/debug/deps" -maxdepth 1 -type f -exec du -k {} + 2>/dev/null | sort -rn | head -5 || true)
        while IFS= read -r line; do
            [ -z "$line" ] && continue
            kb=$(echo "$line" | awk '{print $1}')
            file=$(echo "$line" | awk '{print $2}')
            printf "    %-60s %5.0f MB\n" "$(basename "$file")" "$((kb / 1024))"
        done <<< "$top5"
    fi

    # Check for stale artifacts older than 7 days
    if [ -d "$TARGET_DIR" ]; then
        STALE=$(find "$TARGET_DIR/debug/deps" -maxdepth 1 -type f -mtime +7 2>/dev/null | wc -l)
        if [ "$STALE" -gt 0 ]; then
            echo "  stale files (>7 days): $STALE"
        fi
    fi
    exit 0
fi

# --- Threshold check ---
STATUS="OK"
if (( $(echo "$CURRENT_GB >= $MAX_GB" | bc -l) )); then
    STATUS="CRITICAL"
elif (( $(echo "$CURRENT_GB >= $WARN_GB" | bc -l) )); then
    STATUS="WARN"
fi

echo "[${STATUS}] target/: ${CURRENT_GB} GB (warn: ${WARN_GB}, max: ${MAX_GB})"

if [ "$STATUS" = "OK" ]; then
    exit 0
fi

# --- Action ---
if [ "$ACTION" = "--clean" ] && [ "$STATUS" != "OK" ]; then
    echo "Cleaning target/ ..."
    cargo clean --manifest-path "$BASE_DIR/Cargo.toml" 2>&1
    AFTER_KB=$(size_kb "$TARGET_DIR")
    AFTER_GB=$(gb "$AFTER_KB")
    echo "Cleaned: ${CURRENT_GB} GB -> ${AFTER_GB} GB"
    exit 0
fi

# --check mode: just report and exit
if [ "$STATUS" = "CRITICAL" ]; then
    echo "Run with --clean to free space, or manually: cargo clean"
    exit 1
fi

echo "Consider running: cargo clean"
exit 1
