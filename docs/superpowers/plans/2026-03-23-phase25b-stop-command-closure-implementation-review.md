# Phase 25B Implementation Plan Review

**Review Date:** 2026-03-23
**Reviewer:** Claude (via /gsd:do)
**Plan File:** `docs/superpowers/plans/2026-03-23-phase25b-stop-command-closure-implementation.md`
**Design Spec:** `docs/superpowers/specs/2026-03-23-phase25b-stop-command-closure-design.md`

---

## Summary

The Phase 25B implementation plan is well-structured and aligned with its design spec. The TDD workflow, GitNexus integration, and chunk sequencing are solid. A few minor issues need correction before execution.

**Verdict:** ✅ **PASS with minor fixes required**

---

## ✅ Strengths

| Aspect | Assessment |
|--------|------------|
| **Goal clarity** | Clear, bounded scope with explicit non-goals (no auto-sell, no broker, no custom anchor mode) |
| **Architecture alignment** | Preserves existing `stop` → `trade` → `monitor` split correctly |
| **TDD workflow** | Each task follows RED→GREEN→COMMIT cycle with specific test commands |
| **GitNexus integration** | Pre-edit impact analysis and pre-commit change detection baked into every task |
| **Chunk sequencing** | Logical progression: models → service → evaluation → CLI → docs |
| **Anchor semantics** | Implementation Assumption section (lines 51-65) matches design spec exactly |
| **Error handling** | Validation matrix is complete and matches design spec |

---

## ⚠️ Issues Requiring Fixes

### Issue 1: Task 3 File Scope Ambiguity

**Location:** Line 266

**Problem:**
```markdown
- Modify: `src/trade/models.rs` (read-only reference only, do not edit unless necessary)
```

This is contradictory — "Modify" implies edits, but "(read-only reference only)" implies no edits.

**Recommendation:**
Either:
- Remove from file list and add note in task description: "Read-only dependency on `src/trade/models.rs` for `avg_cost` type"
- Or keep only if actual edits are expected (e.g., adding a helper method)

---

### Issue 2: Missing `stop_history` Migration Strategy

**Location:** Chunk 1, Step 4 (lines 133-140)

**Problem:**
The step says "create `stop_history`" but doesn't specify:
- Migration approach (inline `CREATE TABLE IF NOT EXISTS` vs `sqlx::migrate!`)
- Index strategy for efficient querying

**Recommendation:**
Add explicit guidance:

```markdown
Extend SQLite support:
- migrate `stop_rules` with nullable new columns using `CREATE TABLE IF NOT EXISTS` pattern
- create `stop_history` with:
  - PRIMARY KEY on `id`
  - INDEX on `(code, created_at)` for `--code` / `--date` filter queries
  - INDEX on `(event_type, created_at)` for `--type` filter queries
```

---

### Issue 3: Chunk 5 Test Command Typo

**Location:** Line 507

**Problem:**
```bash
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
```

This is the stop command closure phase, not strategy. The test filter should be `test_stop_`.

**Recommendation:**
```bash
cargo test --lib cli::handlers::tests::test_stop_ -- --nocapture
```

---

### Issue 4: Unclear Handler vs Service Responsibility

**Location:** Chunk 3, Step 4 (lines 319-328)

**Problem:**
```markdown
- handler helpers to build a `code -> avg_cost` map from local paper-trade account state
```

The design spec says handlers provide context, but the implementation plan doesn't specify:
- Which file owns the `avg_cost` map building logic?
- Is it a handler helper or a service method?

**Recommendation:**
Add clarification:

```markdown
Implement:
- threshold derivation for `loss_pct` / `profit_pct`
- quote-missing and anchor-missing states
- in `src/cli/handlers.rs`: add `build_avg_cost_map(account: &PaperAccount) -> HashMap<String, f64>`
  - this keeps trade-store reads at the CLI layer, not pushed into stop storage
- trigger-history writes inside the monitor stop evaluation loop
```

---

### Issue 5: Incomplete Final Memory Section

**Location:** Lines 529-537

**Problem:**
The Final Memory section only mentions Graphiti outcome recording. It lacks:
- Summary of deliverables
- Handoff notes for future phases
- Verification checklist sign-off

**Recommendation:**
Expand to:

```markdown
## Final Memory

- [ ] **Step 1: Record Graphiti outcome**

Write a conclusion-oriented Graphiti memory for the design and implementation outcome. If ingest fails, preserve an equivalent local summary and mark:

```text
Graphiti backfill required
```

- [ ] **Step 2: Phase completion summary**

Create `docs/superpowers/plans/2026-03-23-phase25b-stop-command-closure-COMPLETION.md`:
- Commands delivered: `stop set --loss-pct/--profit-pct`, `stop update`, `stop status`, `stop history`
- Schema changes: `stop_rules` extended, `stop_history` added
- Integration points: monitor evaluation, paper-trade account anchor resolution
- Deferred: auto-sell, real broker, custom anchor mode

- [ ] **Step 3: Verify acceptance criteria**

Confirm all 6 acceptance criteria from the design spec are satisfied:
1. [ ] users can define percent-based stop thresholds
2. [ ] users can patch rules via `stop update`
3. [ ] `stop status` shows evaluated thresholds and anchor source
4. [ ] `stop history` shows rule-change and trigger audit entries
5. [ ] monitor stop evaluation remains compatible
6. [ ] docs and hygiene tests reflect the new command surface
```

---

## ○ Suggestions (Non-blocking)

These are optional improvements that don't block execution:

| Area | Current | Suggestion |
|------|---------|------------|
| **Error i18n** | Design spec shows Chinese error `不能同时指定` | Confirm consistent i18n strategy for all validation errors |
| **History filter** | `--type trigger` not fully specified | Clarify if filter includes `loss`/`profit`/`trailing` subtypes or just `trigger` event class |
| **Transaction atomicity** | No mention of SQLite transaction boundaries | Consider explicit `BEGIN/COMMIT` for `set`+`history` atomic writes |
| **Percent precision** | Not specified | Document expected precision (e.g., 2 decimal places for percentages) |

---

## Checklist Before Execution

- [ ] Fix Issue 1: Clarify `src/trade/models.rs` status
- [ ] Fix Issue 2: Add `stop_history` migration strategy with indexes
- [ ] Fix Issue 3: Correct Chunk 5 typo (`test_strategy_` → `test_stop_`)
- [ ] Fix Issue 4: Clarify where `code -> avg_cost` map is built
- [ ] Fix Issue 5: Expand Final Memory section

---

## Next Steps

1. Apply fixes to the implementation plan
2. Run `/gsd:execute-phase 25b` to begin execution
3. Or run `/gsd:plan-phase 25b --auto` if further planning refinement is needed
