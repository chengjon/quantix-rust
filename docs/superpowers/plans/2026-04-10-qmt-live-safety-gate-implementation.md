# QMT Live Safety Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent any real QMT order submission unless the Windows bridge explicitly reports `qmt.mode == "live"`.

**Architecture:** Keep the existing public contract unchanged. Do not wire `target_mode = live` yet. Instead, add a small capability gate in the real-order paths only: the QMT live adapter path and the manual `execution bridge qmt-live` CLI path. Preview behavior stays unchanged.

**Tech Stack:** Rust, Tokio, reqwest, wiremock, cargo test, GitNexus impact/detect_changes

---

### Task 1: Guard The Adapter Path

**Files:**
- Modify: `src/execution/qmt_live_adapter.rs`
- Test: `tests/qmt_live_adapter_test.rs`
- Reference: `src/bridge/client.rs`
- Reference: `src/bridge/models.rs`

- [ ] **Step 1: Write the failing test for preview-only rejection**

Add a test to `tests/qmt_live_adapter_test.rs` that:

- serves `/api/v1/capabilities` with:

```json
{
  "tdx": { "enabled": true, "supports": ["quote"] },
  "qmt": { "enabled": true, "mode": "preview_only", "supports": ["order_preview"] }
}
```

- does **not** register a successful expectation for `POST /api/v1/broker/qmt/orders`
- calls `QmtLiveExecutionAdapter.submit_order(...)`
- asserts the returned error mentions that QMT live submission is blocked in `preview_only`

- [ ] **Step 2: Run the new adapter test to verify RED**

Run: `cargo test --test qmt_live_adapter_test -- --nocapture`

Expected:
- FAIL because the adapter currently submits without checking capabilities first

- [ ] **Step 3: Implement the minimal adapter gate**

In `src/execution/qmt_live_adapter.rs`:

- fetch `self.client.capabilities().await`
- reject unless:
  - `capabilities.qmt.enabled`
  - `capabilities.qmt.mode == "live"`

Keep the error local and explicit, for example:

```rust
return Err(AdapterError::Execution(format!(
    "QMT live submission blocked: bridge qmt.mode={} (expected live)",
    capabilities.qmt.mode
)));
```

- [ ] **Step 4: Run the adapter test to verify GREEN**

Run: `cargo test --test qmt_live_adapter_test -- --nocapture`

Expected:
- PASS

- [ ] **Step 5: Add the positive live-mode test**

Extend `tests/qmt_live_adapter_test.rs` with a second test that:

- serves `qmt.mode = "live"` from `/api/v1/capabilities`
- serves success from `POST /api/v1/broker/qmt/orders`
- asserts the adapter returns the expected order response

- [ ] **Step 6: Run the adapter test suite again**

Run: `cargo test --test qmt_live_adapter_test -- --nocapture`

Expected:
- PASS

- [ ] **Step 7: Commit the adapter slice**

```bash
git add tests/qmt_live_adapter_test.rs src/execution/qmt_live_adapter.rs
git commit -m "fix(execution): gate qmt live adapter on bridge live mode"
```

### Task 2: Guard The Manual `qmt-live` CLI Path

**Files:**
- Modify: `src/cli/handlers/mod.rs`
- Test: `src/cli/handlers/tests/mod.rs`
- Reference: `src/bridge/client.rs`

- [ ] **Step 1: Run GitNexus impact for the CLI bridge live entrypoint**

Run:

```bash
gitnexus impact --target execute_execution_bridge_qmt_live --direction upstream
```

Expected:
- capture direct callers and risk before editing the CLI live bridge entrypoint

- [ ] **Step 2: Write the failing CLI test for preview-only rejection**

Add a test near the execution bridge handler coverage in `src/cli/handlers/tests/mod.rs` that:

- stubs bridge capabilities to return `qmt.mode = "preview_only"`
- invokes the manual `qmt-live` path with confirmation bypass
- asserts:
  - the command returns an error
  - no real order submission is attempted

- [ ] **Step 3: Run the targeted handler test to verify RED**

Run the smallest matching test target, for example:

```bash
cargo test qmt_live -- --nocapture
```

Expected:
- FAIL because the current CLI path submits directly

- [ ] **Step 4: Implement the minimal CLI gate**

In `src/cli/handlers/mod.rs` inside `execute_execution_bridge_qmt_live(...)`:

- call `client.capabilities().await` before building or submitting the live order
- reject unless `qmt.enabled == true` and `qmt.mode == "live"`
- include the observed mode in the error message

- [ ] **Step 5: Re-run the targeted handler test to verify GREEN**

Run:

```bash
cargo test qmt_live -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Re-run the preview-path coverage**

Run:

```bash
cargo test --test qmt_bridge_preview_test -- --nocapture
```

Expected:
- PASS, proving preview behavior still works under preview-only mode

- [ ] **Step 7: Commit the CLI slice**

```bash
git add src/cli/handlers/mod.rs src/cli/handlers/tests/mod.rs tests/qmt_bridge_preview_test.rs
git commit -m "fix(cli): block qmt live submit unless bridge is in live mode"
```

### Task 3: Verify Scope And Document The Boundary

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Test: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing doc expectation if wording needs to change**

If current docs do not explicitly say that real submission requires bridge live mode, add the matching expectation in `tests/repo_hygiene_test.rs`.

- [ ] **Step 2: Run the doc hygiene test to verify RED**

Run:

```bash
cargo test --test repo_hygiene_test -- --test-threads=1
```

Expected:
- FAIL only if the new wording is not yet documented

- [ ] **Step 3: Update docs with the safety-gate rule**

Add one concise line to `README.md` and `docs/USER_MANUAL.md`:

- QMT preview remains available in `preview_only`
- real QMT submission requires the bridge to report `qmt.mode=live`

- [ ] **Step 4: Re-run the doc hygiene test to verify GREEN**

Run:

```bash
cargo test --test repo_hygiene_test -- --test-threads=1
```

Expected:
- PASS

- [ ] **Step 5: Run change detection and commit**

Run:

```bash
gitnexus detect_changes --scope all
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs
git commit -m "docs: clarify qmt live submission safety gate"
```

### Task 4: Final Verification

**Files:**
- Verify touched files only

- [ ] **Step 1: Run focused execution tests**

Run:

```bash
cargo test --test qmt_live_adapter_test -- --nocapture
cargo test --test qmt_bridge_preview_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 2: Run targeted handler coverage**

Run:

```bash
cargo test qmt_live -- --nocapture
```

Expected:
- PASS

- [ ] **Step 3: Run doc hygiene**

Run:

```bash
cargo test --test repo_hygiene_test -- --test-threads=1
```

Expected:
- PASS

- [ ] **Step 4: Run GitNexus change detection**

Run:

```bash
gitnexus detect_changes --scope all
```

Expected:
- only the QMT live adapter, CLI live bridge path, and any matching docs/tests are affected

- [ ] **Step 5: Record Graphiti outcome**

Write a concise summary to:

- `quantix_rust_main` for the execution safety decision
- `quantix_rust_docs` if doc wording changed
