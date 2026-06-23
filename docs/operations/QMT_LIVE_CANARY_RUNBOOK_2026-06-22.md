# qmt_live Canary Runbook

Date: 2026-06-22

Scope: first controlled qmt_live canary after P0.5a preflight is available.

## Purpose

This runbook defines the minimum operator sequence for a real-money qmt_live canary. It is an operations document only. It does not authorize broader real-money usage and does not change any runtime behavior.

The canary is allowed only when the operator can produce the redacted evidence artifact described in `docs/reports/evidence/qmt-live-canary-20260622/`.

## Hard Boundaries

- Use qmt_live only; generic live broker mode is still not implemented.
- Treat miniQMT as the broker source of truth.
- Do not submit if preflight is not `ready`.
- Do not submit if the preview payload is not understood by the operator.
- Do not submit if the kill switch is enabled, unless the operator is explicitly testing the blocked path and will not send a live order.
- Do not commit secrets, raw credential material, full account identifiers, or raw broker logs.
- Do not retry a qmt_live submission when identity, query, reconciliation, or manual-intervention state is ambiguous.

## Kill Switch Operating Rule

The kill switch is mandatory operational equipment for every qmt_live canary.

Before the canary, the operator must verify that the kill switch state is visible and that the enable command is available for immediate incident response. This pre-canary check does not mean live submission is allowed while the switch is enabled.

For a real qmt_live submission, the kill switch must be observed as disabled immediately before `quantix execution qmt live --request-id <ID> --yes`.

For incident response, enabling the kill switch is the first containment action when bridge behavior, miniQMT state, identity reconciliation, external order identity, or broker status becomes ambiguous. After enabling it, read-only qmt_live status/checklist/preflight, preview, query, reconciliation, and manual-intervention review remain available for investigation.

## Required Inputs

- Quantix commit hash under test.
- Windows Bridge base URL and non-secret environment label.
- miniQMT account label, redacted before evidence is committed.
- A single approved qmt_live request ID.
- Operator name or initials for local audit records.
- Kill switch operator path and current state observed before submission.
- Evidence directory:

```text
docs/reports/evidence/qmt-live-canary-20260622/
```

## Canary Sequence

### 1. start Windows Bridge

Start the Windows Bridge process on the Windows host that can reach miniQMT.

Record in evidence:

- bridge host label, not a secret endpoint;
- bridge startup timestamp;
- bridge process/version note if available.

### 2. confirm miniQMT login

Open miniQMT and confirm the target account is logged in and can display same-day orders.

Record in evidence:

- redacted account label;
- login confirmation timestamp;
- operator confirmation that order/query/cancel panels are visible.

### 3. run qmt_live preflight

Run:

```bash
quantix execution qmt status --checklist
```

Required result:

- `qmt_live_preflight.ready=true`
- `failure_category=null` or `failure_category=none`
- bridge reachable;
- `qmt.enabled=true`;
- `qmt.mode=live`;
- `order_submit_supported=true`;
- kill switch disabled.

Save a redacted readiness summary in the evidence artifact. Do not paste secrets or full endpoint details.

### 4. run preview

Run:

```bash
quantix execution qmt preview --request-id <ID>
```

### 5. verify preview payload

Review the preview output before any submission.

The operator must verify:

- symbol;
- side;
- quantity;
- order type;
- price or pricing policy;
- target account label;
- generated client order identity;
- no unexpected broker payload fields.

Record only a preview payload hash and a redacted summary in the evidence artifact.

### 6. confirm kill switch status

Run or inspect the configured kill-switch status before submission.

The evidence must record:

- kill switch enabled/disabled state;
- reason if enabled;
- timestamp observed.
- operator confirmation that the kill switch can be enabled immediately if incident response is needed.

If the kill switch is enabled, stop here unless this is a blocked-path drill.

### 7. record operator confirmation

Before sending a real order, the operator must explicitly record confirmation in the evidence artifact:

```text
operator confirmation: qmt_live canary submission approved for request <ID>
```

The confirmation must include:

- operator label;
- confirmation timestamp;
- reviewed preview hash;
- statement that the operator understands this is real-money qmt_live submission.

### 8. submit only with explicit operator confirmation

Submit only after the previous step is recorded.

Run:

```bash
quantix execution qmt live --request-id <ID> --yes
```

Stop immediately if the command returns an error, identity mismatch, broker unknown state, missing external order ID, or manual-intervention marker.

### 9. run qmt_live query

Run:

```bash
quantix execution qmt query <ID>
```

Record a redacted query summary:

- request ID;
- local submission ID;
- client order ID;
- task ID;
- external order ID when present;
- latest status;
- rejection or unknown-state summary when present.

### 10. run reconciliation verification

Run the current qmt_live reconciliation verification path for the request or order under test.

Record:

- reconciliation command line;
- reconciliation decision;
- status transition, if any;
- identity fields used for lookup;
- whether local state was preserved due to uncertainty.

### 11. record manual-intervention status

Before closing the canary evidence, record manual-intervention status.

Use one of:

- `none`;
- `identity_mismatch`;
- `broker_unknown_state`;
- `missing_external_order_id`;
- `bridge_failure`;
- `reconciliation_preserved_local_state`;
- `operator_review_required`.

If status is not `none`, do not submit another qmt_live order until the ambiguity is resolved against miniQMT same-day orders.

## Evidence Closure

Save the completed redacted artifact as:

```text
docs/reports/evidence/qmt-live-canary-20260622/evidence.<YYYYMMDD-HHMMSS>.json
```

The template is:

```text
docs/reports/evidence/qmt-live-canary-20260622/evidence.template.json
```

The artifact must be reviewed before commit to confirm it contains no secrets, no full account identifiers, and no raw broker logs.

## Stop Conditions

Stop the canary and preserve evidence if any of these occur:

- preflight is not ready;
- preview payload does not match operator intent;
- operator confirmation is missing;
- live submission returns bridge failure;
- qmt_live task identity mismatches local identity;
- broker state is unknown;
- external order ID is missing after a completed bridge task;
- reconciliation preserves local state due to ambiguity;
- miniQMT same-day orders do not match Quantix local state.
