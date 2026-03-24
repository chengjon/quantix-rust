# Phase 29B Strategy Signal Daemon Design

**Date:** 2026-03-18
**Status:** Approved in-session
**Depends On:** Current green baseline (`master` @ `1ce85bf`)

> This document is the source of truth for the next strategy slice: split long-running strategy evaluation from execution, introduce durable signal and execution-request objects, and deliver a WSL2-friendly strategy daemon that does not auto-trade.

---

## Goal

Build the smallest useful strategy daemon foundation that:

1. continuously evaluates configured strategies when a new daily bar appears
2. persists strategy signals as first-class records with explicit lifecycle state
3. requires human approval before any signal can enter the execution path
4. creates execution requests without actually consuming or executing them
5. keeps strategy evaluation, approval, and execution clearly separated

Phase 29B must not auto-trade, consume execution requests, or rebind the existing Phase 29A direct paper execution path.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. initialize a strategy-daemon configuration file
2. configure one code with multiple enabled strategy instances
3. run a long-lived strategy daemon in the foreground or under `systemd --user`
4. automatically generate a new signal when a new daily bar appears
5. review pending signals
6. approve or reject a signal manually
7. see an approved signal become a durable `execution_request`

### Exact boundary

Phase 29B implements a new signal-oriented path. It does not replace the existing direct execution path.

It must add:

```bash
quantix strategy config init
quantix strategy config show
quantix strategy daemon run
quantix strategy daemon run --once
quantix strategy signal list
quantix strategy signal approve --signal-id <ID> --target-mode paper --target-account default
quantix strategy signal reject --signal-id <ID> [--reason <TEXT>]
quantix strategy request list
quantix strategy service install|uninstall|start|stop|status|enable|disable
quantix strategy service-config show
quantix strategy service-config set --quantix-bin <ABS_PATH> [--env-file <ABS_PATH>]
```

Rules:

- daemon scope is `single code + multiple strategy instances`
- strategies only emit signals in this phase
- approval creates `execution_request` rows only
- `execution_request` rows are not executed in this phase
- `strategy run --mode paper` remains available and unchanged as the direct Phase 29A path
- strategy daemon does not require `trade init`

### Explicitly deferred

Phase 29B does not include:

- automatic trading from the daemon
- execution-request consumption
- auto-approval policies
- multi-code scheduling
- live adapters
- partial fills or `Unknown` order injection
- HTTP health endpoints
- historical signal backfill
- replay from the beginning of available bars
- strong cross-process write coordination beyond the current single-writer assumption

## Approaches Considered

### Option A: Signal store only

Add a daemon that writes durable signals and supports manual review, but stop there.

Pros:

- smallest signal-layer delivery
- low implementation risk
- clean strategy/execution separation

Cons:

- approved signals still have no explicit handoff object
- later automation must add another storage seam

### Option B: Signal store plus execution-request stub

Add a daemon that produces signals, manual approval commands, and a durable execution-request table that future execution workers can consume.

Pros:

- preserves the clean strategy/execution split
- gives signals a durable approval and handoff boundary now
- avoids reworking signal storage when execution automation arrives

Cons:

- larger schema than Option A
- requires approval semantics now

### Option C: Signal store plus policy approval and automatic request creation

Add a daemon, approval policies, and automatic promotion from signal to execution request.

Pros:

- closest to future full automation
- fewer future model changes

Cons:

- too much scope for the current slice
- introduces policy complexity before the manual path is proven
- makes it easier to blur the line between evaluation and execution

## Recommendation

Choose **Option B**.

Phase 29B should make `signal` and `execution_request` first-class objects now, but keep approval manual and keep execution out of scope.

This creates the durable middle seam required for future automation without prematurely building an execution daemon or policy engine.

## Architecture

### System roles

Phase 29B splits responsibilities into four roles:

1. `strategy init`
   - prepares strategy-daemon configuration and stable local paths
   - does not emit signals
   - does not execute trades

2. `strategy daemon`
   - monitors for new daily bars
   - evaluates configured strategy instances
   - writes run and signal audit rows

3. `signal approval`
   - manually approves or rejects pending signals
   - approval is the only path into `execution_request`

4. `execution request`
   - durable queue-like handoff object for future consumers
   - created by approval
   - not consumed in this phase

The resulting chain is:

`init -> strategy daemon -> signal -> manual approve/reject -> execution_request`

### Runtime identity

The scheduling unit is:

`strategy_instance_id + symbol + timeframe`

The daemon must not key state only by `strategy_name`, because future configurations may run multiple differently-parameterized instances of the same strategy type.

### Data ownership

- ClickHouse daily bars
  - source of market data for daemon evaluation

- `runtime.db`
  - source of truth for daemon runs, signals, execution requests, and daemon checkpoints

- `paper_trade.json`
  - remains the source of truth for Phase 29A direct paper execution only

- `risk_state.json`
  - remains the source of truth for Phase 29A direct paper execution only

Phase 29B does not read or mutate trade or risk stores during daemon evaluation.

## Configuration Model

### Storage format

Use JSON, not TOML.

Reason:

- the repository already uses JSON for peer local configuration stores such as monitor config and service config
- keeping strategy daemon config in JSON preserves local-state consistency
- the internal model remains format-agnostic if TOML import/export is desired later

### File path

Default config path:

`~/.quantix/strategy/config.json`

### Shape

```json
{
  "check_interval_secs": 60,
  "bootstrap_policy": "latest_only",
  "stocks": [
    {
      "code": "000001",
      "enabled": true,
      "strategies": [
        {
          "id": "ma_fast_5_slow_20",
          "name": "ma_cross",
          "enabled": true,
          "params": { "fast": 5, "slow": 20 }
        }
      ]
    }
  ]
}
```

Rules:

- Phase 29B validates that exactly one stock is enabled
- multiple enabled strategy instances under that stock are allowed
- `bootstrap_policy` is required in the model but only `latest_only` is implemented in this phase

### Hot reload

The daemon must hot reload configuration using file modification time checks:

1. record last observed config `mtime`
2. before each loop, re-check `mtime`
3. if changed, reload config and rebuild the active instance set

No filesystem event watcher is required in this phase.

## Strategy Registry

### Registry boundary

Add a strategy registry that resolves configured strategies by name and parameters into evaluators.

Recommended interface:

```rust
trait ConfiguredStrategyEvaluator {
    fn lookback_required(&self) -> usize;
    fn evaluate(&self, klines: &[Kline]) -> Result<SignalEnvelope>;
}
```

Rules:

- `lookback_required()` returns the minimum historical window required to evaluate the latest bar
- `evaluate()` returns only a signal envelope, not an order or execution decision
- the daemon core is strategy-type agnostic once it receives a configured evaluator

### Phase 29B support level

Phase 29B must fully support:

- multiple configured instances of `ma_cross`

Other strategies may be registered later without changing daemon orchestration.

## Runtime Database Model

### Existing tables

Keep existing runtime tables in place:

- `strategy_runs`
- `signal_events`
- `orders`
- `order_events`
- existing Phase 29A checkpoint table

Phase 29B must not delete or repurpose them.

### New tables

Add new daemon-oriented tables to the same `runtime.db`.

#### `signals`

```sql
CREATE TABLE IF NOT EXISTS signals (
    signal_id TEXT PRIMARY KEY,
    strategy_instance_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    bar_end TEXT NOT NULL,
    signal_value TEXT NOT NULL,
    signal_status TEXT NOT NULL DEFAULT 'new',
    approval_status TEXT NOT NULL DEFAULT 'pending',
    run_id TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

Constraints:

- `UNIQUE(strategy_instance_id, symbol, timeframe, bar_end)`

Indexes:

- `(symbol, bar_end)`
- `approval_status`
- `(strategy_instance_id, approval_status)`
- `(strategy_instance_id, signal_status)`

#### `execution_requests`

```sql
CREATE TABLE IF NOT EXISTS execution_requests (
    request_id TEXT PRIMARY KEY,
    signal_id TEXT NOT NULL UNIQUE,
    target_mode TEXT NOT NULL,
    target_account TEXT NOT NULL,
    request_status TEXT NOT NULL DEFAULT 'pending',
    approved_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    payload_json TEXT NOT NULL
);
```

Indexes:

- `request_status`
- `(target_mode, request_status)`

Relationship rules:

- one signal can create at most one execution request
- Phase 29B only inserts `pending`
- later phases may transition requests to `completed`, `failed`, or `canceled`

#### `strategy_daemon_checkpoints`

Add a daemon-specific checkpoint table rather than mutating the Phase 29A checkpoint contract.

```sql
CREATE TABLE IF NOT EXISTS strategy_daemon_checkpoints (
    checkpoint_id TEXT PRIMARY KEY,
    strategy_instance_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    last_processed_bar TEXT,
    last_run_id TEXT,
    state_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

Constraint:

- `UNIQUE(strategy_instance_id, symbol, timeframe)`

## Time Model

### Bar identity

For daily bars:

- interpret market close as `Asia/Shanghai 15:00:00`
- convert that timestamp to UTC before storing it in `bar_end`

Rules:

- dedupe and checkpoint comparisons use the normalized UTC `bar_end`
- Phase 29B does not collapse `bar_end` to a date-only surrogate
- future lower timeframes can reuse the same identity model

### Initial bootstrap policy

Phase 29B only implements:

- `bootstrap_policy = latest_only`

Semantics:

1. if a daemon instance has no checkpoint
2. fetch the latest available bar for that stream
3. write a checkpoint at that bar
4. emit no signal
5. create no execution request

If no market data exists for that stream:

- log a warning
- create no checkpoint
- retry on later loops

This keeps the daemon aligned with ongoing monitoring semantics rather than historical replay.

## Daemon Loop

### Startup

`strategy daemon run` must:

1. load config
2. open `runtime.db` and ensure schema
3. build the active configured-strategy registry
4. load checkpoints for each active stream

### Per-loop flow

For each active stream:

1. load enough bars for `lookback_required()`
2. compute normalized `latest_bar_end`
3. compare with checkpoint
4. if `latest_bar_end <= last_processed_bar`, skip
5. if no checkpoint exists, bootstrap and skip signal creation
6. if a new bar exists:
   - evaluate the strategy
   - open a runtime-db transaction
   - insert `strategy_run`
   - insert new `signal`
   - mark older active signals on the same stream as `superseded`
   - cancel still-pending execution requests associated with those superseded signals
   - update checkpoint to the new bar
   - commit

ClickHouse reads and strategy evaluation remain outside the SQLite transaction.

### Error handling

- ClickHouse transient failure
  - log `warn`
  - keep daemon alive
  - retry next loop with simple backoff

- single strategy-instance evaluation failure
  - mark the run failed
  - log the error
  - continue other active instances

- runtime-db failure
  - surface as a loop error
  - keep daemon alive unless startup schema creation failed

The daemon must support graceful shutdown on `SIGTERM` and `SIGINT` by finishing the current iteration and then exiting.

## Signal Lifecycle

### `signal_status`

Phase 29B uses:

- `new`
  - current active signal for its stream
- `superseded`
  - replaced by a newer signal on the same stream
- `expired`
  - reserved for later cleanup or TTL logic

### `approval_status`

Phase 29B uses:

- `pending`
- `approved`
- `rejected`

### State rules

- new signal rows are inserted as `new + pending`
- approved signals remain `signal_status='new'`
- rejected signals remain `signal_status='new'`
- when a later signal is generated on the same stream:
  - older `new` signals become `superseded`
  - associated `pending` execution requests become `canceled`

Historical rows are always retained for auditability.

## Approval and Request Creation

### Approve transaction

Approval must be atomic within SQLite:

1. begin transaction
2. conditional update:
   - only succeed if `signal_status='new' AND approval_status='pending'`
3. verify exactly one row changed
4. insert exactly one `execution_request`
5. commit

If the update touches zero rows, return a user-facing error that the signal is no longer approvable.

### Reject transaction

Rejection must:

1. begin transaction
2. conditionally update the signal from `pending` to `rejected`
3. optionally add rejection reason metadata
4. commit

Reject never creates an execution request.

### Execution-request semantics

In Phase 29B, an approved signal means:

> the signal is eligible to enter the execution layer later.

It does not mean:

- order submitted
- trade executed
- paper account changed

## Service and WSL2 Operations

### Service files

Add strategy-daemon service artifacts parallel to the monitor service pattern:

- `~/.quantix/strategy/service.json`
- `~/.quantix/strategy/service.env`
- wrapper script in `~/.local/bin/quantix-strategy-run`
- unit file in `~/.config/systemd/user/quantix-strategy.service`

### `service.json`

Store:

- stable `quantix` binary absolute path
- optional `EnvironmentFile` path

Validation rules:

- binary path must be absolute
- binary path must exist
- binary path must be executable on Unix
- env-file path may be absent

### Systemd unit

The rendered unit must include:

- `ExecStart=~/.local/bin/quantix-strategy-run`
- `Environment=QUANTIX_STRATEGY_CONFIG_PATH=...`
- `Environment=QUANTIX_STRATEGY_RUNTIME_DB_PATH=...`
- `EnvironmentFile=-<path>` when configured
- `Restart=on-failure`
- `RestartSec=5`

Journald captures stdout and stderr. Phase 29B does not add an HTTP health probe.

## Relationship to Phase 29A

Phase 29A direct execution remains intact:

- `quantix strategy run --mode paper`

Phase 29B adds a separate path:

- `quantix strategy daemon run`
- `quantix strategy signal ...`
- `quantix strategy request ...`

These two paths coexist:

- Phase 29A is direct, single-shot, strategy-to-paper execution
- Phase 29B is daemonized, signal-first, approval-gated, and non-executing

## Testing

### Parser tests

Add parser coverage for:

- `strategy config init/show`
- `strategy daemon run --once`
- `strategy signal list`
- `strategy signal approve`
- `strategy signal reject`
- `strategy request list`
- `strategy service ...`
- `strategy service-config ...`

### Store and transaction tests

Cover:

- signal uniqueness by stream and bar
- approval transaction creates exactly one request
- rejection updates approval state only
- superseding a signal cancels only pending execution requests
- checkpoint bootstrap and checkpoint update behavior

### Daemon integration tests

Use temp SQLite, fake bar loader, fake config, and fake clock to verify:

- first bootstrap writes only checkpoint
- no duplicate signal when no new bar exists
- new bar writes run + signal + checkpoint
- multiple configured strategy instances on one code are independent
- config hot reload changes the active instance set

### Service tests

Cover:

- unit rendering
- wrapper rendering
- env-file inclusion
- service-config validation

Manual WSL2 smoke is useful but not required for automated completion.

## Delivery Standard

Phase 29B is complete when a user can:

1. initialize strategy-daemon config
2. configure one stock with multiple strategy instances
3. run the daemon in the foreground or through `systemd --user`
4. automatically generate a signal when a new daily bar appears
5. review signals from the CLI
6. approve or reject a signal manually
7. see approval create exactly one `execution_request`
8. do all of the above without automatic trading

## Fixed Decisions

These decisions are intentionally fixed for Phase 29B and should not be reopened during implementation unless a blocker appears:

- keep configuration in JSON for local-state consistency
- keep daemon scope to `single code + multiple strategy instances`
- keep `bootstrap_policy` modelled now but implement only `latest_only`
- keep approval manual
- create execution requests but do not consume them
- keep Phase 29A direct paper execution path unchanged
- keep strategy, approval, and execution as separate management concerns
