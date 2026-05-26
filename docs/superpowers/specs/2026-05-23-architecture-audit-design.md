# Architecture Audit Design — quantix-rust

> Date: 2026-05-23
> Status: Audit Complete — Three-document deliverable set reviewed and baselined
> Method: GitNexus Graph Analysis + Pattern Scan + Test Coverage Analysis (方案 A)

## Scope

- **Project**: quantix-rust (A股量化交易 CLI 工具)
- **Scale**: 314 files / 80,261 lines / 28 top-level modules
- **Previous audit**: 2026-05-15 (runtime gates + pattern scan, no S0/S1)

## Audit Dimensions

| # | Dimension | Core Question | Tool |
|---|-----------|--------------|------|
| D1 | Module Dependency & Layering | Does dependency direction follow `cli → service → provider/adapter → domain → core`? Circular deps? Layer violations? | GitNexus Cypher |
| D2 | Code Quality & Tech Debt | unwrap (970), large files, println in lib modules, error handling coverage | Pattern scan + source verification |
| D3 | Module Responsibility & Cohesion | Are module responsibilities clear? Mixed concerns? Cross-module duplication? | GitNexus query + manual review |
| D4 | Test Coverage & Quality | Are critical paths tested? Meaningful or formalistic tests? | Test file scan + structural analysis |

## Out of Scope

- Feature bug fixes
- New feature development
- CI/CD pipeline optimization
- Performance optimization

## Execution Phases

### Phase 1: GitNexus Graph Analysis (D1 + D3)

| Step | Analysis | Target |
|------|----------|--------|
| 1.1 | Inter-module IMPORTS/CALLS topology | Map dependency direction, identify violations |
| 1.2 | Circular dependency detection | Find mutual import pairs |
| 1.3 | Layer violation detection | Lower layers importing higher layers |
| 1.4 | Cross-module duplication detection | Similar symbols across modules |
| 1.5 | Module cohesion analysis | Whether each module's functional clustering is compact |

### Phase 2: Code Quality Pattern Scan (D2)

- File size compliance (CLAUDE.md thresholds)
- Error handling patterns (unwrap / expect / panic)
- Logging conventions (println in lib modules)
- Type system usage
- Public API consistency

### Phase 3: Test Coverage Analysis (D4)

- Scan all `tests/` dirs and `#[cfg(test)]` blocks
- Compare module count vs test module count
- Identify zero-coverage critical modules

## Deliverables and Document Relationships

### Three-Document Set

| Document | Role | Status |
|----------|------|--------|
| `docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md` | Primary audit report (findings, evidence, roadmap) | REVIEWED BASELINE — D2 tables and roadmap labels corrected per review |
| `docs/reports/ARCHITECTURE_AUDIT_REVIEW_2026-05-23.md` | Correction baseline — identifies evidence errors and refines recommendations | Baseline — preserved as-is; Phase 0 corrections applied to primary report |
| `docs/superpowers/specs/2026-05-23-architecture-audit-design.md` | This document — design scope, dimensions, deliverable relationships | Up to date |

### Resolution Rules

1. When the primary report and the review conflict, the **review's architecture recommendations** take precedence (Signal-only extraction, characterization tests first, etc.)
2. D2 evidence tables in the primary report have been **re-verified against source**; the review's Phase 0 correction checklist tracks the delta
3. The review's 6-phase roadmap supersedes the primary report's 5-sprint roadmap where they differ
4. `FUNCTION_TREE.md` remains the sole feature-status registry; no audit document overrides it

### Issue Gate

10 GitHub issues (#63–#72) were created from the review's Suggested Issue Batch. Issues should not be created from the primary report's original sprint tables, which may contain stale labels.
