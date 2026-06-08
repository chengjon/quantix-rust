# Triage Labels

Matt Pocock skills use canonical roles internally. Map them to the GitHub labels below when triaging issues.

## Category Roles

Each triaged issue should have exactly one category role.

| Canonical role | GitHub label |
|---|---|
| `bug` | `type: bug` |
| `enhancement` | `type: enhancement` |

## State Roles

Each triaged issue should have exactly one state role.

| Canonical role | GitHub label |
|---|---|
| `needs-triage` | `status: needs-triage` |
| `needs-info` | `status: needs-info` |
| `ready-for-agent` | `status: ready-for-agent` |
| `ready-for-human` | `status: ready-for-human` |
| `wontfix` | `status: wontfix` |

## Current Code-Audit Defaults

Use these defaults for the current audit finding set:

| Finding type | Category | Initial state |
|---|---|---|
| Failing gate with known file/test | `bug` | `ready-for-agent` |
| Failing or ambiguous gate needing root-cause diagnosis | `bug` | `ready-for-agent` |
| Quality-bar decision, such as clippy warning policy | `enhancement` | `ready-for-human` |
| Audit-spec hardening with clear acceptance criteria | `enhancement` | `ready-for-agent` |
| Architecture refactor candidate without approved design | `enhancement` | `needs-triage` |

If the labels do not exist on GitHub yet, ask the maintainer before creating or applying labels.

## Current Tracker State

As of 2026-06-07, all configured `type:*` and `status:*` labels are present in `chengjon/quantix-rust`. Apply labels normally when triaging.
