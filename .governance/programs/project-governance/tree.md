# project-governance Governance Tree

> Function tree ref: `project/root`
> Description: Project governance entrypoint
> Created at: 2026-05-25T00:21:33.252Z
> Current head: `f242316473c31d1f726adb3efba1da927c639171`

## Status Legend

`planning -> evidence-prepared -> decision-prepared -> authorization-prepared -> approved-for-implementation -> implementation-ready -> implementation-landed -> closeout-prepared -> closed`

`blocked` can interrupt any active state when `blocker_reason` and `unblock_target_state` are recorded.

## Tree

- [x] Q1.1: Adopt function-tree governance entrypoint [external] (closed, FT: project/root/function-tree-bootstrap)
- [x] Q1.2: Connect function-tree guard to project-local agent hooks [external] (closed, FT: project/root/function-tree-hooks)
- [x] Q1.3: Add Quantix project-local function-tree profile [external] (closed, FT: project/root/function-tree-profile)
- [x] Q1.4: Add function-tree usage entrypoint for future agents [external] (closed, FT: project/root/function-tree-usage)
- [x] Q1.5: Add project-local /ft slash command entrypoints [external] (closed, FT: project/root/function-tree-slash-commands)
- [x] Q1.6: Regenerate FUNCTION_TREE with functional tree template [external] (closed, FT: project/root/function-tree-functional-template)
- [x] Q1.7: Clarify FUNCTION_TREE direction guidance [external] (closed, FT: project/root/function-tree-direction-guidance)
- [x] P0.1: Paper order query/cancel runnable closure [implementation] (closed, FT: Paper query/cancel)
- [x] P0.2: Execution mode semantics hardening [implementation] (closed, FT: Execution mode semantics)
- [x] P0.2b: Paper immediate-fill behavior lock [decision] (closed, FT: execution/paper-immediate)
- [x] P0.2c: Execution mode risk notice catalog [decision] (closed, FT: execution/mode-semantics)
- [x] P0.2d: Execution storage namespace semantics [implementation] (closed, FT: FUNCTION_TREE.md)
- [x] P0.2e: Execution mode config namespace semantics [decision] (closed, FT: FUNCTION_TREE.md)
- [x] P0.2f: Execution mode semantics closeout report [decision] (closed, FT: FUNCTION_TREE.md)
- [x] P0.3a: qmt_live capability identity hardening design [decision] (closed, FT: execution)
- [x] P0.3b: qmt_live capability snapshot seed [decision] (closed, FT: execution)
- [x] P0.3c: qmt_live identity reconciliation tightening [decision] (closed, FT: execution)
- [x] P0.3d: qmt_live error taxonomy seed [decision] (closed, FT: execution)
- [x] P0.3e: ExecutionCapabilities MVP [decision] (closed, FT: FUNCTION_TREE.md)
- [x] P0.3f: ExecutionCapabilities read-only observability [decision] (closed, FT: FUNCTION_TREE.md)
- [x] P0.4a: qmt_live hardening design [decision] (closed, FT: FUNCTION_TREE.md)
- [x] P0.4b: qmt_live capability descriptor [decision] (closed, FT: qmt_live capability / identity hardening)
- [x] P0.4c: qmt_live error taxonomy local enrichment [implementation] (closed, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [x] P0.4d: qmt_live gate runtime compatibility check [implementation] (closed, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [x] P0.4e: qmt_live diagnostics wiring [implementation] (closed, FT: qmt_live capability / identity hardening)
- [x] P0.4f: qmt_live identity and runtime metadata recovery [implementation] (closed, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [x] P0.4g: qmt_live reconciliation query refinement [implementation] (closed, FT: docs/reports/QMT_LIVE_HARDENING_DESIGN_P0_4A_2026-06-21.md)
- [x] auction-collector-offline-test-seam: auction collector offline test seam [implementation] (closed, FT: src/sources/auction_collector.rs)
- [x] P0.5a: qmt_live preflight doctor [implementation] (closed, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [x] P0.5b: qmt_live canary runbook and evidence artifact [implementation] (closed, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [x] P0.5b-backfill: P0.5b Graphiti backfill record [evidence] (closed, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [x] P0.5c: qmt_live kill switch acceptance [implementation] (closed, FT: FUNCTION_TREE.md#qmt_live-operational-safety-P0.5)
- [x] P0.5c-backfill: P0.5c Graphiti backfill record [evidence] (closed, FT: docs/reports/QMT_LIVE_KILL_SWITCH_ACCEPTANCE_P0_5C_GRAPHITI_BACKFILL_2026-06-23.md)
- [x] P0.5d: qmt_live audit evidence closure [evidence] (closed, FT: openspec/changes/qmt-live-operational-safety-p0-5/tasks.md)
- [x] P0.5e: qmt_live manual intervention report [evidence] (closed, FT: openspec/changes/qmt-live-operational-safety-p0-5/tasks.md)
- [x] P0.5f: qmt_live release closure docs [closeout] (closed, FT: FUNCTION_TREE.md#qmt_live-operational-safety-p0-5)
- [x] P0.5g: qmt_live OpenSpec archive [closeout] (closed, FT: openspec/changes/qmt-live-operational-safety-p0-5)
- [x] P0.6: qmt_live runtime readiness [external] (closed, FT: P0.5)
- [x] P0.6a: qmt_live environment inventory and prerequisite check [external] (closed, FT: P0.6)
- [x] P0.6b: qmt_live read-only command smoke [external] (closed, FT: P0.6)
- [x] P0.6b-backfill: P0.6b Graphiti backfill record [evidence] (closed, FT: P0.6)
- [x] P0.6c: qmt_live runtime readiness evidence package [decision] (closed, FT: openspec/changes/archive/2026-06-24-qmt-live-runtime-readiness-p0-6/tasks.md#3-p06c-redacted-runtime-evidence-package)
- [x] P0.6c-backfill: qmt_live runtime readiness Graphiti backfill [decision] (closed, FT: docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6C_2026-06-24.md)
- [x] P0.6d: qmt_live runtime readiness failure boundary drill [decision] (closed, FT: openspec/changes/archive/2026-06-24-qmt-live-runtime-readiness-p0-6/tasks.md#4-p06d-failure-boundary-drill)
- [x] P0.6d-backfill: qmt_live failure boundary Graphiti backfill [decision] (closed, FT: docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6D_2026-06-24.md)
- [x] P0.6e: qmt_live runtime readiness decision report [decision] (closed, FT: openspec/changes/archive/2026-06-24-qmt-live-runtime-readiness-p0-6/tasks.md#5-p06e-readiness-decision-report)
- [x] P0.7a: ExecutionCapabilities mode semantics bridge [decision] (closed, FT: ExecutionCapabilities continuation)
- [x] P0.7a-backfill: P0.7a Graphiti backfill record [decision] (closed, FT: docs/reports/EXECUTION_CAPABILITIES_MODE_SEMANTICS_P0_7A_2026-06-25.md)
- [x] P0.7b: ExecutionCapabilities checklist mode semantics [decision] (closed, FT: docs/reports/EXECUTION_CAPABILITIES_MODE_SEMANTICS_P0_7A_2026-06-25.md)
- [x] P0.7b-backfill: P0.7b Graphiti backfill record [decision] (closed, FT: docs/reports/EXECUTION_CAPABILITIES_CHECKLIST_MODE_SEMANTICS_P0_7B_2026-06-25.md)
- [x] P0.7c: ExecutionCapabilities preflight mode semantics [decision] (closed, FT: docs/reports/EXECUTION_CAPABILITIES_CHECKLIST_MODE_SEMANTICS_P0_7B_2026-06-25.md)
- [x] P0.7d: ExecutionCapabilities P0.7 documentation sync [decision] (closed, FT: FUNCTION_TREE.md#execution-mode-semantics-hardening)
- [x] P0.7d-backfill: P0.7d Graphiti backfill record [decision] (closed, FT: docs/reports/EXECUTION_CAPABILITIES_P0_7_DOC_SYNC_2026-06-26.md)
- [x] P0.8: OpenStock data consumption OpenSpec [decision] (closed, FT: openspec/changes/openstock-data-consumption-p0-8)
- [x] P0.8-backfill: OpenStock P0.8 Graphiti backfill [decision] (closed, FT: sources)
- [x] P0.8a: OpenStock data consumption inventory [decision] (closed, FT: sources)
- [x] P0.8a-backfill: OpenStock P0.8a Graphiti backfill [decision] (closed, FT: sources)
- [x] P0.8b: OpenStock daily kline fixture parser [decision] (closed, FT: sources)
- [x] P0.8b-backfill: OpenStock P0.8b Graphiti backfill [decision] (closed, FT: sources)
- [x] P0.8c: OpenStock local fixture validation CLI [decision] (closed, FT: sources)
- [x] P0.8c-backfill: OpenStock P0.8c Graphiti backfill [closeout] (closed, FT: sources/)
- [x] P0.8c-graphiti-completion-sync: OpenStock P0.8c Graphiti completion sync [closeout] (closed, FT: sources/)
- [x] P0.8d: OpenStock analysis fixture loop [task/decision] (closed, FT: sources/)
- [x] P0.8d-backfill: OpenStock P0.8d Graphiti backfill [closeout] (closed, FT: sources/)


## Evidence Ledger

| Node | Evidence | Current HEAD | Notes |
|------|----------|--------------|-------|
| Q1.1 | FUNCTION_TREE.md | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Initialized project-governance program, generated root FUNCTION_TREE.md, and preserved previous document content |
| Q1.2 | .governance/guards/ft-scope-check.sh | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Bootstrap node Q1.1 is closed; project-local Claude settings exist without function-tree PostToolUse hook; scope guard wrapper is installed |
| Q1.3 | AGENTS.md | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Quantix project rules require GitNexus for code structure and impact, Graphiti reads/writes with ingest verification, context-mode for broad scans, and FUNCTION_TREE.md as the sole feature status registry |
| Q1.4 | .governance/profile.md | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Project profile exists, but .governance/README.md usage entrypoint is missing |
| Q1.5 | .governance/README.md | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Function-tree README exists but project-local slash command files under .claude/commands/ft and .codex/commands/ft are missing |
| Q1.6 | FUNCTION_TREE.md | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Current generated section contains Project Snapshot / Skill Activation / Governance Programs / Operating Loop / State Files and preserved previous FUNCTION_TREE body; new generator should promote the functional tree body |
| Q1.7 | FUNCTION_TREE.md | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` | Registration rules now state current/future feature tree, existing plus planned/unfinished features, developer direction guide, and avoid drift. |
| P0.1 | fb0eb72 | `fb0eb72cdce59b9e83032d24576e093820e9e950` | Implementation commit feat: close paper query cancel path with verified gates |
| P0.2 | ef1b136 | `ef1b1369ce620f65be2311d534702160b37a13af` | P0.2a proposal and FUNCTION_TREE registry alignment committed; documentation/governance only. |
| P0.2b | implementation commit e182c52 feat: lock paper immediate-fill semantics | `e182c52e1b52d52888d71407e3072fd13da99993` | P0.2b implementation commit landed locally with fmt/clippy/test/GitNexus gates run before commit |
| P0.2c | implementation commit 51d4130 feat: add execution mode risk notices; gates passed: cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test --all-targets, git diff --check, function-tree validate/status/scope-check, GitNexus detect_changes LOW | `51d41300fe612ce53fae1ebc806527aa95d61999` |  |
| P0.2d | implementation commit be75f56 feat: add execution storage namespaces | `be75f56af27921674a1914f26d0055446d62aaf4` |  |
| P0.2e | P0.2d storage namespace helper is merged; next low-risk slice only adds config namespace semantics helpers/tests/docs, with GitNexus impact LOW for storage_namespace_for_channel and 0 affected processes. | `8845d8fae025498d7fec518bc0e4106f8603a635` |  |
| P0.2f | P0.2e merged at 4d82661 on master; P0.2 scope now spans PRs 232, 234, 235, 236, 240 plus metadata refreshes 233, 237, 238, 239. Current master is clean, FUNCTION_TREE active gates are 0, GitNexus detect_changes reports low/no process impact for the final semantics helpers, and remaining stale warnings are metadata/index lag only. | `4d8266146e636b47b826ebce901c998491ccdc61` |  |
| P0.3a | GitNexus impact: QmtLiveExecutionAdapter impl LOW; QmtTaskSubmitService impl LOW; request_diagnostics capability diagnostic HIGH | `01700f0c4619bdcd917854cdb8037db3dc06de83` | Design slice must avoid production symbol edits and mark request_diagnostics/CLI diagnostics as high-risk follow-up |
| P0.3b | Graphiti read attempted for P0.3b and timed out; existing P0.3a docs/handoff memories remain processing without error | `958d710e9183b73b62f88690328451d87391f7e1` | Continue from local merged report as authority; Graphiti backfill required if closeout ingest remains unavailable |
| P0.3c | GitNexus impact 2026-06-19: reconcile_qmt_live_order LOW direct=1 processes=2 modules=2; persist_qmt_live_query_failure LOW direct=1 processes=2 modules=2; try_update_order_qmt_live_metadata LOW direct=0 processes=0 modules=0. Index stale warning noted; staged diff fresh_for_staged_diff=true. | `be7669b89645682e2ff706af295fa326a40d4bd8` | Pre-edit impact confirms P0.3c remains LOW and qmt_live-local. |
| P0.3d | GitNexus impact 2026-06-19: qmt_live_adapter.submit_order LOW direct=3 processes=0 modules=1; qmt_task_submit_service.query_task_result_internal LOW direct=2 processes=0 modules=2; map_completed_result LOW direct=1 processes=0 modules=2; reconciliation.persist_qmt_live_query_failure LOW direct=1 processes=2 modules=2. Index stale warning noted. | `e5f8a3c96a47c01509f6d94552203e9fd3916976` | LOW candidates support a narrow qmt_live-local taxonomy seed. request_diagnostics and CLI remain out of scope. |
| P0.3e | GitNexus impact: ExecutionAdapter trait LOW direct=4 processes=0; QmtLiveExecutionAdapter impl LOW direct=0 processes=0; PaperExecutionAdapter impl LOW direct=0 processes=0; MockLiveExecutionAdapter impl LOW direct=0 processes=0 | `2eadf409bf82939243b29782efe2c35af9b722e6` | Pre-edit GitNexus impact for ExecutionCapabilities MVP |
| P0.3f | commit 844ecde feat: surface execution capabilities in qmt checklist; gates passed: RED/GREEN checklist test, execution_adapter_capabilities_test, qmt_live_adapter_test, fmt, clippy, cargo test, git diff --check, GitNexus detect_changes MEDIUM expected CLI status flow only | `844ecdecdf9f63ce1feb11fe81ac8520d28a92e4` | Implementation landed and verified before closeout |
| P0.4a | commit 2a2332f docs: design qmt live hardening plan | `2a2332f44ba3f2a4aca5cd141ddfec1ab1f84727` | P0.4a design report and FUNCTION_TREE update landed in local commit |
| P0.4b | commit 531efaa feat: add qmt live capability descriptor | `531efaaba3bd18d85e6e1eccac7e80354de0f7f6` | P0.4b implementation landed in local commit |
| P0.4c | implementation commit 2c1bac2 feat: enrich qmt live error taxonomy | `2c1bac2af30d30c86eb1b0eba5fcdca1109b0afa` |  |
| P0.4d | implementation commit 0910ce5 feat: classify qmt live gate mode failures | `0910ce5f856b162dd177681da4f174e1ec5667ef` | P0.4d implementation landed with report, tests, fmt, qmt_live filter, clippy, cargo test, GitNexus detect_changes LOW/0 affected processes |
| P0.4e | GitNexus impact: build_bridge_qmt_order_submit_capability_missing_diagnostics HIGH, direct=1, processes=2, modules=3; affected processes execute_execution_command and execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch | `8b88cf9f3b4108c3c1166cd99c044f5bd2211003` | P0.4e target impact 2 |
| P0.4f | GitNexus context: ReconciliationService.qmt_live_payload_json in src/execution/reconciliation.rs is called only by apply_qmt_live_result and persist_qmt_live_manual_intervention within reconciliation.rs; no indexed processes; planned change limited to identity metadata recovery inside qmt_live payload construction | `ef5400896f5389f6a51a8c88543b9ef80339d463` |  |
| P0.4g | Graphiti pre-read attempted for quantix_rust_main P0.4g/P0.4f design context; search_memory_facts timed out twice with Request timed out; continuing under documented Graphiti fallback | `25ca2c412315d32c5a9c6145c2cc7c219c921970` | Graphiti backfill required if durable P0.4g conclusions cannot ingest later |
| auction-collector-offline-test-seam | master CI run 27923085656 failed in sources::auction_collector::tests::test_auction_collector_creation; local cargo test --lib sources::auction_collector::tests::test_auction_collector_creation -- --test-threads=1 failed the same way; rustdx_complete::tcp::Tcp::new() in ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustdx-complete-0.6.6/src/tcp/mod.rs calls TcpStream::connect_timeout to a fixed stock IP, so the unit test depends on live external connectivity. | `aa0d9be833b8d3589ddd699429874f9cc896dd19` | external TDX connectivity makes constructor test nondeterministic |
| P0.5a | GitNexus impact: execute_execution_bridge_status LOW direct=1 processes=2; format_qmt_promotion_checklist LOW direct=1 processes=0 | `966e2a09cb050cf7ceacaa9162cfce6820276849` | Pre-edit GitNexus impact |
| P0.5b | GitNexus pre-edit impact: not applicable because P0.5b is docs-only and selects no production function, method, class, handler, trait, enum, storage schema, or bridge protocol symbol for editing; final GitNexus detect_changes remains required before commit | `acb5a359e5809052f1e51fdd637732c6d5a15a7b` | No source symbol selected for P0.5b |
| P0.5b-backfill | GitNexus impact not applicable: docs/governance-only Graphiti fallback record; final detect_changes required before commit | `5c576748aff234f0c46a40506d8ce721da6cdc2b` | No production symbol selected for backfill |
| P0.5c | GitNexus pre-edit impact: build_kill_switch_payload HIGH, load_blocking_kill_switch_state CRITICAL, execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch LOW; avoid HIGH/CRITICAL shared helper changes unless separately approved | `603b83b31a81a500d709719c80a5b95361eb2702` | P0.5c impact split |
| P0.5c-backfill | Graphiti episode 7adb64ad-1d99-4b3b-bbfb-b8229414982c failed ingest in quantix_rust_main with jsondecodeerror; local backfill record required. | `f7f0abf3656d45720fdc7840a9848f1d9e1f42ee` | P0.5c PR #272 and master CI already passed; this node only records Graphiti backfill. |
| P0.5d | Baseline: branch feat/p0-5d-qmt-live-audit-evidence from master b88e463; cargo test --quiet passed; Graphiti pre-read timed out; OpenSpec P0.5d requires audit view from existing runtime payloads without storage schema or behavior changes. | `b88e463170b0b088079f6de258b45ffb11af8f21` | P0.5d startup evidence. |
| P0.5e | baseline: master c747b79 clean worktree; Graphiti pre-read found P0.5d audit view and read-only list_orders; OpenSpec P0.5e requires read-only unresolved qmt_live manual-intervention list/show report from existing runtime payloads; no schema/write-path changes | `c747b7999958083cfbb474decf6871095769cf4e` |  |
| P0.5f | openspec/changes/qmt-live-operational-safety-p0-5/tasks.md | `8fb8b9554c95ca4635074db97e95ae8f075ea47b` | Release closure tasks 6.1-6.8 remain open after P0.5a-P0.5e implementation slices completed. |
| P0.5g | openspec/changes/qmt-live-operational-safety-p0-5/tasks.md | `fb60aab40a548af3c58175afc4c310e0a15c6464` | P0.5 gates 6.1-6.7 are complete after PR #277 merge, master CI, and Graphiti ingest; 6.8 OpenSpec archive remains. |
| P0.6 | baseline: clean master at 325d227; P0.5 archived; openspec has no active qmt_live change; starting P0.6 qmt_live runtime readiness planning only | `325d2276bf939b261385b8fc7a1e2decfe18a946` | P0.6 planning baseline |
| P0.6a | baseline: master 9846390 after P0.6 OpenSpec merge; P0.6a starts as docs/evidence-only runtime environment inventory and prerequisite check | `9846390ccd4ca20a7b4f5887b15f07728a26562a` | P0.6a baseline |
| P0.6b | baseline: master 5d853d9 after P0.6a; P0.6b starts as read-only smoke or blocked-by-environment evidence only | `5d853d96afb64fd54f52a447f43c29be44b57c68` | P0.6b baseline |
| P0.6b-backfill | baseline: Graphiti P0.6b closeout episode 9c3ca9b2 failed validationerror; retry cde99276 failed jsondecodeerror; local backfill record required | `7fc4235a3026e0503de52fcfb525f9f896e304d2` | P0.6b Graphiti backfill baseline |
| P0.6c | Implementation commit 5b8ff8c adds P0.6c runtime evidence README/template/report, OpenSpec task closure, FUNCTION_TREE entry, and repo hygiene coverage. Gates passed: fmt, OpenSpec validate, diff check, function-tree validate/gate/scope-check, repo_hygiene_test, GitNexus detect_changes LOW/0 affected processes. | `5b8ff8c5dc3e97ea8fb1a236da257948247d1311` | P0.6c remains read-only docs/test hygiene; no runtime source or broker mutation. |
| P0.6c-backfill | Backfill implementation commit 31b82f5 records Graphiti episode b580686d-69ff-485b-879b-e84f088e9422 as unverified/stuck processing and preserves equivalent P0.6c memory summary with Graphiti backfill required. | `31b82f5e6bd6230795c92dcf24154b48203cd4db` | Docs/governance only. |
| P0.6d | Implementation commit b67206d records P0.6d failure-boundary drill report and evidence. Executed read-only qmt status checklist returned ready=false/failure_category=bridge_unreachable; focused tests passed for preflight categories and kill-switch blocking/read-only availability. Gates passed: JSON parse, fmt, OpenSpec validate, diff check, function-tree validate/gate/scope-check, GitNexus detect_changes LOW/0 affected processes. | `b67206d8a9ae609a1f8bb382d390b047ef4dbd06` | No qmt_live live submit/cancel, manual-intervention resolution, broker/runtime mutation, or runtime source changes. |
| P0.6d-backfill | Backfill implementation commit 32aaf04 records Graphiti episode 4fe294c3-712e-49b8-b730-d627bc6d7fe5 as unverified/stuck processing and preserves equivalent P0.6d memory summary with Graphiti backfill required. | `32aaf04a3c35ec6dc8667b460319a2e569ae4504` | Docs/governance only. |
| P0.6e | P0.6a-P0.6d reports and OpenSpec tasks show qmt_live runtime readiness evidence complete except selected miniQMT Bridge runtime; P0.6e is documentation-only blocked_by_environment decision closure | `98160104fcacce16c1ff9aa95610da33b786b982` |  |
| P0.7a | GitNexus impact LOW: ExecutionChannel enum direct=0/processes=0; ExecutionChannel impl direct=0/processes=0; risk_notice_for_channel direct=1/processes=0; storage_namespace_for_channel direct=1/processes=0. Current master f0cf8b4 clean; P0.6 archived blocked_by_environment; scope is ExecutionCapabilities continuation only. | `f0cf8b4d847357c8d4c748ddc3c5d6e9384e4e61` | P0.7a pre-edit impact evidence |
| P0.7a-backfill | Graphiti episode 12d36617-8a3b-4a3b-8dff-d2ff91169a84 for P0.7a closeout remained processing after repeated polls; service status ok, queue_depth=0, last_error=null, attempt_count=1. Local fallback required by Graphiti workflow. | `42ac4f171fb5e4d2b0dd8671670413223676806f` | P0.7a Graphiti ingest processing fallback evidence |
| P0.7b | GitNexus impact before edits: format_qmt_promotion_checklist in src/cli/handlers/execution_handler.rs LOW, direct=0, affected processes=0, modules=0. Scope is checklist text only; no JSON response shape/runtime behavior changes. | `e785a67fd9e1df0fbca29d3e0bda46e2cb7f9d29` | P0.7b pre-edit impact evidence |
| P0.7b-backfill | Graphiti episode b8c58837-afd9-4810-a3e4-1d9b3eba837f for P0.7b closeout remained processing after repeated polls; queue_depth=0, last_error=null, attempt_count=1, processed_at=null. Local fallback required by Graphiti workflow. | `90b3ad6407d18f387b9d24f3580797461a1c0d9a` | P0.7b Graphiti ingest processing fallback evidence |
| P0.7c | GitNexus impact before edits: format_qmt_live_preflight_report in src/cli/handlers/execution_handler.rs LOW, direct=0, affected processes=0, modules=0. If report building is touched, build_qmt_live_preflight_report is also LOW, direct=1, affected processes=1, modules=1. Scope is human-readable qmt_live preflight text only. | `bfc68c878b3c1ba16f3b894e5e83b2603394f467` | P0.7c pre-edit impact evidence |
| P0.7d | GitNexus impact for request_diagnostics qmt_live diagnostic constructors returned HIGH; no production-code follow-up will be included in this node. | `f9778d5e039db483be996f33198d77df50fe12e9` | This node is docs/governance only. |
| P0.7d-backfill | Graphiti episode a7be5422-a926-416d-8c27-dea7f772d7e5 for P0.7d remained processing with queue_depth=0, attempt_count=1, no last_error after repeated polling, although search found extracted nodes. Local backfill required by workflow because ingest status did not reach completed. | `c9efd9c7ad667b17dd35504df5db675b49d73279` | Backfill node records equivalent memory only; no code changes. |
| P0.8 | GitNexus overview identifies Sources, Market, Io, Analysis, Strategy, and Execution clusters; detect_changes before edits reports no uncommitted changes. OpenStock symbol is not yet a code entry, so P0.8 first formalizes scope before implementation. | `4c9c7a6edc203ffa36d666eab78e399714d95128` | Avoids accidental qmt_live/miniQMT coupling. |
| P0.8-backfill | PR #297 merged as 821e723; Graphiti episode fb126253-d46e-41eb-98fd-924083015af3 remained processing with queue_depth=0 and last_error=null after repeated polling | `821e72302a3df0cfa5dd6e113d618b248a48777d` | Record local equivalent memory because ingest completion could not be verified. |
| P0.8a | openspec/changes/openstock-data-consumption-p0-8/tasks.md | `543ac7a7b6ea1be8ce2e1e6ab332f559bb1ca368` | P0.8a scope is inventory-only before parser/provider implementation. |
| P0.8a-backfill | PR #299 merged as 4779a76; master CI 28217484616 passed; Graphiti episode 914a72e6-369e-4100-9a28-7ae0d2846834 remained processing with queue_depth=0 and last_error=null after repeated polling | `4779a76e084c7f5d6673f6eb491f56b9fdc688f0` | Record local equivalent memory because ingest completion could not be verified. |
| P0.8b | baseline: branch feat/openstock-p0-8b-fixture-parser at 6c9f703, clean worktree before edits; P0.8a inventory recommends fixture-owned daily Kline parser/normalizer; Graphiti reads completed; GitNexus overview/query completed | `6c9f703d5a6a809d89c4d5c29a9e8cf63d85dfc9` | P0.8b baseline and prior evidence captured |
| P0.8b-backfill | Graphiti episode 3cd46c5b-3c6e-44ab-91b0-896af306753e for P0.8b closeout remained processing after repeated get_ingest_status polls: queue_depth=0, attempt_count=1, last_error=null, processed_at=null. | `02568972ac442b98168d5614bfff1b9c947fdae2` | Graphiti ingest stalled; local backfill required |
| P0.8c | HEAD eb83a4a on branch feat/openstock-p0-8c-fixture-validation; master clean before worktree creation; P0.8b parser baseline command running in worktree; OpenSpec tasks 3.1-3.4 define read-only local fixture validation, fail closed without fixture/config, no live OpenStock in CI, no ClickHouse writes; Graphiti read completed for quantix_rust_main/docs; GitNexus impact LOW for DataCommands and run_data_command | `eb83a4a208a46a7acea533566832f55908e39fbb` | P0.8c baseline and impact context |
| P0.8c-backfill | Graphiti episode 6192a37c-4d9a-461c-8d98-a4823de08cda remained processing after repeated get_ingest_status polling; PR #304 merged as 44b43f4 and master CI run 28254792250 passed | `44b43f43e56548d5e37a48df7e4156fb98dc0bae` | P0.8c closeout memory needs local fallback record |
| P0.8c-graphiti-completion-sync | Graphiti episode 6192a37c-4d9a-461c-8d98-a4823de08cda get_ingest_status returned completed with processed_at 2026-06-26T17:48:14.726558Z after PR #305 had been merged | `760aec5390859e68322ff61f2e4a9ec16a7b5e8e` | Correct P0.8c fallback docs to avoid stale backfill-required status |
| P0.8d | P0.8d RED missing test target; GREEN openstock_analysis_fixture_loop_test 1/1; parser 9/9; CLI 2/2; cargo fmt --check; clippy -D warnings; cargo test; OpenSpec single/all strict; git diff --check; GitNexus detect_changes LOW 0 affected processes | `c5f8b1330921821983904af3e5032e3d783cd633` |  |
| P0.8d-backfill | P0.8d closeout Graphiti episode fe2a3fd5-6b08-4f79-95a1-6723ce4985c4 remained processing after repeated get_ingest_status checks; queue_depth=0, last_error=null, attempt_count=1 | `c7565485bab26fca5f3f6f18e005c44c7bb6e6a6` |  |

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
- [ ] P0.6c: qmt_live runtime readiness evidence package (planning, FT: openspec/changes/archive/2026-06-24-qmt-live-runtime-readiness-p0-6/tasks.md#3-p06c-redacted-runtime-evidence-package)
- [ ] P0.6c-backfill: qmt_live runtime readiness Graphiti backfill (planning, FT: docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6C_2026-06-24.md)
- [ ] P0.6d: qmt_live runtime readiness failure boundary drill (planning, FT: openspec/changes/archive/2026-06-24-qmt-live-runtime-readiness-p0-6/tasks.md#4-p06d-failure-boundary-drill)
- [ ] P0.6d-backfill: qmt_live failure boundary Graphiti backfill (planning, FT: docs/reports/QMT_LIVE_RUNTIME_READINESS_P0_6D_2026-06-24.md)
- [ ] P0.6e: qmt_live runtime readiness decision report (planning, FT: openspec/changes/archive/2026-06-24-qmt-live-runtime-readiness-p0-6/tasks.md#5-p06e-readiness-decision-report)
- [ ] P0.7a: ExecutionCapabilities mode semantics bridge (planning, FT: ExecutionCapabilities continuation)
- [ ] P0.7a-backfill: P0.7a Graphiti backfill record (planning, FT: docs/reports/EXECUTION_CAPABILITIES_MODE_SEMANTICS_P0_7A_2026-06-25.md)
- [ ] P0.7b: ExecutionCapabilities checklist mode semantics (planning, FT: docs/reports/EXECUTION_CAPABILITIES_MODE_SEMANTICS_P0_7A_2026-06-25.md)
- [ ] P0.7b-backfill: P0.7b Graphiti backfill record (planning, FT: docs/reports/EXECUTION_CAPABILITIES_CHECKLIST_MODE_SEMANTICS_P0_7B_2026-06-25.md)
- [ ] P0.7c: ExecutionCapabilities preflight mode semantics (planning, FT: docs/reports/EXECUTION_CAPABILITIES_CHECKLIST_MODE_SEMANTICS_P0_7B_2026-06-25.md)
- [ ] P0.7d: ExecutionCapabilities P0.7 documentation sync (planning, FT: FUNCTION_TREE.md#execution-mode-semantics-hardening)
- [ ] P0.7d-backfill: P0.7d Graphiti backfill record (planning, FT: docs/reports/EXECUTION_CAPABILITIES_P0_7_DOC_SYNC_2026-06-26.md)
- [ ] P0.8: OpenStock data consumption OpenSpec (planning, FT: openspec/changes/openstock-data-consumption-p0-8)
- [ ] P0.8-backfill: OpenStock P0.8 Graphiti backfill (planning, FT: sources)
- [ ] P0.8a: OpenStock data consumption inventory (planning, FT: sources)
- [ ] P0.8a-backfill: OpenStock P0.8a Graphiti backfill (planning, FT: sources)
- [ ] P0.8b: OpenStock daily kline fixture parser (planning, FT: sources)
- [ ] P0.8b-backfill: OpenStock P0.8b Graphiti backfill (planning, FT: sources)
- [ ] P0.8c: OpenStock local fixture validation CLI (planning, FT: sources)
