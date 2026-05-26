# Review: CODE_AUDIT_METHODOLOGY.md

**Type**: .md / methodology doc | **Perspective**: auto (completeness + consistency + feasibility + architecture) | **Date**: 2026-05-11 | **Reviewer**: Claude

---

## Executive Summary

This is a well-structured, comprehensive audit methodology document that demonstrates strong understanding of the codebase's architecture and risk landscape. However, it contains multiple factual inaccuracies in its code references -- the CLI command tree is materially wrong (lists subcommands as top-level), three state machine descriptions diverge from actual enum definitions, and adapter type names are abbreviated incorrectly. These errors would mislead any auditor attempting to follow the methodology without first verifying claims against the live code. The document is usable after targeted corrections.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | docs/standards/CODE_AUDIT_METHODOLOGY.md |
| File Type | .md |
| Doc Type | methodology (workflow + checklist hybrid) |
| Sections | 8 major + 2 appendices |
| Referenced Files | 15 found / 0 missing |
| Referenced Symbols | 12 found / 4 mismatched names |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `src/execution/mod.rs` | yes | src/execution/mod.rs |
| `src/execution/kernel.rs` | yes | src/execution/kernel.rs |
| `src/execution/reconciliation.rs` | yes | src/execution/reconciliation.rs |
| `src/execution/paper.rs` | yes | src/execution/paper.rs |
| `src/execution/mock_live.rs` | yes | src/execution/mock_live.rs |
| `src/execution/qmt_live_adapter.rs` | yes | src/execution/qmt_live_adapter.rs |
| `src/execution/qmt_bridge.rs` | yes | src/execution/qmt_bridge.rs |
| `src/strategy/daemon.rs` | yes | src/strategy/daemon.rs |
| `src/strategy/registry.rs` | yes | src/strategy/registry.rs |
| `src/risk/service.rs` | yes | src/risk/service.rs |
| `src/cli/commands/mod.rs` | yes | src/cli/commands/mod.rs |
| `src/cli/handlers/*.rs` | yes | 31 handler files in src/cli/handlers/ |
| `docs/RUST_CODING_STANDARDS.md` | yes | docs/RUST_CODING_STANDARDS.md |
| `scripts/verify_features.sh` | yes | scripts/verify_features.sh |
| `FUNCTION_TREE.md` | yes | FUNCTION_TREE.md (project root) |
| `config/` | yes | 13 config files in config/ |
| `tests/` | yes | 74 integration test files |

### Functions/Classes Referenced

| Symbol in Doc | Found? | Actual Name | Location |
|---------------|--------|-------------|----------|
| `ExecutionKernel` | yes | `ExecutionKernel` | src/execution/kernel.rs |
| `ExecutionRequest` | yes | `ExecutionRequest` (model) / `ExecutionRequestStatus` (enum) | src/execution/models.rs |
| `PaperAdapter` | **mismatch** | `PaperExecutionAdapter` | src/execution/paper.rs:12 |
| `MockLiveAdapter` | **mismatch** | `MockLiveExecutionAdapter` | src/execution/mock_live.rs:27 |
| `QmtLiveAdapter` | **mismatch** | `QmtLiveExecutionAdapter` | src/execution/qmt_live_adapter.rs:49 |
| `QmtBridgePreviewAdapter` | missing from doc | `QmtBridgePreviewAdapter` | src/execution/qmt_bridge.rs:8 |
| `Strategy::evaluate()` | **mismatch** | `ConfiguredStrategyEvaluator::evaluate()` | src/strategy/registry.rs:15 |
| `RiskService` | yes | `RiskService` | src/risk/service.rs |
| `RiskDecision` | **not found** | No such enum; closest: `ApprovalStatus` + `AutoReduceDecision` | src/execution/models.rs:112, src/risk/service/industry_checks.rs:171 |
| `execute_request()` | yes | `ExecutionKernel::execute_request()` | src/execution/kernel.rs:136 |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| "src/ 下全部 30 个模块" | **contradicted** | 28 subdirectories under src/ (plus lib.rs, main.rs as files) |
| "72 个集成测试文件" | **contradicted** | 74 .rs files in tests/ |
| "715+ .unwrap() 调用" | **unverified** | 438 matches in src/ production code; may have decreased since original count, or measurement method differs |
| "lib.rs <= 150 行" | **confirmed** | lib.rs is 50 lines |
| "handlers.rs 已拆分为 handlers/ 目录" | **confirmed** | 31 handler files in src/cli/handlers/ |
| GitNexus stats: "5410 符号, 13194 关系, 300 执行流" | **unverified** | Copied from CLAUDE.md; may be stale as index changes with each analyze |
| State machines match code | **contradicted** | All three state machine descriptions diverge from actual enums (see Findings) |

## Checklist Results

### Architecture (A1-A9)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| A1 | Component boundaries | PASS | Clear layer definition (CLI -> Service -> Provider/Adapter -> Domain -> Core) |
| A2 | Data flow | PASS | Execution chains described with explicit data movement |
| A3 | Coupling | PASS | Dependency direction verified; priority matrix reflects coupling risk |
| A4 | Interface contracts | PASS | Adapter trait boundaries and kernel contracts described |
| A5 | Scalability | N/A | Not a design doc; methodology focuses on audit, not scaling |
| A6 | Terminology consistency | FAIL | Adapter names abbreviated inconsistently (PaperAdapter vs PaperExecutionAdapter) |
| A7 | Backward compatibility | PASS | Principle stated; risk assessment accounts for API changes |
| A8 | Implementation surface precision | FAIL | Section 4.1.5 lists files to audit but does not specify exact functions to check |
| A9 | Named entities verified | FAIL | 4 of 12 referenced symbols use incorrect names |

### Completeness (C1-C5)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | All expected sections present: goals, scope, tools, dimensions, workflow, risk, deliverables |
| C2 | Edge cases | FAIL | State machine descriptions omit existing states (PendingCancel, Unknown, InProgress, Canceled) |
| C3 | Implicit assumptions | FAIL | Assumes `RiskDecision` enum exists; assumes CLI command tree is flat |
| C4 | Acceptance criteria | PASS | Each audit dimension has clear checklist items |
| C5 | Missing roles/stakeholders | N/A | Single-auditor methodology; no multi-role requirements |

### Consistency (N1-N5)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | FAIL | Mixes abbreviated and full names; "Created" vs "PendingSubmit"; "Cancelled" vs "Canceled" |
| N2 | Naming conventions | FAIL | Tool names use non-standard casing (`grep_files` vs `Grep`, `read_file` vs `Read`) |
| N3 | Formatting | PASS | Consistent heading hierarchy, table format, code block usage |
| N4 | Cross-references | PASS | Internal section links resolve; appendix references correct |
| N5 | Style consistency | PASS | Uniform formal Chinese technical writing throughout |

### Feasibility (F1-F5)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | P0 modules correctly identified as highest-risk areas |
| F2 | Dependency availability | PASS | All referenced tools (GitNexus, cargo clippy, Graphiti) exist in the project |
| F3 | Timeline realism | FAIL | Phase estimates (e.g., "P0 模块深度审查 120 min") assume no blockers; 3 modules x 40 min each is aggressive for deep audit |
| F4 | Resource constraints | N/A | Single-auditor methodology |
| F5 | Rollback plan | N/A | Audit produces reports, no code changes to roll back |

## Findings

### Critical Issues

_None._

### High Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | 4.2.1 (CLI command tree) | Command tree lists 25 top-level commands; actual CLI groups them under 12 subcommand categories. `ai`, `fundamental`, `import`, `news`, `notify`, `sentiment` are subcommands of `info`; `screener`, `task` are under `analysis`; `watchlist` is under `market`; `stop` is under `monitor`; `algo`, `anomaly`, `execution` are under `trade`; `data-source` is under `data`. | Auditor will waste time looking for top-level commands that do not exist; audit checklist items about "command naming consistency" will produce false findings. | Read `src/cli/commands/mod.rs` lines 1-44: commands are grouped as AccountCommands, AnalyzeCommands, BacktestCommands, DataCommands, FactorCommands, InfoCommands (containing Ai/Fundamental/Import/News/Notify/Sentiment), MarketCommands (containing Watchlist), MonitorCommands (containing Stop), PerformanceCommands, RiskCommands, StrategyCommands, TradeCommands (containing Algo/Anomaly/Execution). Verified by reading full file. Document section 4.2.1 does not address this grouping anywhere. | Rewrite the command tree to reflect actual subcommand hierarchy. Use `quantix --help` output or read `src/cli/commands/mod.rs` directly. |
| 2 | 4.3.2 (State machines) | All three state machine descriptions diverge from actual enum definitions. **Order**: doc says `Created -> Submitted -> Accepted -> PartiallyFilled -> Filled/Rejected/Cancelled`; actual is `PendingSubmit, Submitted, Accepted, PartiallyFilled, PendingCancel, Filled, Canceled, Rejected, Unknown` -- missing `PendingCancel`, `Unknown`, no `Created`. **ExecutionRequest**: doc says `Pending -> Queued -> Submitted -> Completed/Failed`; actual is `Pending, InProgress, Completed, Failed, Canceled` -- no `Queued` or `Submitted`, has `InProgress` and `Canceled`. **StrategyRun**: doc says `Inactive -> Running -> Paused -> Stopped`; actual is `Running, Success, Failed` -- completely different. | Auditor will check for states that do not exist (Created, Queued, Paused) and miss states that do (PendingCancel, Unknown, Canceled, InProgress, Success). Checklist items about "dead states" and "closure" will be based on wrong models. | Read `src/execution/models.rs`: `OrderStatus` (line 42-52), `ExecutionRequestStatus` (line 138-144), `StrategyRunStatus` (line 16-20). Also verified spelling: code uses `Canceled` (single L), document uses `Cancelled` (double L). Document section 4.3.2 does not caveat these as approximate. | Replace all three state machine descriptions with exact enum variants from `src/execution/models.rs`. Copy-paste the actual enum definitions to prevent drift. |
| 3 | 4.1.3 / 4.3.1 (Adapter names) | Document uses abbreviated names `PaperAdapter`, `MockLiveAdapter`, `QmtLiveAdapter`. Actual struct names are `PaperExecutionAdapter`, `MockLiveExecutionAdapter`, `QmtLiveExecutionAdapter`. | Auditors using grep/search for the abbreviated names will get zero results, wasting time or concluding the types do not exist. | Grep for `PaperAdapter` in src/ returns 0 matches. Grep for `PaperExecutionAdapter` finds it in `src/execution/paper.rs:12`. Same pattern for MockLive and QmtLive. Document does not note these are abbreviated for readability. | Use actual struct names from code. If abbreviating for readability, add a naming table mapping abbreviations to full names. |

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 4 | 4.3.1 (Risk chain) | `RiskDecision (approve / reject / reduce)` does not exist as a type. Closest matches: `ApprovalStatus { Pending, Approved, Rejected }` in `src/execution/models.rs:112` and `AutoReduceDecision` in `src/risk/service/industry_checks.rs:171`. The document implies a single unified decision type. | Auditor searching for `RiskDecision` will find nothing; risk chain diagram implies a type contract that does not match code. | Grep for `RiskDecision` returns 0 matches across entire src/. Grep for `Decision` finds only `AutoReduceDecision`. The risk evaluation likely returns `ApprovalStatus` or a service-level result, not a single `RiskDecision` enum. | Verify actual risk evaluation return type by reading `RiskService::evaluate()` signature and update the diagram. |
| 5 | 3.1 / 3.3 (Tool names) | Document references `grep_files` and `read_file` as tool names. Actual Claude Code tool names are `Grep` and `Read`. Similarly, `GitNexus context` should be `gitnexus_context`, `GitNexus impact` should be `gitnexus_impact`. | Anyone executing the methodology (including AI agents) will use wrong tool names, causing confusion or failed invocations. | Checked tool names in Claude Code: `Grep`, `Read`, `gitnexus_context`, `gitnexus_impact`, `gitnexus_query`. Document section 3.1 table and section 3.2 code blocks use inconsistent names. Document section 3.3 uses `grep_files` which does not exist. | Standardize all tool references to actual tool names. Add a mapping table if the document is intended for non-Claude-Code environments. |
| 6 | 2.1 (Module count) | Document claims "src/ 下全部 30 个模块" but actual count is 28 subdirectories. The priority matrix (section 2.3) lists exactly 28 entries (3 P0 + 4 P1 + 6 P2 + 15 P3 = 28), creating an internal inconsistency. | Internal contradiction: section 2.1 says 30, section 2.3 and Appendix A.2 show 28. | `ls -d src/*/` returns 28 directories. Counting all entries in Appendix A.2 yields 28. Section 2.1 explicitly states 30. Document does not explain the discrepancy. | Change "30" to "28" in section 2.1, or clarify what the additional 2 items are (e.g., lib.rs/main.rs as top-level files). |
| 7 | 4.5.1 (Test count) | Document claims "72 个集成测试文件" but `tests/` contains 74 .rs files. | Minor factual error; could cause auditor to think 2 tests are unexpected or missing. | `ls tests/*.rs | wc -l` returns 74. Document section 4.5.1 does not note this is an approximate count. | Update to "74" or qualify as "~72" if approximate. |
| 8 | 4.3.1 (Strategy evaluate) | Document writes `Strategy::evaluate()` implying a method on a `Strategy` type. Actual trait is `ConfiguredStrategyEvaluator` with `evaluate(&self, klines: &[Kline]) -> Result<SignalEnvelope>`. There is also a `Strategy` trait in `trait_def.rs` but it does not define `evaluate()`. | Misidentifies the trait that produces signals; auditor looking for `Strategy::evaluate` will not find the correct implementation. | Grep for `fn evaluate` in `src/strategy/` finds it only in `registry.rs:15` on `ConfiguredStrategyEvaluator`. `Strategy` trait in `trait_def.rs` does not have `evaluate`. | Change to `ConfiguredStrategyEvaluator::evaluate()`. |
| 9 | 4.1.3 (Missing adapter) | Document's execution boundary table lists `qmt_bridge adapter -> QMT Preview/Submit` but does not mention `QmtBridgePreviewAdapter` by name. The adapter exists in `src/execution/qmt_bridge.rs:8` and is a distinct struct from `QmtLiveExecutionAdapter`. | Auditor may not realize there are two separate QMT adapters (preview vs live submission) and may conflate their responsibilities. | Grep for `struct.*Adapter` in `src/execution/` finds 5 adapter structs: `AdapterOrderRequest`, `PaperExecutionAdapter`, `MockLiveExecutionAdapter`, `QmtLiveExecutionAdapter`, `QmtBridgePreviewAdapter`. Document only names 3. | Add `QmtBridgePreviewAdapter` to the execution boundary table and adapter list. |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 10 | 4.1.4 (File size: mod.rs) | `cli/commands/mod.rs` is 267 lines, above the implied "mod.rs should only contain mod + pub use" rule. Contains `#[derive(Parser)]` command definitions mixed with re-exports. | Read file: lines 1-50 show mod declarations + re-exports, but lines 49+ contain the `QuantixCli` struct with `#[derive(Parser)]` and `#[command]` attributes. Document section 4.1.4 does not flag this file. | Note this as a known deviation; consider whether command definitions should be in a separate file. |
| 11 | 6.3 (unwrap count) | Document states "715+ .unwrap() 调用" as a known debt baseline. Current grep finds 438 in src/ production code (excluding test modules). Count may have decreased since original audit, or measurement differs (e.g., including test code). | `grep -r '\.unwrap()' --include='*.rs' src/ | grep -v '#\[cfg(test)\]' | grep -v 'tests/' | wc -l` returns 438. CLAUDE.md (Known Tech Debt table) also states 715. Both may be stale. | Re-run the count with a precise method (e.g., excluding `#[cfg(test)]` blocks properly) and update the baseline. |
| 12 | 5.1 (Phase estimates) | Phase time estimates assume no blockers (e.g., P0 modules x 120 min total for execution + strategy + risk + cli). For a deep audit with code reading and evidence collection, 30 min per module is aggressive. | Document section 5.1 allocates 120 min for 4 P0 modules = 30 min each. Even with GitNexus assist, reading kernel.rs (730+ lines), daemon.rs, and their tests exceeds 30 min per module. | Add a note that estimates are minimums; budget 2x for modules with known complexity. |
| 13 | 3.2 (GitNexus API syntax) | GitNexus usage examples use pseudo-code syntax (`READ gitnexus://repo/...`, `gitnexus_query(query=...)`). These mix MCP resource URIs with tool call syntax in a non-executable format. | Section 3.2 code blocks use `READ` (not a Claude Code tool) and `gitnexus_query(query=...)` (should be `gitnexus_query` tool with `{query: "..."}` JSON params). Document does not clarify these are illustrative. | Either mark examples as pseudocode, or use exact tool invocation syntax. |

## Strengths

- **Thorough risk prioritization**: The P0-P3 priority matrix correctly identifies execution, strategy, and risk as the highest-risk modules, matching the codebase's actual critical paths.
- **Evidence-based approach**: The methodology demands code citations, dual confirmation for CRITICAL/HIGH findings, and cross-references to coding standards. This is sound audit practice.
- **Decision tree for severity**: Section 6.2's decision tree provides clear, reproducible severity classification. The S0-S4 scale with response-time expectations is practical.
- **Comprehensive checklist structure**: Section 4's 5 audit dimensions with specific checklist items gives auditors a clear framework. The self-check appendix (B) is well-designed.
- **Deliverable specification**: Section 7 defines concrete outputs (report structure, CSV findings, module health reports) with enough detail to prevent vague deliverables.
- **MOCK boundary awareness**: The document repeatedly emphasizes distinguishing MOCK, real, and fallback paths -- a critical concern for a trading system.

## Detailed Recommendations

1. **Regenerate CLI command tree from code**: Read `src/cli/commands/mod.rs` and `src/cli/handlers/mod.rs` to produce the accurate command hierarchy. The tree should show the actual 12 command groups with their subcommands nested beneath.

2. **Copy-paste enum definitions for state machines**: Rather than describing state machines in prose, embed the actual Rust enum definitions (with source file references) into section 4.3.2. This prevents drift and gives auditors exact variants to check against.

3. **Add a naming reference table**: Create a table mapping document abbreviations to actual struct names:
   - `PaperAdapter` -> `PaperExecutionAdapter` (src/execution/paper.rs:12)
   - `MockLiveAdapter` -> `MockLiveExecutionAdapter` (src/execution/mock_live.rs:27)
   - `QmtLiveAdapter` -> `QmtLiveExecutionAdapter` (src/execution/qmt_live_adapter.rs:49)
   - `QmtBridgePreviewAdapter` (src/execution/qmt_bridge.rs:8) -- new entry

4. **Standardize tool names**: Replace all `grep_files` with `Grep`, `read_file` with `Read`, and fix GitNexus tool invocation syntax to match actual MCP tool signatures.

5. **Verify risk evaluation contract**: Read `RiskService::evaluate()` return type and update section 4.3.1's risk chain to use the actual decision type rather than the assumed `RiskDecision`.

6. **Re-run baseline metrics**: Execute the violation pattern searches from section 3.3 against the current codebase and update the known-risk baseline in section 6.3 with current counts.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 2 | 3 state machines wrong, 4 symbol names wrong, CLI tree wrong, module/test counts wrong |
| Completeness | 4 | All major audit dimensions covered; only missing QmtBridgePreviewAdapter and risk decision type |
| Codebase Alignment | 2 | Multiple factual claims contradicted by live code; adapter names, state machines, command tree |
| Actionability | 4 | Clear checklists, deliverable specs, and decision trees; reduced by incorrect code references |
| Terminology Consistency | 3 | Internal document is consistent, but diverges from codebase naming (Canceled vs Cancelled, adapter abbreviations) |
| **Overall** | **3.0** | |

## Verdict

**NEEDS_REVISION**

The methodology's structure, scope, and process design are excellent. However, the document contains enough factual errors about the codebase (wrong CLI command tree, wrong state machines, wrong adapter names, wrong module count, non-existent types) that an auditor following it literally would be misdirected. The document should be corrected against the live code before being used as an audit reference. All HIGH findings (items 1-3) are blocking issues that must be resolved first.
