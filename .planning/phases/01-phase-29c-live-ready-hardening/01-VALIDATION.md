---
phase: 01
slug: phase-29c-live-ready-hardening
status: completed
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-12
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | none — workspace default cargo configuration |
| **Quick run command** | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test mock_live_adapter_test --test strategy_mock_live_run_test --test execution_kernel_test` |
| **Full suite command** | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --all-features` |
| **Estimated runtime** | ~120 seconds |

Linker note: on this machine the default `cargo test` linker path still fails while linking the `quantix` binary (`rust-lld ... unknown file type`), so Phase 1 verification currently uses `-fuse-ld=bfd`.

---

## Sampling Rate

- **After every task commit:** Run `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test mock_live_adapter_test --test strategy_mock_live_run_test --test execution_kernel_test`
- **After every plan wave:** Run `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --all-features`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | EXE-01 | integration | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test mock_live_adapter_test --test execution_kernel_test` | ✅ | ✅ green |
| 01-02-01 | 02 | 2 | EXE-02 | integration | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_kernel_test --test strategy_mock_live_run_test` | ✅ | ✅ green |
| 01-03-01 | 03 | 3 | EXE-03 | integration+docs | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test` + `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --lib 'cli::handlers::tests::test_format_strategy_request_detail_keeps_request_status_separate_from_order_status' -- --exact` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| README / USER_MANUAL live-ready wording is unambiguous to an operator | EXE-02, EXE-03 | Automated tests can assert strings but not communication clarity | Read the updated README and USER_MANUAL sections for `mock_live`/`live`; confirm they describe non-final statuses, recovery/reconciliation scaffolding, and explicitly avoid claiming real broker live execution |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
