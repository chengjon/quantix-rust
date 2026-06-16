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
