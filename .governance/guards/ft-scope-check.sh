#!/usr/bin/env bash
set -euo pipefail

FT_GOVERNANCE_SCRIPT='/root/.codex/skills/myskills/skills/function-tree/scripts/ft-governance.cjs'
export FT_GOVERNANCE_SCRIPT
exec node "$FT_GOVERNANCE_SCRIPT" scope-check --root "$(git rev-parse --show-toplevel 2>/dev/null || pwd)" "$@"
