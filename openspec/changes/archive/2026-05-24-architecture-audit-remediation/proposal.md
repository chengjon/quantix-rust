## Why

The architecture audit is reviewed and baselined, but its development work is still spread across GitHub issues and report tables. The remaining work touches dependency direction, shared domain types, CLI boundaries, storage adapters, safety cleanup, and large-file splits. Those changes need a single spec-driven control point so each slice is designed, tested, validated, and archived without drifting into cosmetic cleanup.

## What Changes

- Establish OpenSpec as the governing workflow for the reviewed architecture audit remediation.
- Treat issue #63 as the completed correction baseline and issues #64-#72 as the active implementation queue.
- Require characterization tests before architecture seam changes.
- Require GitNexus impact and detect_changes gates for code refactors.
- Sequence large-file splits after dependency seams are stable and tested.
- Track safety/hygiene cleanup as behavior-risk remediation, not pattern-count cleanup.

## Capabilities

### New Capabilities

- `architecture-remediation`: governs the architecture audit remediation workflow, requirements, task order, and validation gates for issues #64-#72.

### Modified Capabilities

- None. This repository had no existing OpenSpec specs before this change.

## Impact

- Adds OpenSpec project configuration under `openspec/config.yaml`.
- Adds the active OpenSpec change under `openspec/changes/architecture-audit-remediation/`.
- Does not change runtime code by itself.
- Future code changes under this change are expected to touch `core`, `strategy`, `execution`, `risk`, `market`, `db`, `cli`, `monitoring`, `tasks`, `io`, and focused tests as tasks are executed.
