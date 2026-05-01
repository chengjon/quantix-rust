---
phase: 03
slug: real-live-broker-execution-closure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-12
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | none — workspace default cargo configuration |
| **Quick run command** | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test qmt_live_gate_test --test qmt_live_adapter_test --test execution_daemon_test --test repo_hygiene_test` |
| **Full suite command** | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --all-features` |
| **Estimated runtime** | ~150 seconds |

Linker note: on this machine the default `cargo test` linker path still fails while linking the `quantix` binary (`rust-lld ... unknown file type`), so Phase 3 verification should continue using `-fuse-ld=bfd`.

---

## Sampling Rate

- **After every task commit:** Run `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test qmt_live_gate_test --test qmt_live_adapter_test --test execution_daemon_test --test repo_hygiene_test`
- **After every plan wave:** Run `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --all-features`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | LIV-01 | integration | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test qmt_live_gate_test --test qmt_live_adapter_test` | ✅ | ⬜ pending |
| 03-02-01 | 02 | 2 | LIV-02 | integration | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_daemon_test --lib qmt_live` | ✅ | ⬜ pending |
| 03-03-01 | 03 | 3 | LIV-03 | integration+docs | `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Operator can tell that only `qmt_live` is the guarded real-submit path | LIV-01 | Automated tests can lock strings but not operator judgment | Read the final daemon, approval, and manual bridge messages; confirm generic `live` still reads as unsupported and redirected to `qmt_live` |
| Misconfigured bridge state fails before real submission with actionable guidance | LIV-02 | Tests prove rejection, but manual review judges operator clarity | Read failure output and persisted request diagnostics for preview-only / disabled / unsupported capability scenarios |
| Real-submit flow exposes the minimum safe verification loop | LIV-03 | Manual review is needed to judge whether the operator can follow the workflow | Read the final docs plus `execution bridge qmt-live` output; confirm they mention preconditions, confirmation semantics, and the broker-side follow-up query step |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
