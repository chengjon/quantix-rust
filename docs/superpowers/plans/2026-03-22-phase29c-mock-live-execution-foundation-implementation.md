# Phase 29C Mock Live Execution Foundation Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `mock_live` execution mode for `strategy run` with durable non-final order lifecycle semantics, while preserving the existing immediate-fill `paper` path.

**Architecture:** Extend the existing `StrategyRuntime -> ExecutionKernel -> ExecutionAdapter` flow rather than replacing it. Add a dedicated `MockLiveExecutionAdapter`, extend runtime SQLite to carry public lifecycle fields plus adapter-private mock state, and teach the kernel to handle non-final statuses, fill deltas, and recovery passes.

**Tech Stack:** Rust, Tokio, SQLx/SQLite, serde, clap, existing paper-trade and risk services, GitNexus for code navigation.

---

Graphiti backfill completed on 2026-03-24; related memories are now present in Graphiti.

## Local Checkpoints

### Task 2 Checkpoint

- Runtime store schema now extends `orders` with `remaining_quantity`, `last_transition_at`, and `version`.
- `mock_live_orders` now exists for adapter-private typed state storage.
- Typed store helpers now cover insert/get/update of `MockLiveOrderState`, optimistic locking via `try_update_order_with_version`, and non-terminal listing via `list_recoverable_mock_live_orders`.
- Verification passed with:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
```

- Result: `18 passed, 0 failed`.
- Graphiti write for this checkpoint failed during ingest because of upstream rate limiting (`429`).

Graphiti backfill completed on 2026-03-24; related memory is now present in Graphiti.

### Task 3 Checkpoint

- `ExecutionAdapter` 现在暴露 `adapter_name()`.
- `PaperExecutionAdapter` 和测试里的 `CountingAdapter` 已实现 adapter identity。
- `src/execution/mod.rs` 已导出 `mock_live`.
- `src/execution/mock_live.rs` 已新增最小 shell：
  - `MockLiveClock`
  - `MockLiveExecutionAdapter::new(...)`
  - `MockLiveExecutionAdapter::with_state_template(...)`
  - `submit_order` 默认返回 `Accepted`
  - `query_order` 支持 partial -> fill 推进
  - `cancel_order` 支持 cancel_requested 后在下一次 query 收敛为 `Canceled`
  - `unknown_once` fault 在 follow-up query 中可恢复
- 验证通过：

```bash
cargo test --test mock_live_adapter_test -- --nocapture
cargo test --test execution_kernel_test kernel_success_path_persists_run_signal_order_and_events -- --nocapture
```

- 结果：
  - `mock_live_adapter_test`: `4 passed, 0 failed`
  - targeted `execution_kernel_test`: `1 passed`
- Graphiti write for this checkpoint failed during ingest because of upstream rate limiting (`429`).

Graphiti backfill completed on 2026-03-24; related memory is now present in Graphiti.

### Task 4 Checkpoint

- `ExecutionKernel::execute_once()` 不再把 adapter 写死为 `paper`，现在会记录 `self.adapter.adapter_name()`.
- 针对 `Accepted` 和 `PartiallyFilled` 的 kernel 行为已加测试。
- `CountingAdapter` 测试桩现在支持 `Filled` / `Accepted` / `PartiallyFilled` 三种 submit 响应。
- `FixedRiskEvaluator` 测试桩现在能统计 `sync_after_fill()` 调用次数。
- 当前实现已满足：
  - non-final submit 会保留 adapter identity
  - partial fill 会触发 `sync_after_fill()`
- 验证通过：

```bash
cargo test --test execution_kernel_test -- --nocapture
```

- 结果：`14 passed, 0 failed`
- Graphiti write for this checkpoint failed during ingest because of upstream rate limiting (`429`).

Graphiti backfill completed on 2026-03-24; related memory is now present in Graphiti.

### Task 5 Checkpoint

- `recover_pending_orders()` 已从占位实现升级为真实 recovery loop。
- 当前 recovery 会：
  - 扫描 `list_recoverable_mock_live_orders()`
  - 调用 adapter `query_order`
  - 用 `try_update_order_with_version()` 做乐观锁更新
  - 在需要时写入新的 `order_events`
  - 对正向 fill 增量调用 `sync_after_fill()`
  - 在 `Unknown` 超过预算时写 `recovery_exhausted` 事件，但保持公共状态为 `Unknown`
- `MockLiveExecutionAdapter` 现在额外支持 `unknown_always` 故障模式，用于 recovery 耗尽测试。
- 验证通过：

```bash
cargo test --test execution_kernel_test -- --nocapture
cargo test --test mock_live_adapter_test -- --nocapture
```

- 结果：
  - `execution_kernel_test`: `17 passed, 0 failed`
  - `mock_live_adapter_test`: `4 passed, 0 failed`
- Graphiti write for this checkpoint failed during ingest because of upstream rate limiting (`429`).

Graphiti backfill completed on 2026-03-24; related memory is now present in Graphiti.

### Task 6 Checkpoint

- CLI parser 现在接受 `strategy run --mode mock_live`.
- `execute_strategy_run_with_components(...)` 现在支持 `mock_live` 分支：
  - 复用现有 StrategyRuntime
  - 复用现有 risk bridge
  - 使用 `MockLiveExecutionAdapter::new(..., SystemMockLiveClock)`
- `paper` 路径保持不变，`live` 仍然 unsupported。
- 修复了 handler 中 `trade_store` 的所有权问题，改为在构造 risk bridge 前 clone。
- 验证通过：

```bash
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --lib cli::handlers::tests::test_strategy_ -- --nocapture
```

- 结果：
  - `cli::tests::strategy`: `5 passed`
  - `cli::handlers::tests::test_strategy_`: `4 passed`
- Graphiti write for this checkpoint failed during ingest because of upstream rate limiting (`429`).

Graphiti backfill completed on 2026-03-24; related memory is now present in Graphiti.

## File Map

### Core execution files

- Modify: `src/execution/adapter.rs`
  - add concrete adapter identity API so kernel stops hardcoding `"paper"`
- Modify: `src/execution/mod.rs`
  - export the new mock-live module
- Modify: `src/execution/models.rs`
  - add `PendingCancel`
  - extend `OrderRecord`
  - add `MockLiveOrderState`
  - extend `RecoverySummary`
- Modify: `src/execution/runtime_store.rs`
  - extend `orders` schema
  - add `mock_live_orders` table
  - add optimistic-lock update helpers
  - add typed mock-live state read/write helpers
- Modify: `src/execution/kernel.rs`
  - stop assuming submit returns final status
  - add fill-delta application path
  - implement recovery scan / retry / exhaustion behavior
- Create: `src/execution/mock_live.rs`
  - implement `MockLiveExecutionAdapter`
  - add mock clock abstraction and simulation-plan helpers

### CLI and user-facing files

- Modify: `src/cli/handlers.rs`
  - wire `strategy run --mode mock_live`
  - print current state truthfully
- Modify: `src/cli/tests/strategy.rs`
  - add parser coverage for `--mode mock_live`
- Modify: `README.md`
  - document mock-live execution boundary
- Modify: `docs/USER_MANUAL.md`
  - add mock-live usage and state semantics
- Modify: `tests/repo_hygiene_test.rs`
  - update doc expectations for strategy runtime modes

### Test files

- Modify: `tests/execution_runtime_store_test.rs`
  - cover schema extensions and optimistic locking
- Create: `tests/mock_live_adapter_test.rs`
  - adapter lifecycle tests
- Modify: `tests/execution_kernel_test.rs`
  - non-final submit, fill delta, recovery, pending cancel
- Create: `tests/strategy_mock_live_run_test.rs`
  - end-to-end `strategy run --mode mock_live`
- Keep existing: `tests/strategy_paper_run_test.rs`
  - verify paper remains immediate-fill and unchanged

## Chunk 1: Public Model and Runtime Store

### Task 1: Extend shared execution models

**Files:**
- Modify: `src/execution/models.rs`
- Test: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Write failing model tests for new status and structs**

Add assertions for:

```rust
assert_eq!(OrderStatus::PendingCancel.as_str(), "pending_cancel");
assert_eq!(
    OrderStatus::from_str("pending_cancel"),
    Some(OrderStatus::PendingCancel)
);
```

Add typed-state roundtrip coverage using:

```rust
let state = MockLiveOrderState::default();
let json = serde_json::to_string(&state).unwrap();
let parsed: MockLiveOrderState = serde_json::from_str(&json).unwrap();
assert_eq!(parsed.unknown_retries, 0);
assert!(!parsed.recovery_exhausted);
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
```

Expected:
- compile failure because `PendingCancel` and `MockLiveOrderState` do not exist yet

- [ ] **Step 3: Extend `OrderStatus`, `OrderRecord`, and `RecoverySummary`**

Add at minimum:

```rust
pub enum OrderStatus {
    PendingSubmit,
    Submitted,
    Accepted,
    PartiallyFilled,
    PendingCancel,
    Filled,
    Canceled,
    Rejected,
    Unknown,
}
```

Extend `OrderRecord` with:

```rust
pub remaining_quantity: i64,
pub last_transition_at: DateTime<Utc>,
pub version: i64,
```

Extend `RecoverySummary` with:

```rust
pub unchanged: usize,
pub failed: usize,
pub skipped: usize,
```

- [ ] **Step 4: Add typed mock-live private state**

Introduce a typed state model in `src/execution/models.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MockLiveOrderState {
    pub fill_plan: Vec<MockLiveFillStep>,
    pub next_step_index: usize,
    pub planned_fill_time: Option<DateTime<Utc>>,
    pub fault_injection: Option<MockLiveFaultInjection>,
    pub unknown_until: Option<DateTime<Utc>>,
    pub cancel_requested: bool,
    pub last_applied_fill_id: u64,
    pub unknown_retries: u32,
    pub recovery_exhausted: bool,
    pub exhausted_reason: Option<String>,
}
```

Keep this backward-compatible with serde defaults.

- [ ] **Step 5: Run the targeted tests to verify they pass**

Run:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
```

Expected:
- the new model tests PASS
- store tests may still fail because schema/store code is not updated yet

- [ ] **Step 6: Commit**

```bash
git add src/execution/models.rs tests/execution_runtime_store_test.rs
git commit -m "feat: extend phase29c execution models"
```

### Task 2: Extend `orders` schema and add `mock_live_orders`

**Files:**
- Modify: `src/execution/runtime_store.rs`
- Test: `tests/execution_runtime_store_test.rs`

- [ ] **Step 1: Write failing store tests for schema shape**

Add tests that verify:

- `orders` rows persist `remaining_quantity`, `last_transition_at`, `version`
- `mock_live_orders` table exists
- mock-live private state can be inserted and read back

Suggested assertions:

```rust
assert!(store.has_table("mock_live_orders").await.unwrap());
assert_eq!(order.remaining_quantity, 100);
assert_eq!(order.version, 0);
```

- [ ] **Step 2: Run targeted store tests to verify failure**

Run:

```bash
cargo test --test execution_runtime_store_test schema -- --nocapture
```

Expected:
- failing assertions because the new fields/table do not exist yet

- [ ] **Step 3: Add schema migration-safe extensions**

In `ensure_schema()`:

- preserve existing `CREATE TABLE IF NOT EXISTS`
- add `ALTER TABLE`-style compatibility helpers for missing order columns
- create:

```sql
CREATE TABLE IF NOT EXISTS mock_live_orders (
    order_id TEXT PRIMARY KEY,
    adapter_order_id TEXT,
    state_json TEXT NOT NULL
);
```

Extend `orders` with:

```sql
remaining_quantity INTEGER NOT NULL DEFAULT 0
last_transition_at TEXT NOT NULL DEFAULT ''
version INTEGER NOT NULL DEFAULT 0
```

- [ ] **Step 4: Add typed mock-live store helpers**

Add helpers such as:

```rust
pub async fn insert_mock_live_order_state(...)
pub async fn get_mock_live_order_state(...)
pub async fn update_mock_live_order_state(...)
pub async fn list_recoverable_mock_live_orders(...)
pub async fn try_update_order_with_version(...)
```

`try_update_order_with_version(...)` should update only when `version = expected_version`, then increment `version` on success.

- [ ] **Step 5: Re-run store tests**

Run:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
```

Expected:
- schema and typed-state tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/execution/runtime_store.rs tests/execution_runtime_store_test.rs
git commit -m "feat: add phase29c mock live runtime store primitives"
```

## Chunk 2: Mock Live Adapter

### Task 3: Add adapter identity and mock-live adapter shell

**Files:**
- Modify: `src/execution/adapter.rs`
- Modify: `src/execution/mod.rs`
- Create: `src/execution/mock_live.rs`
- Test: `tests/mock_live_adapter_test.rs`

- [ ] **Step 1: Write failing adapter tests**

Create `tests/mock_live_adapter_test.rs` with failing tests covering:

- submit defaults to `Accepted`
- query can move `Accepted -> PartiallyFilled -> Filled`
- cancel moves into `PendingCancel`
- unknown can later recover

Use a fake clock and a temp runtime store.

- [ ] **Step 2: Run the adapter tests to confirm failure**

Run:

```bash
cargo test --test mock_live_adapter_test -- --nocapture
```

Expected:
- compile failure because `MockLiveExecutionAdapter` does not exist

- [ ] **Step 3: Add adapter identity API**

Extend `ExecutionAdapter`:

```rust
fn adapter_name(&self) -> &'static str;
```

Implement for `PaperExecutionAdapter`:

```rust
fn adapter_name(&self) -> &'static str { "paper" }
```

Export `mock_live` from `src/execution/mod.rs`.

- [ ] **Step 4: Implement mock-live shell and clock abstraction**

In `src/execution/mock_live.rs`, add:

```rust
pub trait MockLiveClock {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemMockLiveClock;
pub struct MockLiveExecutionAdapter<C> { ... }
```

`submit_order()` should:

- create shared/private state
- default to `Accepted`
- not touch paper-trade state

- [ ] **Step 5: Implement query/cancel lifecycle**

Implement:

- `query_order()`:
  - replay private state plan
  - return `PartiallyFilled`, `Filled`, `Unknown`, or unchanged state
- `cancel_order()`:
  - only allow `PendingSubmit | Submitted | Accepted | PartiallyFilled | Unknown`
  - first move to `PendingCancel`

- [ ] **Step 6: Run adapter tests**

Run:

```bash
cargo test --test mock_live_adapter_test -- --nocapture
```

Expected:
- adapter lifecycle tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/execution/adapter.rs src/execution/mod.rs src/execution/mock_live.rs tests/mock_live_adapter_test.rs
git commit -m "feat: add phase29c mock live execution adapter"
```

## Chunk 3: Kernel Non-Final States and Recovery

### Task 4: Teach `ExecutionKernel::execute_once()` non-final semantics

**Files:**
- Modify: `src/execution/kernel.rs`
- Modify: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Write failing kernel tests for non-final submit**

Add tests for:

- submit returns `Accepted` and order persists as non-final
- `sync_after_fill()` is triggered when filled quantity increases, not only on `Filled`
- adapter name comes from the adapter, not hardcoded `"paper"`

- [ ] **Step 2: Run targeted kernel tests to verify failure**

Run:

```bash
cargo test --test execution_kernel_test kernel_ -- --nocapture
```

Expected:
- failing assertions because kernel still assumes final submit semantics

- [ ] **Step 3: Replace hardcoded adapter identity**

Update order creation from:

```rust
adapter: "paper".to_string(),
```

to:

```rust
adapter: self.adapter.adapter_name().to_string(),
```

- [ ] **Step 4: Add fill-delta application helper**

Add an internal helper similar to:

```rust
async fn apply_fill_delta(
    &self,
    order_id: &str,
    old_filled_qty: i64,
    new_filled_qty: i64,
    fill_price: Option<Decimal>,
    fill_details: serde_json::Value,
) -> Result<()>
```

This helper should:

- compute delta fill quantity
- sync risk/account only when delta > 0
- append fill event details

- [ ] **Step 5: Update `execute_once()`**

Required behavior:

- persist `PendingSubmit`
- persist adapter-returned current status even if non-final
- update `remaining_quantity`, `last_transition_at`, and `version`
- call `apply_fill_delta(...)` when `filled_quantity` increases

- [ ] **Step 6: Re-run kernel tests**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
```

Expected:
- non-final submit tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/execution/kernel.rs tests/execution_kernel_test.rs
git commit -m "feat: support phase29c non-final execution kernel states"
```

### Task 5: Implement `recover_pending_orders()`

**Files:**
- Modify: `src/execution/kernel.rs`
- Modify: `tests/execution_kernel_test.rs`

- [ ] **Step 1: Write failing recovery tests**

Add tests for:

- recover scans `Submitted | Accepted | PartiallyFilled | Unknown | PendingCancel`
- version conflict causes one reload/retry, then skip
- `Unknown` retry exhaustion emits `recovery_exhausted` while status stays `Unknown`
- `submit -> unknown -> recover -> filled` works end to end

- [ ] **Step 2: Run targeted recovery tests**

Run:

```bash
cargo test --test execution_kernel_test recover_ -- --nocapture
```

Expected:
- current placeholder summary causes failures

- [ ] **Step 3: Implement recovery scan**

Implement `recover_pending_orders()` to:

- list recoverable mock-live orders
- read current `version`
- call adapter `query_order()`
- if changed, update via optimistic locking
- on conflict, reload once and retry
- if repeated conflict, increment `skipped`

- [ ] **Step 4: Add exhaustion semantics**

When repeated `Unknown` exceeds threshold:

- keep public order status as `Unknown`
- set private `recovery_exhausted=true`
- increment `unknown_retries`
- write `order_events.event_type = "recovery_exhausted"`

- [ ] **Step 5: Re-run kernel tests**

Run:

```bash
cargo test --test execution_kernel_test -- --nocapture
```

Expected:
- recovery tests PASS

- [ ] **Step 6: Commit**

```bash
git add src/execution/kernel.rs tests/execution_kernel_test.rs
git commit -m "feat: add phase29c pending order recovery"
```

## Chunk 4: CLI, Integration, and Documentation

### Task 6: Wire `strategy run --mode mock_live`

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `src/cli/tests/strategy.rs`
- Create: `tests/strategy_mock_live_run_test.rs`
- Keep: `tests/strategy_paper_run_test.rs`

- [ ] **Step 1: Write failing CLI parser and integration tests**

Add parser coverage:

```rust
let cli = Cli::try_parse_from([
    "quantix", "strategy", "run", "-n", "ma_cross", "--mode", "mock_live", "-c", "000001",
]).unwrap();
```

Add integration tests for:

- `mock_live` returns non-final status
- paper path still returns `Filled`
- runtime rows exist

- [ ] **Step 2: Run the new tests**

Run:

```bash
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
```

Expected:
- parser or handler failures because `mock_live` is not handled yet

- [ ] **Step 3: Update handler branching**

In `src/cli/handlers.rs`:

- keep `paper` path unchanged
- add `mock_live` branch using the new adapter
- keep `live` unsupported

Summary output must reflect current state truthfully:

```rust
signal=buy order_status=accepted
signal=buy order_status=partially_filled
signal=buy order_status=unknown
```

- [ ] **Step 4: Re-run CLI and integration tests**

Run:

```bash
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
```

Expected:
- all three test targets PASS

- [ ] **Step 5: Commit**

```bash
git add src/cli/handlers.rs src/cli/tests/strategy.rs tests/strategy_mock_live_run_test.rs tests/strategy_paper_run_test.rs
git commit -m "feat: wire phase29c mock live strategy run mode"
```

### Task 7: Update user-facing docs and hygiene checks

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write failing hygiene/doc tests if needed**

Add or update assertions so docs mention:

- `paper` remains immediate-fill
- `mock_live` adds lifecycle semantics
- `live` remains unsupported

- [ ] **Step 2: Run hygiene tests to verify failure**

Run:

```bash
cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- failing assertions until docs are updated

- [ ] **Step 3: Update docs**

Document:

- new `strategy run --mode mock_live`
- non-final statuses
- paper/mock-live/live boundary

- [ ] **Step 4: Run final targeted verification**

Run:

```bash
cargo test --test repo_hygiene_test -- --nocapture
cargo test --test mock_live_adapter_test -- --nocapture
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
```

Expected:
- all listed test targets PASS

- [ ] **Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: document phase29c mock live execution boundary"
```

## Final Verification

- [ ] Run focused regression set:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test mock_live_adapter_test -- --nocapture
cargo test --test execution_kernel_test -- --nocapture
cargo test --test strategy_mock_live_run_test -- --nocapture
cargo test --test strategy_paper_run_test -- --nocapture
cargo test --lib cli::tests::strategy:: -- --nocapture
cargo test --test repo_hygiene_test -- --nocapture
```

- [ ] If runtime or adapter changes spill into other execution surfaces, run:

```bash
cargo test --test execution_runtime_store_test -- --nocapture
cargo test --test strategy_daemon_test -- --nocapture
```

- [ ] Inspect changed scope:

```bash
git diff --stat
git diff --name-only
```

- [ ] Run GitNexus changed-scope review before final code handoff:

```bash
# Graphiti backfill required if graphiti-memory is unavailable in this client
```

Plan complete and saved to `docs/superpowers/plans/2026-03-22-phase29c-mock-live-execution-foundation-implementation.md`. Ready to execute?
