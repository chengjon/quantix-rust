---
status: superseded
trigger: "Investigate issue: graphiti-mcp-review-stuck"
created: 2025-03-23T00:00:00Z
updated: 2026-03-24T18:10:32Z
---

## Current Focus
superseded: 2026-03-24 benchmark against NAS `graphiti-mcp` succeeded. Observed `processing` at T+20.0s and `completed` at T+40.1s, so this note should not be treated as the current root-cause conclusion.

historical_hypothesis: ROOT CAUSE - Commit af9e809 (2026-03-22) added global rate limit cooldown. When any episode hits a rate limit error (429), _global_rate_limit_cooldown_until is set (15s base, doubles to 180s max). ALL subsequent episodes wait for this cooldown before processing, appearing "stuck" in "processing" state. The "Graphiti backfill required" message likely comes from a UI layer misinterpreting the long wait time.
test: Check MCP server logs for rate limit errors and "Global rate-limit cooldown active" warnings
expecting: Find rate limit errors (429) or cooldown warnings in logs
next_action: Ask user to check MCP server logs for rate limit errors

## Resolution
root_cause: Recent commit af9e809 added queue-wide rate limit cooldown that causes ALL episodes to wait when any episode hits a rate limit. Default cooldown starts at 15s and can escalate to 180s (3 minutes), making episodes appear "stuck" in "processing" state.

fix: Multiple options:
1. Reduce cooldown times via env vars (GRAPHITI_RATE_LIMIT_COOLDOWN_BASE_SECONDS, GRAPHITI_RATE_LIMIT_COOLDOWN_MAX_SECONDS)
2. Fix upstream LLM/embedder API rate limit issues
3. Revert commit af9e809 if behavior is too aggressive
4. Add visibility to UI showing "waiting for rate limit cooldown" instead of "processing"

verification: Need user to:
1. Check MCP server logs for rate limit errors
2. Verify if cooldown is actually triggering (look for "Global rate-limit cooldown active" messages)
3. Test with reduced cooldown times or investigate upstream API issues

files_changed: []

## Symptoms
expected: Memory should be saved successfully to the knowledge graph and become searchable
actual: Status stuck at "processing" with "Graphiti backfill required" message showing
errors: User has not checked logs yet - need to investigate
reproduction: Trigger a review memory write operation in the Graphiti MCP server
started: This used to work, now broken (regression)

## Eliminated

## Evidence
- timestamp: 2025-03-23T23:54:00Z
  checked: Searched entire Graphiti codebase for "backfill" and "Graphiti backfill required"
  found: NO matches for "backfill" or "backfill required" in any Graphiti source code
  implication: Error message originates from external system (UI layer, frontend, or different codebase)

- timestamp: 2025-03-23T23:54:00Z
  checked: Read MCP server queue_service.py implementation
  found: Queue service manages episode processing with states: queued -> processing -> completed/failed
  implication: "processing" state is set when worker starts processing episode (line 219 in queue_service.py)

- timestamp: 2025-03-23T23:54:00Z
  checked: Searched quantix-rust Rust source code for error message
  found: NO matches for "backfill", "Graphiti required", or "stuck" in /opt/claude/quantix-rust/src
  implication: Error message likely from a UI/ frontend layer not in these codebases

- timestamp: 2025-03-23T23:55:00Z
  checked: Recent git commits for regression cause
  found: Commit af9e809 on 2026-03-22 "add queue-wide rate limit cooldown for mcp ingest"
  implication: REGRESSION IDENTIFIED - Recent changes added global rate limit cooldown that could cause episodes to get stuck in "processing" state

- timestamp: 2025-03-23T23:55:00Z
  checked: Commit af9e809 changes to queue_service.py
  found: Added _global_rate_limit_cooldown_until, _wait_for_global_rate_limit_cooldown() method
  implication: If global cooldown is triggered, ALL episodes wait until cooldown expires, causing apparent "stuck" behavior
