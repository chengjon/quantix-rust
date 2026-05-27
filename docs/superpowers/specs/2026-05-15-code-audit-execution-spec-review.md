# Review: 2026-05-15-code-audit-execution-spec.md

**Type**: .md / spec | **Perspective**: auto (completeness + consistency + feasibility + architecture) | **Date**: 2026-05-15 | **Reviewer**: Claude

---

## Executive Summary

This execution spec is well-structured with clear phase contracts, acceptance criteria, and a thorough finding lifecycle. It correctly references all upstream methodology documents and existing evidence. However, it contains four unacknowledged departures from its primary methodology source (`CODE_AUDIT_METHODOLOGY.md`): a renamed finding status (`rejected` vs `wontfix`), four pattern-severity downgrades without justification, a restructured phase-to-module assignment, and a stricter finding lifecycle path. These divergences should either be explicitly documented as intentional overrides or reconciled with the methodology before execution begins.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | `docs/superpowers/specs/2026-05-15-code-audit-execution-spec.md` |
| File Type | .md |
| Doc Type | spec |
| Sections | 11 |
| Referenced Files | 7 found / 0 missing (of 7 input documents) |
| Referenced Symbols | 1 found / 0 missing (`AUDIT-S3-009`) |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `docs/standards/CODE_AUDIT_METHODOLOGY.md` | yes | Primary methodology |
| `docs/standards/CODE_AUDIT_METHODOLOGY_REVIEW_CODEX_2026-05-11.md` | yes | Review constraints |
| `docs/CODE_AUDIT_EVIDENCE/` | yes | 6 files: baseline.md, cargo-gates.md, pattern-scan-summary.csv, pattern-hotspots.md, manual-review-log.md, findings.csv |
| `FUNCTION_TREE.md` | yes | Root directory |
| `docs/standards/MOCK_USAGE_POLICY.md` | yes | Standards directory |
| `docs/CODE_AUDIT_EVIDENCE/gitnexus-queries.md` | no | Not yet created (expected during execution) |
| `docs/CODE_AUDIT_EVIDENCE/sampled-files.md` | no | Not yet created (expected during execution) |
| `docs/reports/CODE_AUDIT_FINAL_2026-05-15.md` | no | Not yet created (expected during execution) |

### Functions/Classes/Symbols Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `AUDIT-S3-009` | yes | `docs/CODE_AUDIT_EVIDENCE/findings.csv:20`, `docs/CODE_AUDIT_EVIDENCE/manual-review-log.md:126` |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| `src/execution/` exists with adapter, kernel, reconciliation | confirmed | 32 .rs files found under `src/execution/` |
| `src/strategy/` exists with daemon, registry, evaluator | confirmed | 14 .rs files found under `src/strategy/` |
| `src/risk/` exists with service, storage, industry | confirmed | 16 .rs files found under `src/risk/` |
| `src/bridge/` exists with client, models, error | confirmed | 4 .rs files found under `src/bridge/` |
| `src/cli/commands/*.rs` contains 13 command files | confirmed | 13 files found (matches methodology's last observation) |
| `src/cli/handlers/*.rs` contains handler files | confirmed | 30 files found (matches methodology's last observation) |
| `src/monitor/`, `src/monitoring/`, `src/stop/`, `src/account/`, `src/trade/` exist | confirmed | All directories verified with .rs files |
| P2/P3 modules exist (`src/analysis/`, `src/factor/`, etc.) | confirmed | 7 directories verified; note: `src/sources/` exists but `src/market/` also has sentiment sub-module |
| Finding schema matches methodology | contradicted | Status `rejected` replaces methodology's `wontfix` (see Findings) |
| Pattern severities match methodology | contradicted | 4 downgrades without justification (see Findings) |

## Checklist Results

### Completeness

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | 11 sections covering Purpose through Out-of-Scope Follow-Ups; each phase has acceptance criteria |
| C2 | Edge cases | PASS | Phase 0/1 acceptance criteria cover stale GitNexus, gate failures, and degraded confidence |
| C3 | Implicit assumptions | FAIL | Graphiti availability assumed without fallback (see Findings) |
| C4 | Acceptance criteria | PASS | Every phase has explicit acceptance criteria; completion criteria in section 10 is comprehensive |
| C5 | Missing roles/stakeholders | N/A | Single-auditor execution spec; roles not required |

### Consistency

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | FAIL | `rejected` vs methodology's `wontfix`; no acknowledgment of the change |
| N2 | Naming conventions | FAIL | 4 pattern severities downgraded without justification (see Findings) |
| N3 | Formatting | PASS | Consistent heading hierarchy, table formatting, and code blocks |
| N4 | Cross-references | PASS | Internal section references resolve correctly; all input documents exist |
| N5 | Style consistency | PASS | Consistent bilingual style matching methodology conventions |

### Feasibility

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | Hardest parts (GitNexus freshness, gate failures) addressed with fallback acceptance criteria |
| F2 | Dependency availability | PASS | Referenced tools (cargo, gitnexus, graphiti) exist in the environment |
| F3 | Timeline realism | N/A | No explicit timeline in the spec; methodology time estimates available |
| F4 | Resource constraints | N/A | Single-auditor execution; no personnel constraints |
| F5 | Rollback plan | PASS | Section 9 constraint: "Do not modify production code during audit" |

### Architecture

| # | Check | Result | Notes |
|---|-------|--------|-------|
| A1 | Component boundaries | N/A | Audit process spec, not software architecture |
| A2 | Data flow | PASS | Evidence flow from scan -> review -> findings -> report is explicit |
| A3 | Coupling | PASS | Phase dependencies are linear and clear |
| A4 | Interface contracts | FAIL | Finding schema diverges from methodology (see Findings) |
| A5 | Scalability | N/A | |
| A6 | Terminology consistency | FAIL | `rejected` vs `wontfix`, severity mismatches |
| A7 | Backward compatibility | FAIL | Reuses old evidence package but lifecycle path changes how findings close |
| A8 | Implementation surface precision | PASS | Every phase specifies exact files to create/refresh and directories to review |
| A9 | Named entities verified | PASS | All referenced files and the one named finding (AUDIT-S3-009) exist |

## Findings

### Critical Issues

None.

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Section 6 (Finding Lifecycle), lines 225-226 | `rejected` replaces methodology's `wontfix` without acknowledgment. Semantic shift: `wontfix` = "valid but we choose not to fix" vs `rejected` = "finding is invalid." The review constraints doc (`CODE_AUDIT_METHODOLOGY_REVIEW_CODEX_2026-05-11.md:312, 465, 553`) consistently uses `wontfix`. | Closure criteria for findings that are valid but unfixed changes. Existing findings.csv uses the methodology's status values. | Methodology line 737: `wontfix`; review doc lines 312, 465, 553: `wontfix`. Spec line 226: `rejected`. No "departure from methodology" note found in spec. | Either (a) use `wontfix` to maintain alignment with methodology and review constraints, or (b) add an explicit note documenting this as an intentional override with rationale, and update the methodology to match. |
| 2 | Section 5 Phase 2 (Pattern Scan), lines 119-127 | Four pattern severities downgraded from methodology without justification: `.expect(` HIGH->MEDIUM (113 occurrences across 8 files), `unsafe {` CRITICAL->HIGH (14 occurrences across 9 files), `println!` MEDIUM->LOW, `TODO` MEDIUM->LOW (spec names it "待办注记"). | In a trading system, `unsafe {` at CRITICAL (S0-level) means stop-and-fix; at HIGH (S1-level) it means fix-or-defer. The downgrade could change triage behavior for 14 `unsafe` occurrences. | Methodology lines 270-276: `.expect(` = HIGH, `unsafe {` = CRITICAL, `println!` = MEDIUM, `TODO` = MEDIUM. Spec lines 122-127: `.expect(` = MEDIUM, `unsafe {` = HIGH, `println!` = LOW, 待办注记 = LOW. | Add a note explaining each downgrade rationale, or revert to methodology severities. For `unsafe {` in a trading system, reverting to CRITICAL is strongly recommended. |
| 3 | Section 5 Phase 3-4, lines 162-192 | Phase-to-module assignment differs from methodology without acknowledgment. Spec Phase 3 includes `bridge/` and CLI commands/handlers (methodology Phase 4). Spec Phase 4 covers `monitor/monitoring/stop/account/trade` (methodology splits these across Phase 4-5). | Audit reviewer following methodology as primary reference will find different scope per phase than the spec expects. | Methodology lines 688-698: Phase 3 = execution/strategy/risk only; Phase 4 = bridge/cli/monitor/monitoring. Spec lines 162-192: Phase 3 = execution/strategy/risk/bridge/CLI; Phase 4 = monitor/monitoring/stop/account/trade/persistence/notifications. | Add an explicit note: "Phase module assignments restructured from methodology to consolidate bridge/CLI into P0 review." This documents the intent. |
| 4 | Section 7 (Finding Lifecycle), lines 267-278 | Spec requires `open -> accepted -> deferred` (acceptance before deferral). Methodology allows direct `open -> deferred`. Also spec's `rejected` rules differ: spec says "evidence that finding is invalid" while methodology's `wontfix` says "不修复，需说明技术依据" (won't fix, need technical rationale). | A finding that is valid but deliberately deferred must go through an acceptance step first in the spec, adding process overhead. The methodology allows direct deferral, which may be more appropriate for findings that are clearly valid but out of scope. | Methodology line 743-746: `open -> deferred` is a direct path; `wontfix` requires technical rationale. Spec lines 267-278: `open -> accepted -> deferred`; `rejected` requires evidence of invalidity. | If the stricter lifecycle is intentional, document it as a deliberate process improvement. Otherwise, allow direct `open -> deferred` to match methodology. |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 1 | Section 6 (Finding Schema), line 216 | `probable` confidence value (from methodology) not mentioned in spec's lifecycle rules. The findings.csv schema inherits the `confidence` field which the methodology defines as `confirmed / probable / needs-repro`. | Methodology line 727: `confirmed / probable / needs-repro`. Spec's lifecycle section (lines 265-278) only discusses statuses, not confidence values. No explicit exclusion of `probable`. | Add a note in Section 6 confirming whether `probable` is an allowed confidence value, or restrict to `confirmed` and `needs-repro` with explicit justification. |
| 2 | Section 10 (Completion Criteria), line 312 | Graphiti review memory ingestion completion is a hard gate, but no fallback if Graphiti is unavailable. Phase 0/1 have degraded-confidence patterns but Section 10 does not. | Spec line 312: "Graphiti review memory has completed ingestion" is a hard requirement. No exception path documented. | Add a degraded-completion path: "If Graphiti is unavailable, the audit may complete with a documented gap, and Graphiti ingest becomes a follow-up action item." |
| 3 | Section 5 Phase 2, line 126 | Pattern name "待办注记" diverges from methodology's `TODO[^-]` regex. Both target the same concept (TODOs without tracking), but use different naming conventions. | Methodology line 273: `TODO[^-]` (无 issue 编号). Spec line 126: "待办注记". | Use the methodology's regex-based name or add both names for cross-referencing. |

## Strengths

- **Phase acceptance criteria are specific and testable.** Each phase names exact output files, required content, and closure conditions. This is significantly more actionable than the methodology's time-based estimates alone.
- **Finding lifecycle is well-defined.** The required verification command, rationale, and acceptance criteria for each closure path provide a strong audit trail.
- **Existing evidence handling is pragmatic.** Section 8's approach of treating old evidence as historical rather than current truth, with explicit re-verification requirements, is sound.
- **FUNCTION_TREE.md authority is consistently asserted.** The status-source note and repeated references to FUNCTION_TREE.md as the sole feature registry prevent the audit from becoming a competing status source.
- **AUDIT-S3-009 carry-forward rule.** Explicitly naming a specific open finding ensures continuity across audit iterations.
- **Out-of-scope follow-up separation.** Section 11 prevents scope creep by requiring separate implementation plans for remediation work.

## Detailed Recommendations

1. **Add a "Departures from Methodology" section** (or a table at the end of Section 2) that lists every intentional change from the source methodology: the `rejected`/`wontfix` rename, severity downgrades, phase restructuring, and lifecycle path changes. This makes the spec self-documenting and prevents confusion during execution.

2. **Revert `unsafe {` severity to CRITICAL.** The codebase has 14 `unsafe` occurrences across 9 files. In a trading system where execution-path safety is paramount, the methodology's CRITICAL (S0-equivalent) classification is appropriate. The spec's HIGH (S1-equivalent) classification allows deferral with rationale, which is weaker than stop-and-fix.

3. **Reconcile `rejected` vs `wontfix`.** The existing `findings.csv` uses methodology terminology. If the spec intentionally changes this, it should also specify a migration rule for existing findings (all of which currently use the methodology's status values).

4. **Add Graphiti degraded-completion path.** Completion criteria in Section 10 should include an exception for Graphiti unavailability, consistent with the degraded-confidence pattern already used in Phase 0/1.

5. **Clarify `probable` confidence usage.** Either include `probable` in the spec's finding rules or explicitly exclude it with a note explaining why the spec is stricter than the methodology.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 4 | All referenced files and symbols verified present; pattern scan targets align with codebase reality |
| Completeness | 5 | 11 sections with comprehensive coverage; every phase has acceptance criteria |
| Codebase Alignment | 3 | 4 severity mismatches with methodology; status terminology diverges; phase assignments restructured without acknowledgment |
| Actionability | 5 | Each phase specifies exact output files, commands, and acceptance criteria |
| Terminology Consistency | 3 | `rejected`/`wontfix` mismatch with methodology and review constraints doc; bilingual naming inconsistency |
| **Overall** | **4.0** | |

## Verdict

**APPROVE_WITH_NOTES**

The spec is well-structured and actionable. The four unacknowledged departures from the source methodology (status rename, severity downgrades, phase restructuring, lifecycle path change) should be explicitly documented before execution begins. The `unsafe {` severity downgrade from CRITICAL to HIGH is the most impactful concern for a trading system and is recommended for revert. None of these issues block execution, but documenting them prevents confusion when cross-referencing the spec against the methodology during the audit.
