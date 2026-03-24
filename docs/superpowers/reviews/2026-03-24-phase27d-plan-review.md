# Phase 27D Implementation Plan Review

**Reviewer:** Claude Code
**Date:** 2026-03-24
**Plan:** [2026-03-24-phase27d-risk-industry-blocklist-implementation.md](../plans/2026-03-24-phase27d-risk-industry-blocklist-implementation.md)
**Spec:** [2026-03-24-phase27d-risk-industry-blocklist-design.md](../specs/2026-03-24-phase27d-risk-industry-blocklist-design.md)
**Verdict:** ✅ Ready for execution (with one pre-flight check)

---

## Summary

The implementation plan correctly implements the spec's Option C (three-tier industry resolution), preserves all 6 design constraints, and follows TDD + GitNexus safety protocols. The plan is well-structured with logical chunk decomposition and clear commit hygiene.

---

## Strengths

| Aspect | Assessment |
|--------|------------|
| **TDD discipline** | Each task follows RED→GREEN→COMMIT cycle correctly |
| **GitNexus integration** | Impact analysis before edits, change detection before commits |
| **Scope isolation** | Clear file boundaries, no scope creep beyond spec |
| **Chunk decomposition** | Logical grouping: model → resolver → integration → docs |
| **Test coverage** | 13-point test matrix from spec mapped to concrete test files |
| **Commit hygiene** | Atomic commits per task, clear messages |
| **Assumption preservation** | 6 design constraints explicitly listed and preserved |
| **Fallback semantics** | Three-tier resolution (latest → month → fallback) clearly specified |

---

## Items to Verify Before Execution

### 1. Chunk 1, Task 1, Step 4 — Parsing edge cases

The spec mentions:
> "split on commas, trim surrounding whitespace, drop empty segments"

The plan's suggested test includes `银行, ,地产` (whitespace-only segment). **Confirm** the parser also handles:
- Trailing comma: `银行,`
- Leading comma: `,地产`
- Multiple commas: `银行,,地产`

### 2. Chunk 2, Task 2, Step 3 — DB path derivation

> "Derive the default DB path from the existing risk path sibling directory... `risk_state.json`'s directory plus `industry_snapshots.db`"

**Verify** this path exists and is writable in the current runtime. If `risk_state.json` is in a user-configurable location, ensure the derivation logic handles missing parent gracefully.

### 3. Chunk 2, Task 2, Step 4 — Market service boundary

> "Avoid touching `src/market/service.rs` unless absolutely necessary; prefer a thin risk-side adapter"

The spec says to use `sector_daily` with `sector_type = industry`. **Confirm** there's an existing query path for this that can be wrapped, or document if a new adapter is needed.

### 4. Chunk 3, Task 3, Step 4 — Injection boundary

> "Preserve existing injection-friendly constructors or extend them minimally"

**Verify** the current `RiskService::new()` signature and ensure the new `IndustryResolver` can be injected alongside the existing volatility loader without breaking existing tests.

### 5. Chunk 4, Task 4, Step 1 — Hygiene test expectations

The plan lists 6 doc expectations. The spec's acceptance criteria has 7 items. **Item 4** (month snapshot freeze) is implicit in the docs but may need explicit hygiene assertion.

---

## Minor Gaps (Non-Blocking)

| Gap | Impact | Recommendation |
|-----|--------|----------------|
| No explicit error-type design | Low | The plan uses string matching (`contains("latest/monthly/fallback")`). Consider a structured error enum for future extensibility. |
| `IndustrySourceTier` not defined | Low | Test assertions reference `IndustrySourceTier::SnapshotMonth` but the enum isn't in the file map. Add to `src/risk/industry.rs`. |
| Graphiti retry handling | Low | Plan mentions "Graphiti backfill required" fallback. Ensure this is appended to the plan file, not lost. |

---

## Pre-Flight Checklist

Before executing the plan:

```
□ Verify risk_state.json parent directory is stable and writable
```

---

## Recommended Next Step

```bash
/gsd:execute-phase 27d
```

Or clarify any of the verification items above before proceeding.

---

## Resolution Note

The implementation plan was updated after this review to address the concrete execution-facing items:

- expanded parser edge-case coverage for leading, trailing, and repeated commas
- added an explicit preflight check for the sibling snapshot DB path derived from `risk_state.json`
- clarified that a thin risk-side ClickHouse adapter is needed because there is no current direct `code -> industry` market-service query path
- made the dual injection boundary explicit so tests can provide both a fake volatility loader and a fake industry resolver
- added hygiene/documentation expectations for month-snapshot freeze-on-first-success semantics

Graphiti backfill required
