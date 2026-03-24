# Phase 29C Mock Live Execution Foundation Design Review

**Review Date:** 2026-03-22
**Design Document:** `2026-03-22-phase29c-mock-live-execution-foundation-design.md`
**Reviewer:** Claude Code
**Verdict:** ✅ **Approved for Implementation**

---

## 1. Architecture Compatibility Analysis

| Design Requirement | Current State | Compatibility |
|-------------------|---------------|---------------|
| `ExecutionAdapter` trait | ✅ Exists with `submit_order`, `query_order`, `cancel_order` | Fully compatible |
| `OrderStatus` enum | ✅ Has 8 states, **missing `PendingCancel`** | Minor change needed |
| `ExecutionKernel.execute_once()` | ✅ Implemented, assumes final status on submit | Needs modification |
| `recover_pending_orders()` | ⚠️ Placeholder returning empty result | Needs implementation |
| `orders` table | ⚠️ Missing `remaining_quantity`, `last_transition_at`, `version` | Migration needed |
| adapter field | ✅ Exists, hardcoded as "paper" | Needs parameterization |

**Conclusion:** Design direction is correct. Existing architecture provides excellent extension points.

---

## 2. Issues and Recommendations

### 2.1 OrderStatus Missing `PendingCancel` State

The design defines 9 states (including `pending_cancel`), but current `models.rs:36-45` only has 8.

**Recommendation:** Add `PendingCancel` variant to `OrderStatus` enum.

```rust
// src/execution/models.rs
pub enum OrderStatus {
    PendingSubmit,
    Submitted,
    Accepted,
    PartiallyFilled,
    PendingCancel,  // NEW
    Filled,
    Canceled,
    Rejected,
    Unknown,
}
```

---

### 2.2 orders Table Missing Lifecycle Tracking Fields

Design requires `remaining_quantity`, `last_transition_at`, `version`. Current table definition (`runtime_store.rs:59-77`) lacks these.

**Recommendations:**
1. Add migration logic in `ensure_schema()`
2. Update `OrderRecord` struct
3. Update `insert_order()` / `update_order()` methods

**Risk:** SQLite migration requires handling existing data. Suggest using `ALTER TABLE` or table rebuild strategy.

---

### 2.3 adapter Field Hardcoded as "paper"

`kernel.rs:240` has `adapter: "paper".to_string()` hardcoded, not supporting mock_live mode.

**Recommendation:** Get adapter name from `ExecutionRunRequest.mode` or adapter itself.

```rust
adapter: request.mode.clone(),  // or adapter.name()
```

---

### 2.4 sync_after_fill() Only Called on Filled Status

`kernel.rs:292-294` only syncs when `status == Filled`, but `PartiallyFilled` may also need risk state sync.

**Recommendation:** Per design doc, trigger sync when **new fill quantity** is observed:

```rust
if response.filled_quantity > previous_filled_qty {
    self.risk.sync_after_fill().await?;
}
```

---

### 2.5 RecoverySummary Missing Fields

Design requires `RecoverySummary` with `scanned`, `recovered`, `unchanged`, `failed`, `skipped`. Current (`kernel.rs:51-54`) only has `scanned` and `recovered`.

**Recommendation:** Extend struct to support complete recovery reporting.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySummary {
    pub scanned: usize,
    pub recovered: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub skipped: usize,
}
```

---

### 2.6 state_json Field Design

`mock_live_orders.state_json` example structure is clear, but suggestions:

- Define strongly-typed Rust struct `MockLiveOrderState`
- Use `serde` for serialization/deserialization
- Avoid runtime JSON parsing errors

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockLiveOrderState {
    pub fill_plan: Vec<FillStep>,
    pub next_step_index: usize,
    pub planned_fill_time: Option<DateTime<Utc>>,
    pub fault_injection: Option<FaultInjectionConfig>,
    pub unknown_until: Option<DateTime<Utc>>,
    pub cancel_requested: bool,
    pub last_applied_fill_id: u64,
    pub unknown_retries: u32,
    pub recovery_exhausted: bool,
    pub exhausted_reason: Option<String>,
}
```

---

## 3. Design Highlights

1. **Clear Boundary Definition:** Explicit "must" and "must not" lists prevent scope creep
2. **Option B Selection:** Lifecycle-first design lays foundation for future automation
3. **Unknown State Handling:** `unknown` as non-terminal + retry budget is pragmatic
4. **Optimistic Locking:** `version` field prevents concurrent modification conflicts
5. **Fill-Delta Semantics:** Paper account only changes on actual fill, matching real trading semantics

---

## 4. Test Coverage Assessment

| Layer | Coverage | Assessment |
|-------|----------|------------|
| Layer 1: adapter tests | 7 scenarios | ✅ Adequate |
| Layer 2: kernel tests | 6 scenarios | ✅ Adequate |
| Layer 3: CLI tests | 2 scenarios | ⚠️ Add error paths |
| Layer 4: integration | 6 scenarios | ✅ Adequate |

**Additional Recommendations:**
- Add `--mode live` error test in CLI tests
- Add `unknown` -> `recovery_exhausted` end-to-end test

---

## 5. Implementation Suggestions

### Recommended Commit Sequence

```
1. feat(execution): add PendingCancel status and extend RecoverySummary
2. feat(store): add remaining_quantity, last_transition_at, version to orders
3. feat(store): add mock_live_orders table and store helpers
4. feat(execution): add MockLiveExecutionAdapter
5. feat(kernel): support non-final states and fill-delta in execute_once
6. feat(kernel): implement recover_pending_orders
7. feat(cli): add --mode mock_live to strategy run
8. test: cover mock_live lifecycle
9. docs: document phase29c mock live execution boundary
```

---

## 6. Summary

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Architecture Compatibility | ⭐⭐⭐⭐⭐ | Perfect reuse of existing trait and kernel |
| Scope Control | ⭐⭐⭐⭐⭐ | Clear boundaries, no over-engineering |
| State Model | ⭐⭐⭐⭐⭐ | `unknown` non-terminal design is excellent |
| Data Model | ⭐⭐⭐⭐ | Migration needed, but design is sound |
| Test Strategy | ⭐⭐⭐⭐ | Adequate coverage, minor enhancements possible |

---

## 7. Review Verdict

**✅ APPROVED FOR IMPLEMENTATION**

The design document is high quality, compatible with existing code, and ready for implementation. The issues identified are minor and can be addressed during implementation without blocking progress.

---

## Appendix: Files Reviewed

- `src/execution/adapter.rs` - ExecutionAdapter trait definition
- `src/execution/kernel.rs` - ExecutionKernel implementation
- `src/execution/models.rs` - OrderStatus, OrderRecord, and related types
- `src/execution/runtime_store.rs` - StrategyRuntimeStore and schema definitions
- `src/execution/paper.rs` - PaperExecutionAdapter reference
