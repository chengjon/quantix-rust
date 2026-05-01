#!/usr/bin/env bash
# cb.sh — cargo build wrapper with target size guard
#
# Usage:
#   ./scripts/dev/cb.sh build        # cargo build + size check
#   ./scripts/dev/cb.sh test         # cargo test + size check
#   ./scripts/dev/cb.sh test --test xxx  # forward extra args
#
# Or add to your shell:
#   alias cb='/opt/claude/quantix-rust/scripts/dev/cb.sh'
#   alias ct='/opt/claude/quantix-rust/scripts/dev/cb.sh test'

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Run the actual cargo command
cargo "$@"
EXIT_CODE=$?

# Check target size regardless of build result
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
if [ -d "$PROJECT_ROOT/target" ]; then
    "$SCRIPT_DIR/guard_target_size.sh" --status
fi

exit $EXIT_CODE
