# Design Review: QMT Live Query Reconciliation Hardening

**Reviewed**: 2026-05-02
**Document**: `docs/superpowers/specs/2026-05-02-qmt-live-query-reconciliation-hardening-design.md`

## Baseline Claims — All Verified

| Claim | Status |
|-------|--------|
| `QmtTaskSubmitService` supports `query_task_result_by_task_id` | Confirmed at `src/execution/qmt_task_submit_service.rs:120` |
| `QmtLiveExecutionAdapter` uses task-receipt/result semantics | Confirmed in `src/execution/qmt_live_adapter.rs` |
| Runtime orders have `payload_json` field | Confirmed in `src/execution/runtime_store/orders.rs` and `schema.rs` |
| `ReconciliationService` is generic, no qmt_live-specific loop | Confirmed — `reconciliation.rs` only has `OpenOrderScanner` with status-based scanning |
| `OrderStatus` enum variants match design's eligibility list | Confirmed at `src/execution/models.rs:42-52` |
| `ReconciliationAction` includes `StateUpdated` and `ManualIntervention` | Confirmed at `reconciliation.rs:64-77` |

## Strengths

1. **Tight scope with explicit non-goals** — the 6 non-goals prevent scope creep, especially "no new SQLite tables" and "no automatic cancel-recovery."
2. **Failure strategy is conservative** — query failures don't create false terminal certainty. Rule 5 (bridge failure handling) and Rule 6 (missing task_id) are well-specified with concrete examples of what should remain non-terminal.
3. **Data model is minimal** — nesting under `payload_json.qmt_live` avoids schema changes and the "explicitly rejected data shapes" section preempts common scope expansion.
4. **CLI wording discipline** — distinguishing "waiting for convergence" vs "bridge query failure" vs "missing recovery identity" is a good operator-experience decision.

## Issues and Suggestions

### 1. Storage update gap is real but underspecified (Medium)

The design correctly identifies the need for "a targeted payload-update path that can safely modify order payload JSON without destroying unrelated payload fields" (line 288). However, no such function exists today — `Grep` for `update_order_payload` / `patch_payload` / `update_payload` returns nothing. This is the implementation's single largest unknown, and deserves a small subsection specifying:

- Whether it will be a generic `patch_payload_json(order_id, path, value)` or a typed `update_qmt_live_metadata(order_id, QmtLiveMetadata)` method
- Whether it will use SQLite JSON functions (`json_set`, `json_insert`) or read-modify-write at the Rust level
- Concurrency expectations — is there a risk of two reconciliation runs racing on the same order?

### 2. Rule 4 (Execution → Filled) defers fill accounting without a tracking mechanism (Low-Medium)

The design explicitly defers "precise fill quantity / average fill price recovery." This is fine for first pass, but `QmtTaskResolvedResult` already carries `filled_quantity` and `avg_fill_price` fields (line 35-36 of `qmt_task_submit_service.rs`). Consider persisting these into `last_query` so the data isn't lost when the next query overwrites it:

```json
"last_query": {
  "latest_status": "filled",
  "filled_quantity": 100,
  "avg_fill_price": "10.50",
  ...
}
```

This avoids needing a query-history array while still preserving the most recent fill snapshot.

### 3. Reconciliation eligibility silently skips `PartiallyFilled` (Low)

The design lists `PartiallyFilled` as not targeted in first pass. This is reasonable, but `PartiallyFilled` is a non-terminal state where the operator *does* need visibility. Consider adding a note that partially-filled orders should at minimum be visible in the CLI "unrecoverable" category (similar to the missing-task_id path), rather than being silently excluded.

### 4. No idempotency guarantee specified (Low)

If reconciliation runs overlap (e.g., two invocations before the first completes), the design doesn't specify expected behavior. Since `last_query` and `reconciliation` only store the latest snapshot, concurrent runs would naturally last-writer-wins. This is probably fine, but worth a one-line acknowledgment.

### 5. Test naming may not match acceptance commands (Low)

The acceptance commands use `cargo test qmt_live` and `cargo test reconciliation` as substring filters. This is correct for Rust's default test runner, but the design should note that test function names need to include these substrings — which is trivial but easy to overlook during implementation.

## Verdict

**Solid, implementable design.** Baseline assumptions are accurate against the codebase. The only actionable gap is specifying the payload-update mechanism (Issue 1), since it's the design's single novel infrastructure requirement and currently has zero implementation. The other items are minor refinements.

Recommended before implementation: add a "Storage Update Path" subsection under "Storage Update Strategy" addressing Issue 1, and optionally add `filled_quantity`/`avg_fill_price` to the `last_query` shape (Issue 2).
