# OpenStock P0.15b — Daily Minute Import Scheduler

> **Sibling:** P0.15b-pre (standalone slice — `quantix.stock_info` table + OpenStock fetch CLI). P0.15b assumes that slice has landed.
> **Builds on:** P0.15a (single-code minute import CLI — `import-minute-klines` / `import-minute-share`).
> **Scope:** A daily batch scheduler that iterates the full A-share code list, calls the P0.15a import logic per code, and tracks success/failure in PostgreSQL.

## 1. Motivation

P0.15a ships a CLI that imports one code's minute data for one date range. Running it 5000+ times a day by hand is not viable. P0.15b closes the gap: a batch entrypoint plus a deployment story that runs it automatically every trading day after market close.

## 2. Decisions (locked during brainstorming)

| # | Decision | Choice |
|---|----------|--------|
| 1 | Cadence | Trading days 15:30 Asia/Shanghai |
| 2 | Failure handling | Continue-on-error; failures recorded, batch continues |
| 3 | State tracking | Maintain state table; skip codes with latest status=success on rerun |
| 4 | Scheduler deployment | Docker container on NAS (`docker compose run --rm`) triggered by Synology DSM scheduled task |
| 5 | Acceptance scope | Batch logic + live tests + state table + status-query subcommand |
| 6 | State table location | PostgreSQL (`quantix.import_state`) |
| 7 | `stock_info` data source | `ALL_STOCKS` (primary) + `STOCK_BASIC_INFO` (LEFT JOIN for enriched fields) |
| 8 | Status query output | text + json dual format |
| 9 | Code iteration | In-process function call (option A) — reuse P0.15a handler logic via `*_inner` refactor |
| 10 | HTTP/CH clients | Constructed once per batch, reused across all codes |

## 3. Slice Boundary — P0.15b-pre vs P0.15b

### 3.1 P0.15b-pre (separate slice, ships first)

Goal: give `quantix` database the two tables P0.15b depends on.

Deliverables:

1. `quantix.stock_info` — new PostgreSQL table, populated from OpenStock.
2. `quantix.import_state` — new PostgreSQL table, starts empty.
3. New CLI: `quantix data openstock refresh-stock-info [--day YYYY-MM-DD]` — pulls from OpenStock `ALL_STOCKS` + `STOCK_BASIC_INFO` and upserts into `quantix.stock_info`.
4. **No modification** to `src/db/postgresql.rs::PostgresClient::list_stocks()` — that reads `mystocks.stock_info` for legacy code; leave it alone.

### 3.2 P0.15b (this slice)

Goal: daily batch scheduler.

**Assumes** P0.15b-pre has landed: `quantix.stock_info` exists and is populated; `quantix.import_state` exists and is empty.

**Out of scope**:
- P0.15b-pre itself
- Realtime intraday polling (OpenStock is historical API)
- Automatic retry with backoff (YAGNI; wait for real-world failure-rate data)
- Concurrency / parallel code iteration (default serial; P0.15c if needed)
- Failure alerting (DingTalk/email) — P0.15c or later
- Web UI dashboard (status query json is enough for Grafana)
- Container sidecar to OpenStock (deployment uses independent container on same NAS)
- Stateful `ReplacingMergeTree`-style dedup (PostgreSQL handles "latest wins" via PK + ORDER BY)

## 4. Architecture

### 4.1 Module Layout

New directory under `src/tasks/`:

```
src/tasks/openstock_import/
├── mod.rs          — pub re-exports: BatchScheduler, StockListFetcher, ImportStateStore, ImportEngine
├── fetcher.rs      — StockListFetcher (reads quantix.stock_info)
├── state.rs        — ImportStateStore + Status enum + ImportStateStoreTrait (for test injection)
├── engine.rs       — ImportEngine + CodeResult
└── scheduler.rs    — BatchScheduler + BatchSummary
```

Module dependency direction (matches coding standards): `cli -> tasks/openstock_import -> sources/openstock + db + core`. No cycles.

### 4.2 Components

#### StockListFetcher

```rust
#[async_trait]
pub trait StockListFetchTrait: Send + Sync {
    /// Returns active codes (trade_status='1') for the given date.
    async fn list_active_codes(&self, date: NaiveDate) -> Result<Vec<String>>;
}

pub struct StockListFetcher { pg: PostgresClient }

#[async_trait]
impl StockListFetchTrait for StockListFetcher {
    async fn list_active_codes(&self, date: NaiveDate) -> Result<Vec<String>> {
        // SELECT code FROM quantix.stock_info WHERE trade_status='1'
        // ORDER BY code
    }
}
```

- **Input**: date (informational today; reserved for future filtering)
- **Output**: e.g. `["sh600000", "sz000001", ...]` (~5000 entries)
- **Depends on**: `PostgresClient`
- **Why trait**: lets `BatchScheduler` unit tests inject an in-memory fetcher without touching PG
- **Test**: integration — real PG (quantix_test database), no mock

#### ImportStateStore

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Success,
    Failed { reason: String },
}

#[async_trait]
pub trait ImportStateStoreTrait: Send + Sync {
    /// Returns latest status for (code, date, kind). None = no record (should run).
    async fn get_status(&self, code: &str, date: NaiveDate, kind: &str) -> Result<Option<Status>>;
    /// Writes a status row. Multiple writes for same (code, date, kind) kept as history.
    async fn record(&self, code: &str, date: NaiveDate, kind: &str,
                    status: &Status, batch_id: &str) -> Result<()>;
}

pub struct ImportStateStore { pg: PostgresClient }

#[async_trait]
impl ImportStateStoreTrait for ImportStateStore { /* real PG queries */ }
```

- **Test**: integration — real PG (quantix_test); tests for None/Success/Failed round-trip, latest-wins ordering
- **Why trait**: lets `ImportEngine` unit tests use an in-memory store without touching PG

#### ImportEngine

```rust
pub struct CodeResult {
    pub code: String,
    pub klines: Status,
    pub share: Status,
}

pub struct ImportEngine<'a, S: ImportStateStoreTrait> {
    openstock: &'a OpenStockClient,
    clickhouse: &'a ClickHouseClient,
    state: &'a S,
    batch_id: String,
}

impl<'a, S: ImportStateStoreTrait> ImportEngine<'a, S> {
    /// Processes one code: query klines status, run or skip, record result;
    /// same for share. klines and share are independent — one's failure
    /// does NOT block the other.
    pub async fn process_code(&self, code: &str, date: NaiveDate) -> Result<CodeResult>;
}
```

- **Depends on**: OpenStockClient, ClickHouseClient, ImportStateStoreTrait
- **Test**: unit — inject in-memory `MockStateStore`, real OpenStock + real ClickHouse (since P0.15a already validated them); tests "skip on success", "klines failed but share continues", "successful record on success"

#### BatchScheduler

```rust
pub struct BatchSummary {
    pub batch_id: String,
    pub date: NaiveDate,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub total_codes: usize,
    pub success_count: KlineShareCount,  // {klines: N, share: M}
    pub failed_count: KlineShareCount,
    pub failures: Vec<FailureEntry>,  // {code, kind, reason}
}

pub struct BatchScheduler<F: StockListFetchTrait, S: ImportStateStoreTrait> {
    fetcher: F,
    state_store: S,
    settings: OpenStockSettings,  // for OpenStockClient construction
}

impl<F: StockListFetchTrait, S: ImportStateStoreTrait> BatchScheduler<F, S> {
    /// Main entrypoint. Steps:
    /// 1. fetch codes
    /// 2. generate batch_id (uuid v4)
    /// 3. construct OpenStockClient + ClickHouseClient ONCE
    /// 4. construct ImportEngine ONCE (holds client refs)
    /// 5. for each code: engine.process_code → push to summary
    ///    continue-on-error: process_code returns Ok(CodeResult) even when kinds
    ///    internally failed. Err is reserved for infra-fatal (PG down) that
    ///    breaks the whole batch.
    pub async fn run(&self, date: NaiveDate, dry_run: bool) -> Result<BatchSummary>;
}
```

- **Test**: unit — mock fetcher + mock state_store via traits; tests "batch_id is fresh uuid", "engine constructed once", "BatchSummary aggregates correctly". Engine itself is a concrete struct (not a trait) — testing its state machine is done in `engine.rs` unit tests with in-memory state.

### 4.3 Resource Lifecycle (option A: in-process function call)

```rust
async fn run(&self, date, dry_run) -> Result<BatchSummary> {
    let codes = self.fetcher.list_active_codes(date).await?;
    let batch_id = uuid::Uuid::new_v4().to_string();

    // Constructed ONCE per batch, reused across all codes.
    let openstock = OpenStockClient::from_settings(&self.settings)?;
    let clickhouse = ClickHouseClient::with_default_config().await?;
    let engine = ImportEngine {
        openstock: &openstock,
        clickhouse: &clickhouse,
        state: &self.state_store,
        batch_id: batch_id.clone(),
    };

    let mut summary = BatchSummary::new(batch_id, date);
    for code in codes {
        match engine.process_code(&code, date).await {
            Ok(result) => summary.push(result),
            Err(e) => {
                // infra-fatal — abort batch
                tracing::error!("infra-fatal on {}: {}", code, e);
                return Err(e);
            }
        }
    }
    Ok(summary)
}
```

Cost: 5000 codes × 2 kinds = 10000 in-process function calls. No client reconstruction per code (saves ~50s vs subprocess).

### 4.4 P0.15a Refactor — `*_inner` Extraction

`src/cli/handlers/openstock_handler.rs::import_openstock_minute_klines` is split:

```rust
// CLI entrypoint — unchanged signature, unchanged behavior
pub(crate) async fn import_openstock_minute_klines(
    settings: &OpenStockSettings,
    code: String, period: String, adjust: String,
    start: Option<String>, end: Option<String>, apply: bool,
) -> Result<()> {
    // ... arg parsing ...
    let client = OpenStockClient::from_settings(settings)?;
    let ch = ClickHouseClient::with_default_config().await?;
    let sink = ClickHouseMinuteKlineSink { client: ch.client() };
    import_minute_klines_inner(&client, &sink, code, period_enum,
                               start_date, end_date, adjust_enum, will_apply).await
}

// New — reused by both CLI and BatchScheduler
pub(crate) async fn import_minute_klines_inner(
    client: &OpenStockClient,
    sink: &ClickHouseMinuteKlineSink<'_>,
    code: &str,
    period: MinutePeriod,
    start: NaiveDate, end: NaiveDate,
    adjust: AdjustType,
    will_apply: bool,
) -> Result<ImportStats>;
```

Same for `import_minute_share` → `import_minute_share_inner`.

CLI interface is 100% backward-compatible.

## 5. Data Flow & State Machine

### 5.1 End-to-end

```
[15:30 Asia/Shanghai] Synology DSM scheduled task fires
    ↓
docker compose run --rm quantix-openstock-import \
    quantix data openstock import-minute-all --date today
    ↓
CLI parses args → CliRuntime::load() reads env
    ↓
BatchScheduler::run(date=today, dry_run=false)
    ├─ StockListFetcher::list_active_codes(today)
    │     → SELECT code FROM quantix.stock_info WHERE trade_status='1'
    │     → ["sh600000", "sz000001", ...] (~5000 codes)
    ├─ batch_id = uuid::Uuid::new_v4()
    ├─ construct OpenStockClient + ClickHouseClient (ONCE)
    └─ for code in codes:
         engine.process_code(code, today) → CodeResult
            └─ klines: query state → skip if success, else run + record
            └─ share: query state → skip if success, else run + record
    ↓
print BatchSummary (text or json) to stdout
exit 0
```

### 5.2 Per-code state machine (process_code)

```
For each kind in [klines, share]:
  ┌─ state.get_status(code, date, kind)
  └─ latest == Success? ────→ skip (no CH call, no state write)
       │ (None or Failed)
       ↓
  ┌─ call inner function (stream_minute_*_to_clickhouse)
  │   ↑ internally calls ch_delete(code, date, table) before write
  │     — idempotent over reruns
  └─ result?
       Ok(stats) → state.record(code, date, kind, Success, batch_id)
       Err(e)    → state.record(code, date, kind, Failed{reason}, batch_id)
                  ↑ continue to next kind; do NOT abort batch
```

### 5.3 Decisions encoded

- **klines and share are independent**: one's failure does not block the other
- **"Latest wins" semantics**: query `ORDER BY imported_at DESC LIMIT 1` — an early success followed by a failure means "rerun"
- **Implicit retry**: rerun same date → skips Success, retries Failed/None. No separate `retry-failed` subcommand (YAGNI)
- **Continue-on-error boundary**:
  - **Continue**: per-code HTTP error, parse error, CH write error, empty data, schema mismatch
  - **Abort batch** (`return Err`): PG unreachable (cannot read/write state), OpenStock global unreachable (healthcheck fails), env/config error

### 5.4 Idempotency proof

```
Run 1 (date=2026-07-08):
  sh600000 klines → CH gets 48 rows + state(success)
  sh600000 share  → CH gets 240 rows + state(success)

Run 2 (date=2026-07-08, manual rerun):
  sh600000 klines → query state → latest success → SKIP
  sh600000 share  → query state → latest success → SKIP
  CH untouched. State untouched. Idempotent.
```

Half-failed run:
```
Run 1 partial:
  sh600000 klines → CH gets 24 rows, write fails mid-stream → state(failed)
Run 2:
  sh600000 klines → query state → failed → run inner
    → inner's ch_delete clears the 24 partial rows
    → fresh write of 48 rows
    → state(success)
  Old failed record retained as audit history.
```

## 6. CLI Surface

### 6.1 New subcommand — `import-minute-all`

```
quantix data openstock import-minute-all \
    [--date YYYY-MM-DD]   # default: today
    [--format text|json]  # default: text
    [--dry-run]           # default: false (off)
```

- `--date today` (or omitted) — schedule-driven daily run
- `--date 2026-07-08` — manual backfill
- `--dry-run` — print plan (codes list, batch_id) without calling OpenStock/CH or writing state
- Output: BatchSummary to stdout, progress logs to stderr (journald-friendly)

### 6.2 New subcommand — `import-status`

```
quantix data openstock import-status \
    --date YYYY-MM-DD \
    [--format text|json]   # default: text
```

Output includes latest batch summary for the date, pending codes (in `stock_info` but no success record), and failed detail.

## 7. Schema

### 7.1 `quantix.stock_info` (P0.15b-pre delivers)

```sql
CREATE TABLE quantix.stock_info (
    code           VARCHAR(16) PRIMARY KEY,
    name           VARCHAR(64) NOT NULL,
    market         VARCHAR(16),
    exchange       VARCHAR(16),
    listing_board  VARCHAR(16),
    total_shares   BIGINT,
    listing_date   DATE,
    trade_status   VARCHAR(8),     -- '1'=active, '0'=suspended/delisted
    fetched_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 7.2 `quantix.import_state` (P0.15b-pre delivers, P0.15b consumes)

```sql
CREATE TABLE quantix.import_state (
    code         VARCHAR(16) NOT NULL,
    trade_date   DATE NOT NULL,
    kind         VARCHAR(8) NOT NULL CHECK (kind IN ('klines', 'share')),
    status       VARCHAR(8) NOT NULL CHECK (status IN ('success', 'failed')),
    reason       TEXT,
    batch_id     VARCHAR(40) NOT NULL,
    imported_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (code, trade_date, kind, imported_at)
);
CREATE INDEX idx_import_state_status ON import_state(trade_date, status);
```

Latest-wins query pattern:
```sql
SELECT status, reason FROM quantix.import_state
WHERE code=$1 AND trade_date=$2 AND kind=$3
ORDER BY imported_at DESC LIMIT 1;
```

## 8. Deployment

### 8.1 Target Machine — NAS 192.168.123.104 (Docker-first)

Per `/opt/claude/2hermes/shared/nas-deployment-guide.md`:

- OpenStock, ClickHouse, PostgreSQL all run on NAS as Docker containers
- NAS is the only 7×24 machine in this environment
- WSL2 (dev) is not viable for production (sleep/hibernate unreliability)
- iStoreOS 192.168.123.81 has only 1.8GB free disk — too tight
- NAS deployment follows the standardized Docker workflow (compose + .env + ACL fix + image acceleration)

### 8.2 Container Image

Multi-stage Dockerfile in repo root:

```dockerfile
# Stage 1: musl builder (static binary)
FROM rust:1.83-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl \
    --bin quantix

# Stage 2: minimal runtime
FROM alpine:3.19
RUN apk add --no-cache ca-certificates tzdata
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/quantix \
                    /usr/local/bin/quantix
RUN chmod +x /usr/local/bin/quantix
ENV TZ=Asia/Shanghai
ENTRYPOINT ["/usr/local/bin/quantix"]
```

**Estimated image size**: ~60-70 MB (alpine ~7 MB + binary ~50 MB + certs/tzdata ~5 MB).

### 8.3 Compose File (NAS)

`deploy/nas/quantix-openstock-import/docker-compose.yaml`:

```yaml
services:
  quantix-openstock-import:
    image: quantix-openstock-import:nas
    container_name: quantix-openstock-import
    network_mode: host   # reach OpenStock/CH/PG on 192.168.123.104 directly
    env_file:
      - .env
    # No volumes — stateless. All state lives in PG.
    # No ports — not a server.
```

`deploy/nas/quantix-openstock-import/.env.example` (committed; real `.env` deployed separately):

```bash
OPENSTOCK_BASE_URL=http://192.168.123.104:8040
OPENSTOCK_API_KEY=changeme
CLICKHOUSE_URL=http://192.168.123.104:8123
CLICKHOUSE_USER=default
CLICKHOUSE_PASSWORD=changeme
POSTGRESQL_HOST=192.168.123.104
POSTGRESQL_PORT=5438
POSTGRESQL_USER=postgres
POSTGRESQL_PASSWORD=changeme
POSTGRESQL_DATABASE=quantix
QUANTIX_OPENSTOCK_MINUTE_APPLY=yes
TZ=Asia/Shanghai
```

Per NAS guide §2.4 — no hardcoded secrets in docs or compose; real values in `.env` (gitignored, deployed separately).

### 8.4 Synology DSM Scheduled Task

Triggered via DSM Control Panel → Task Scheduler → Create → Scheduled Task → User-defined script:

- **User**: `root` (needed for docker)
- **Schedule**: Run on Monday-Friday at 15:30 Asia/Shanghai
- **Script**:
  ```bash
  cd /volume5/docker5/quantix-openstock-import && \
    /usr/local/bin/docker compose run --rm --no-deps \
    quantix-openstock-import \
    data openstock import-minute-all --date today \
    >> /var/log/quantix-import.log 2>&1
  ```

Logs: DSM captures task output in `/var/log/quantix-import.log` and DSM Task Scheduler's own log viewer.

### 8.5 Deploy Workflow

1. **WSL2 build**: `cargo build --release --target x86_64-unknown-linux-musl`
2. **WSL2 image**: `docker build -t quantix-openstock-import:nas -f Dockerfile .`
3. **Transfer to NAS**:
   ```bash
   docker save quantix-openstock-import:nas | gzip > /tmp/q.tar.gz
   sshpass -p 'c790414J' scp -P 223 /tmp/q.tar.gz \
     john@192.168.123.104:/volume5/docker5/quantix-openstock-import/
   NAS="sshpass -p 'c790414J' ssh -p 223 john@192.168.123.104"
   $NAS "cd /volume5/docker5/quantix-openstock-import && \
     echo c790414J | sudo -S /usr/local/bin/docker load -i q.tar.gz"
   ```
4. **Upload compose + .env**:
   ```bash
   sshpass -p 'c790414J' scp -P 223 docker-compose.yaml \
     john@192.168.123.104:/volume5/docker5/quantix-openstock-import/
   sshpass -p 'c790414J' scp -P 223 .env \
     john@192.168.123.104:/volume5/docker5/quantix-openstock-import/
   ```
5. **Smoke test** (manual):
   ```bash
   $NAS "cd /volume5/docker5/quantix-openstock-import && \
     echo c790414J | sudo -S /usr/local/bin/docker compose run --rm \
     quantix-openstock-import data openstock import-minute-all --date 2026-07-08"
   ```
6. **Configure DSM scheduled task** (manual, one-time, via web UI)
7. **Verify** by manually triggering the DSM task and checking logs

## 9. Observability

### 9.1 Stdout / stderr

- **stdout**: BatchSummary (text or json per `--format`)
- **stderr**: progress logs every 100 codes (`[progress] 100/5165 codes processed (success=98, failed=2, elapsed=42s)`)
- **stderr**: per-failure notification (`[failed] sh600001 klines: OpenStock 404: symbol not found`)

### 9.2 Example outputs

Text summary:
```
BatchSummary
  batch_id: 550e8400-e29b-41d4-a716-446655440000
  date: 2026-07-08
  started_at: 2026-07-08T15:30:05+08:00
  finished_at: 2026-07-08T15:42:18+08:00
  elapsed: 12m 13s
  total_codes: 5165
  success: klines=5120 share=5118
  failed:   klines=45   share=47
  ── failed detail ──
  sh600001 (klines): OpenStock 404
  sz000002 (share): ClickHouse write timeout
  ...
```

JSON summary:
```json
{
  "batch_id": "550e8400-e29b-41d4-a716-446655440000",
  "date": "2026-07-08",
  "started_at": "2026-07-08T15:30:05+08:00",
  "finished_at": "2026-07-08T15:42:18+08:00",
  "elapsed_seconds": 733,
  "total_codes": 5165,
  "success_count": {"klines": 5120, "share": 5118},
  "failed_count": {"klines": 45, "share": 47},
  "failures": [
    {"code": "sh600001", "kind": "klines", "reason": "OpenStock 404"},
    {"code": "sz000002", "kind": "share", "reason": "ClickHouse write timeout"}
  ]
}
```

## 10. Testing Strategy

### 10.1 What NOT to mock (already validated)

OpenStock and ClickHouse are validated by P0.15a live tests. Do not mock them in P0.15b. Mocking them would repeat P0.15a work and miss real API drift (the `toDateString` → `toDate` fix came from live testing, not mocks).

### 10.2 Unit tests (in-module `#[cfg(test)]`)

| Module | Test | Mocks |
|--------|------|-------|
| `scheduler.rs` | batch_id is fresh uuid per run; BatchSummary aggregates CodeResults correctly; engine factory reuses clients | mock fetcher + mock engine via traits |
| `engine.rs` | skip when state=Success; klines failed but share continues; both succeed → both recorded | mock StateStoreTrait (in-memory); real OpenStock + real CH (PG triple-gated) |

### 10.3 Integration tests (`tests/openstock_live_import_all.rs`)

Triple-gated: `QUANTIX_OPENSTOCK_LIVE=1` + `QUANTIX_CLICKHOUSE_LIVE=1` + `QUANTIX_POSTGRES_LIVE=1`.

| Test | Verifies |
|------|---------|
| T1 `import_minute_all_live_smoke` | 3-code batch → CH has new data, PG has 6 success records |
| T2 `import_minute_all_continue_on_error` | 1 fake code (sh999999) mixed with 2 real → batch completes, 1 failed recorded |
| T3 `import_minute_all_skips_already_success` | rerun same date → no CH writes, no new state records |
| T4 `import_status_query` | `import-status --format json` reflects correct counts |

### 10.4 PG test isolation

PG tests run against a separate `quantix_test` database (same TimescaleDB instance, different DB). Tests use `sqlx::test` macro which auto-creates a temporary schema per test and cleans up on drop. No pollution of production `quantix` database.

### 10.5 Static quality gates

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test                          # unit + vacuous live (gates off)
cargo test --test openstock_live_import_all -- --ignored  # live verification only
cargo build --release --target x86_64-unknown-linux-musl  # static binary build check
```

## 11. Acceptance Criteria

Code:
- [ ] `src/tasks/openstock_import/` directory with 5 files (mod/fetcher/state/engine/scheduler), each <500 lines
- [ ] P0.15a handler refactored: `import_minute_klines_inner` + `import_minute_share_inner` exist; CLI unchanged
- [ ] `import-minute-all` subcommand registered
- [ ] `import-status` subcommand registered
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -- -D warnings` clean
- [ ] P0.15a existing tests pass (no regression from refactor)

Tests:
- [ ] Unit tests cover BatchScheduler logic and ImportEngine state machine
- [ ] Live tests T1-T4 pass under triple-gating
- [ ] `quantix_test` PG database exists; tests do not touch production `quantix`

Deployment:
- [ ] `Dockerfile` at repo root (multi-stage, musl + alpine)
- [ ] `deploy/nas/quantix-openstock-import/docker-compose.yaml` + `.env.example`
- [ ] Deploy appendix documents: WSL2 build steps, image transfer (`docker save | ssh | docker load`), DSM scheduled task setup
- [ ] Image size <100 MB verified

Design constraints:
- [ ] OpenStockClient + ClickHouseClient constructed once per batch (verified via logging)
- [ ] continue-on-error verified (T2)
- [ ] Idempotency verified (T3)
- [ ] Status query works (T4)

## 12. Known Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| 10000 OpenStock HTTP requests per batch may trigger rate limit | Default serial execution (no delay); if real-world rate limits surface, add `--delay-ms` flag in a follow-up slice before enabling concurrency |
| OpenStock single point of failure (NAS container down) | Engine healthchecks OpenStock at batch start; aborts if unreachable |
| PG connection pool exhaustion | ImportStateStore uses sqlx PoolOptions with bounded pool; one pool per batch |
| Timezone: A-share on Shanghai, cron on NAS local | NAS configured to Asia/Shanghai (deploy step verifies); scheduler uses `chrono::Local` for date boundaries |
| Docker image storage on NAS | Image ~60-70 MB; negligible vs NAS multi-TB storage; no GC needed |
| WSL2 build environment flakiness | Documented in deploy appendix; build is repeatable via `cargo build --release --target x86_64-unknown-linux-musl` |

## 13. Out of Scope (YAGNI)

- Automatic retry with exponential backoff
- Concurrent code iteration
- Failure alerts (DingTalk / email)
- Multi-day backfill subcommand (run `--date` multiple times manually)
- Web UI dashboard (status json is Grafana-consumable)
- `assert_cmd` subprocess CLI tests (unit + live tests suffice)
- Sidecar to OpenStock container (independent compose project on same NAS)

## 14. References

- `docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md` — P0.15a sibling
- `docs/superpowers/plans/2026-07-07-openstock-p0-15a-live-import-test.md` — P0.15a live tests
- `/opt/claude/2hermes/shared/nas-deployment-guide.md` — NAS Docker SOP
- `/opt/claude/openstock/docs/DATA_CAPABILITY_SCOPE.md` — OpenStock data sources
- `docs/RUST_CODING_STANDARDS.md` — file size limits, error handling, logging rules
