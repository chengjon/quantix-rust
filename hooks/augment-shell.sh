#!/bin/bash
# GitNexus beforeShellExecution hook for Cursor
# Receives JSON on stdin with { command, cwd, timeout }
# Returns JSON on stdout with { permission, agent_message }
#
# Extracts search pattern from grep/rg commands, runs gitnexus augment,
# and injects the enriched context via agent_message.

INPUT=$(cat)

COMMAND=$(echo "$INPUT" | jq -r '.command // empty' 2>/dev/null)
CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)

if [ -z "$COMMAND" ]; then
  echo '{"permission":"allow"}'
  exit 0
fi

# Skip non-search commands
case "$COMMAND" in
  cd\ *|npm\ *|yarn\ *|pnpm\ *|git\ commit*|git\ push*|git\ pull*|mkdir\ *|rm\ *|cp\ *|mv\ *|echo\ *|cat\ *)
    echo '{"permission":"allow"}'
    exit 0
    ;;
esac

# Extract search pattern from rg/grep commands
PATTERN=""
if echo "$COMMAND" | grep -qE '\brg\b'; then
  PATTERN=$(echo "$COMMAND" | sed -n "s/.*\brg\s\+\(--[^ ]*\s\+\)*['\"]\\?\([^'\";\| >]*\\).*/\2/p")
elif echo "$COMMAND" | grep -qE '\bgrep\b'; then
  PATTERN=$(echo "$COMMAND" | sed -n "s/.*\bgrep\s\+\(-[^ ]*\s\+\)*['\"]\\?\([^'\";\| >]*\\).*/\2/p")
fi

if [ -z "$PATTERN" ] || [ ${#PATTERN} -lt 3 ]; then
  echo '{"permission":"allow"}'
  exit 0
fi

# Infer repo name from cwd (gitnexus multi-repo requires explicit --repo)
REPO_NAME=""
if [ -n "$CWD" ]; then
  REPO_NAME=$(basename "$CWD")
fi

# Run gitnexus query (augment may be intentionally silent in newer versions)
GITNEXUS_BIN=${GITNEXUS_BIN:-/root/.nvm/versions/node/v24.7.0/bin/gitnexus}
if [ ! -x "$GITNEXUS_BIN" ]; then
  GITNEXUS_BIN=gitnexus
fi

GITNEXUS_HOME=${GITNEXUS_HOME:-/opt/claude/quantix-rust/.cursor-gitnexus}
mkdir -p "$GITNEXUS_HOME" 2>/dev/null || true

if [ -n "$REPO_NAME" ]; then
  RESULT=$(HOME="$GITNEXUS_HOME" $GITNEXUS_BIN query --repo "$REPO_NAME" "$PATTERN" 2>&1)
else
  RESULT=$(HOME="$GITNEXUS_HOME" $GITNEXUS_BIN query "$PATTERN" 2>&1)
fi
QUERY_EXIT=$?

HAS_MEANINGFUL_RESULT=0
if [ $QUERY_EXIT -eq 0 ] && [ -n "$RESULT" ]; then
  # Suppress noisy empty payloads like:
  # {"processes":[],"process_symbols":[],"definitions":[]}
  if echo "$RESULT" | jq -e 'type == "object" and (((.processes // []) | length) > 0 or ((.process_symbols // []) | length) > 0 or ((.definitions // []) | length) > 0)' >/dev/null 2>&1; then
    HAS_MEANINGFUL_RESULT=1
  fi
fi

if [ $HAS_MEANINGFUL_RESULT -eq 1 ]; then
  # Escape for JSON
  ESCAPED=$(echo "$RESULT" | jq -Rs .)
  echo "{\"permission\":\"allow\",\"agent_message\":$ESCAPED}"
else
  echo '{"permission":"allow"}'
fi

exit 0
