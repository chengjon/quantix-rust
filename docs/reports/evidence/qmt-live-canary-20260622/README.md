# qmt_live Canary Evidence Directory

This directory is the P0.5b evidence artifact location for the first controlled qmt_live canary:

```text
docs/reports/evidence/qmt-live-canary-20260622/
```

Use `evidence.template.json` as the redacted artifact shape. Completed artifacts should be named:

```text
evidence.<YYYYMMDD-HHMMSS>.json
```

## Commit Policy

Artifacts committed here must be redacted.

Allowed:

- command lines without secrets;
- redacted environment labels;
- redacted account labels;
- readiness summaries;
- preview payload hashes;
- submission/query/reconciliation summaries;
- manual-intervention status.

Forbidden:

- secrets;
- raw credential material;
- full account identifiers;
- raw broker logs;
- raw bridge payloads that include account identifiers or credentials.

If a canary is blocked before submission, still save an artifact with `canary_status=blocked` and the fail-closed reason.
