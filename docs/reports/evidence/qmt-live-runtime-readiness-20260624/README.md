# qmt_live Runtime Readiness Evidence Directory

This directory stores redacted P0.6 runtime-readiness evidence:

```text
docs/reports/evidence/qmt-live-runtime-readiness-20260624/
```

Use `evidence.template.json` for the consolidated evidence package shape. Completed artifacts should be named:

```text
evidence.<YYYYMMDD-HHMMSS>.json
```

P0.6 runtime-readiness evidence is allowed to record both successful read-only checks and fail-closed blocked checks. A blocked artifact is valid when it clearly states the missing operator-selected runtime, command target, or safety prerequisite.

## Commit Policy

Artifacts committed here must be redacted.

Allowed:

- command lines that contain no secrets;
- commit hashes and branch names;
- redacted runtime labels;
- redacted bridge host labels;
- redacted account labels;
- summarized qmt_live read-only command outcomes;
- blocked or deferred command reasons;
- mutation-guard attestations;
- operator review notes that contain no personal or account-identifying data.

Forbidden:

- secrets, credentials, API keys, bearer values, or environment file contents;
- raw account IDs or account names tied to a real person;
- raw bridge URLs or connection strings;
- raw broker logs;
- screenshots containing broker/account identifiers;
- process command lines that expose endpoint or account details;
- raw bridge or broker payloads that contain account identifiers or credentials.

## Mutation Boundary

No submit/cancel or no broker-state mutation is allowed for P0.6 runtime-readiness evidence unless a later, separately approved canary plan explicitly authorizes it.

For P0.6, evidence may include:

- local CLI version and safety status;
- qmt_live status/checklist output summaries;
- preview output summaries when no submission is made;
- query/audit/manual-intervention read-only summaries when safe targets exist;
- explicit skipped-command records when no selected runtime or query target exists.

Evidence must not include:

- `quantix execution qmt live --request-id <ID> --yes` execution;
- broker cancel execution;
- manual-intervention resolution execution;
- runtime-store mutation performed only for evidence collection;
- any broker-side state mutation.

