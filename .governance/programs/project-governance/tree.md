# project-governance Governance Tree

> Function tree ref: `project/root`
> Description: Project governance entrypoint
> Created at: 2026-05-25T00:21:33.252Z
> Current head: `f242316473c31d1f726adb3efba1da927c639171`

## Status Legend

`planning -> evidence-prepared -> decision-prepared -> authorization-prepared -> approved-for-implementation -> implementation-ready -> implementation-landed -> closeout-prepared -> closed`

`blocked` can interrupt any active state when `blocker_reason` and `unblock_target_state` are recorded.

## Tree

- [ ] Root program: `project-governance`

## Evidence Ledger

| Node | Evidence | Current HEAD | Notes |
|------|----------|--------------|-------|

## Active Gates

Generated summary lives in `.governance/active-gates.md`.
- [ ] Q1.1: Adopt function-tree governance entrypoint (planning, FT: project/root/function-tree-bootstrap)
- [ ] Q1.2: Connect function-tree guard to project-local agent hooks (planning, FT: project/root/function-tree-hooks)
- [ ] Q1.3: Add Quantix project-local function-tree profile (planning, FT: project/root/function-tree-profile)
- [ ] Q1.4: Add function-tree usage entrypoint for future agents (planning, FT: project/root/function-tree-usage)
- [ ] Q1.5: Add project-local /ft slash command entrypoints (planning, FT: project/root/function-tree-slash-commands)
- [ ] Q1.6: Regenerate FUNCTION_TREE with functional tree template (planning, FT: project/root/function-tree-functional-template)
- [ ] Q1.7: Clarify FUNCTION_TREE direction guidance (planning, FT: project/root/function-tree-direction-guidance)
- [ ] P0.1: Paper order query/cancel runnable closure (planning, FT: Paper query/cancel)
- [ ] P0.2: Execution mode semantics hardening (planning, FT: Execution mode semantics)
- [ ] P0.2b: Paper immediate-fill behavior lock (planning, FT: execution/paper-immediate)
- [ ] P0.2c: Execution mode risk notice catalog (planning, FT: execution/mode-semantics)
- [ ] P0.2d: Execution storage namespace semantics (planning, FT: FUNCTION_TREE.md)
- [ ] P0.2e: Execution mode config namespace semantics (planning, FT: FUNCTION_TREE.md)
- [ ] P0.2f: Execution mode semantics closeout report (planning, FT: FUNCTION_TREE.md)
- [ ] P0.3a: qmt_live capability identity hardening design (planning, FT: execution)
- [ ] P0.3b: qmt_live capability snapshot seed (planning, FT: execution)
- [ ] P0.3c: qmt_live identity reconciliation tightening (planning, FT: execution)
- [ ] P0.3d: qmt_live error taxonomy seed (planning, FT: execution)
- [ ] P0.3e: ExecutionCapabilities MVP (planning, FT: FUNCTION_TREE.md)
- [ ] P0.3f: ExecutionCapabilities read-only observability (planning, FT: FUNCTION_TREE.md)
- [ ] P0.4a: qmt_live hardening design (planning, FT: FUNCTION_TREE.md)
- [ ] P0.4b: qmt_live capability descriptor (planning, FT: qmt_live capability / identity hardening)
- [ ] P0.4c: qmt_live error taxonomy local enrichment (planning, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [ ] P0.4d: qmt_live gate runtime compatibility check (planning, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [ ] P0.4e: qmt_live diagnostics wiring (planning, FT: qmt_live capability / identity hardening)
- [ ] P0.4f: qmt_live identity and runtime metadata recovery (planning, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [ ] P0.4g: qmt_live reconciliation query refinement (planning, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [ ] auction-collector-offline-test-seam: auction collector offline test seam (planning, FT: src/sources/auction_collector.rs)
- [ ] P0.5a: qmt_live preflight doctor (planning, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [ ] P0.5b: qmt_live canary runbook and evidence artifact (planning, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [ ] P0.5b-backfill: P0.5b Graphiti backfill record (planning, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [ ] P0.5c: qmt_live kill switch acceptance (planning, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [ ] P0.5c-backfill: P0.5c Graphiti backfill record (planning, FT: docs/reports/QMT_LIVE_KILL_SWITCH_ACCEPTANCE_P0_5C_GRAPHITI_BACKFILL_2026-06-23.md)
- [ ] P0.5d: qmt_live audit evidence closure (planning, FT: openspec/changes/qmt-live-operational-safety-p0-5/tasks.md)
- [ ] P0.5e: qmt_live manual intervention report (planning, FT: openspec/changes/qmt-live-operational-safety-p0-5/tasks.md)
- [ ] P0.5f: qmt_live release closure docs (planning, FT: FUNCTION_TREE.md#qmt_live-operational-safety-p0-5)
- [ ] P0.5g: qmt_live OpenSpec archive (planning, FT: openspec/changes/qmt-live-operational-safety-p0-5)
- [ ] P0.6: qmt_live runtime readiness (planning, FT: P0.5)
- [ ] P0.6a: qmt_live environment inventory and prerequisite check (planning, FT: P0.6)
- [ ] P0.6b: qmt_live read-only command smoke (planning, FT: P0.6)
- [ ] P0.6b-backfill: P0.6b Graphiti backfill record (planning, FT: P0.6)
- [ ] P0.6c: qmt_live runtime readiness evidence package (planning, FT: openspec/changes/qmt-live-runtime-readiness-p0-6/tasks.md#3-p06c-redacted-runtime-evidence-package)
- [ ] P0.6c-backfill: qmt_live runtime readiness Graphiti backfill (planning, FT: docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6C_2026-06-24.md)
