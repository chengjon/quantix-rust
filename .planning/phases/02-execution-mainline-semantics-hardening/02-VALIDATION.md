---
phase: 02
slug: execution-mainline-semantics-hardening
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-12
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | none — workspace default cargo configuration |
| **Quick run command** | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_daemon_test --test execution_runtime_store_test --test repo_hygiene_test` |
| **Full suite command** | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --all-features` |
| **Estimated runtime** | ~150 seconds |

Linker note: on this machine the default `cargo test` linker path still fails while linking the `quantix` binary (`rust-lld ... unknown file type`), so Phase 2 verification should continue using `-fuse-ld=bfd`.

---

## Sampling Rate

- **After every task commit:** Run `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_daemon_test --test execution_runtime_store_test --test repo_hygiene_test`
- **After every plan wave:** Run `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --all-features`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | SEM-01 | unit | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --lib 'cli::handlers::tests::test_format_strategy_request_detail_keeps_request_status_separate_from_order_status' -- --exact` | ✅ | ✅ green |
| 02-02-01 | 02 | 2 | SEM-02 | integration | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_daemon_test --test execution_runtime_store_test` | ✅ | ✅ green |
| 02-03-01 | 03 | 3 | SEM-03 | integration+docs | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Operator can read request output without confusing request completion and order terminality | SEM-01 | Automated tests can lock strings but not operator comprehension | Read the final `strategy request list/show` and daemon summary output examples; confirm they present request status and order status as separate truths |
| Failed or stuck requests expose enough reason/timestamp context for triage | SEM-02 | Manual review is needed to judge diagnostic sufficiency | Read the final request detail and daemon summary output for failed and `in_progress` scenarios; confirm an operator can infer the last executor action and reason |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
