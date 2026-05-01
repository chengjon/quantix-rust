#!/bin/bash
# Preview or clean stale local OMX state under the current repository.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STATE_DIR="${PROJECT_ROOT}/.omx/state"
MODE="dry-run"

usage() {
    cat <<'EOF'
Usage: scripts/dev/clean_omx_state.sh [--apply] [--state-dir PATH]

Options:
  --apply           Persist cleanup changes. Default is dry-run.
  --state-dir PATH  Override the OMX state directory. Default: .omx/state under repo root.
  -h, --help        Show this help.

Behavior:
  - Finds local OMX JSON state files with active=true or non-terminal phases.
  - In dry-run mode, prints the files that would be normalized.
  - In apply mode, rewrites them to inactive/cancelled and clears active_skills.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --apply)
            MODE="apply"
            shift
            ;;
        --state-dir)
            STATE_DIR="${2:?missing value for --state-dir}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ ! -d "${STATE_DIR}" ]]; then
    echo "State directory not found: ${STATE_DIR}" >&2
    exit 1
fi

python - "$STATE_DIR" "$MODE" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

state_dir = Path(sys.argv[1])
mode = sys.argv[2]
terminal = {"", "cancelled", "complete", "completed", "failed"}
phase_keys = ("current_phase", "phase")
changed = []

def normalize(path: Path, payload: dict, now_iso: str) -> tuple[dict, bool]:
    active = payload.get("active") is True
    phase_key = next((key for key in phase_keys if key in payload), None)
    phase_value = str(payload.get(phase_key, "")).strip().lower() if phase_key else ""
    needs_cleanup = active or phase_value not in terminal
    if not needs_cleanup:
        return payload, False

    updated = dict(payload)
    updated["active"] = False
    if phase_key:
        updated[phase_key] = "cancelled"
        if phase_key == "current_phase":
            updated["completed_at"] = now_iso
    if "active_skills" in updated:
        updated["active_skills"] = []
    if "updated_at" in updated:
        updated["updated_at"] = now_iso
    return updated, True

now_iso = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")

for path in sorted(state_dir.rglob("*.json")):
    try:
        payload = json.loads(path.read_text())
    except Exception:
        continue
    if not isinstance(payload, dict):
        continue
    updated, dirty = normalize(path, payload, now_iso)
    if not dirty:
        continue
    changed.append(path)
    if mode == "apply":
        path.write_text(json.dumps(updated, indent=2) + "\n")

if not changed:
    print(f"[{mode}] no stale OMX state found under {state_dir}")
    raise SystemExit(0)

print(f"[{mode}] stale OMX state files under {state_dir}:")
for path in changed:
    print(path)
PY
