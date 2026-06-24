# qmt_live Runtime Readiness P0.6a

Date: 2026-06-24

Status: environment inventory recorded; runtime smoke blocked by environment selection

Branch: `feat/p0-6a-qmt-live-runtime-inventory`

Base commit: `9846390ccd4ca20a7b4f5887b15f07728a26562a`

Evidence: `docs/reports/evidence/qmt-live-runtime-readiness-20260624/environment-inventory.json`

## Summary

P0.6a records the current qmt_live runtime inventory and prerequisite state for the active P0.6 OpenSpec change.

The local workspace has the expected Windows-side project directories:

| Item | Status |
| --- | --- |
| Canonical Windows Bridge path `/mnt/d/mystocks/quantix/quantix_bridge` | present |
| miniQMT workspace path `/mnt/d/MyCode3/miniQMT` | present |
| `target/debug/quantix` | present |
| `quantix --version` | `quantix 0.1.0` |
| qmt/bridge process observed in this session | no |
| selected qmt bridge endpoint/config | no |
| selected miniQMT account label | no |
| local kill switch status | `enabled=false` |

Because no operator-confirmed bridge endpoint/config or account label was selected and no qmt/bridge process was observed, P0.6a does not claim runtime readiness. P0.6b read-only smoke remains blocked until an operator starts or selects a test-owned miniQMT/Windows Bridge runtime and provides commit-safe labels.

## Evidence Captured

The committed evidence file records only derived, redacted facts:

- OS/runtime label.
- Commit hash and branch.
- Presence of canonical bridge and miniQMT paths.
- Presence of Quantix binary and version.
- Absence of matching qmt/bridge process in this session.
- Absence of selected qmt bridge environment/config.
- Local kill-switch status from `target/debug/quantix safety kill-switch status`.
- Redaction guarantees.

The evidence does not include:

- raw account IDs;
- account names tied to a real person;
- credentials, API keys, tokens, or `.env` contents;
- raw bridge URLs;
- raw broker logs;
- screenshots;
- process command-line details.

## Commands

Executed read-only probes only:

```text
environment_probe
target/debug/quantix --version
target/debug/quantix safety kill-switch --help
target/debug/quantix safety kill-switch status
```

No `execution qmt live`, broker cancel, manual-intervention resolution, runtime-store write, or bridge mutation command was executed.

## P0.6a Task Mapping

| Task | Result |
| --- | --- |
| Identify operator-owned miniQMT/Windows Bridge runtime, or record none | Recorded: directories present, runtime endpoint/account not selected |
| Capture redacted runtime metadata | Completed in evidence JSON |
| Store commit-safe evidence | Completed |
| If runtime unavailable, stop and prepare blocked readiness report | Completed: P0.6b is blocked by environment selection |

## Decision

P0.6a result:

```text
blocked_by_environment_selection
```

This is not a system defect. It is a correct fail-closed operational state: runtime assets are present, but no operator-selected, running, test-owned qmt_live endpoint was confirmed for this session.

## Required Next Action

Before P0.6b read-only smoke, an operator must provide:

- a running Windows Bridge process;
- a commit-safe bridge host label, not a raw credential-bearing URL;
- a test-owned miniQMT account label, not a raw account ID;
- confirmation that read-only inspection is allowed for the selected runtime;
- kill-switch expectation for the smoke session.

Only after that can P0.6b attempt read-only qmt commands such as status checklist, preview dry path, query, audit, and manual-intervention listing.

## Boundaries Preserved

P0.6a did not change runtime code, tests, bridge protocol, storage schema, response shapes, `OrderStatus`, `ExecutionAdapter`, qmt_live submit/query/cancel behavior, paper execution semantics, or any `.unwrap()` technical-debt item.
