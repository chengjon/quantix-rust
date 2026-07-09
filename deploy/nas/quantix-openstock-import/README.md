# quantix-openstock-import (NAS deployment)

P0.15b daily minute-import batch container. Runs once per trading day via
Synology DSM scheduled task, writes minute klines + shares to ClickHouse,
records per-code outcome in PostgreSQL `quantix.import_state`.

## Preconditions (run once before first batch)

1. **PostgreSQL schema + tables** — apply the DDL on the NAS Postgres
   instance (`quantix` database AND `quantix_test` database if running
   live integration tests):

   ```bash
   cat db/schema/quantix_openstock_import_init.sql | \
     PGPASSWORD=<pass> psql -h 192.168.123.104 -p 5438 -U postgres -d quantix
   ```

   Idempotent (`IF NOT EXISTS` on every object). Creates `quantix` schema,
   `quantix.stock_info`, `quantix.import_state`, and
   `idx_import_state_status`. See file header for the operator runbook.

2. **`stock_info` populated** — the scheduler only imports codes with
   `trade_status='1'` in `quantix.stock_info`. Populate from OpenStock's
   `/data/all_stocks` endpoint or insert test codes manually.

3. **ClickHouse `minute_klines` + `minute_shares` tables** — delivered
   by P0.14 (`src/db/clickhouse/schema.rs::init_database`).

## Build

```bash
docker build -t quantix-openstock-import:nas .
```

Multi-stage musl build (rust:1.83-alpine builder + alpine:3.19 runtime).
Produces a static binary suitable for the NAS host.

## Deploy

```bash
# Upload compose + .env
sshpass -p '<nas-pass>' scp -P 223 docker-compose.yaml \
  john@192.168.123.104:/volume5/docker5/quantix-openstock-import/
sshpass -p '<nas-pass>' scp -P 223 .env \
  john@192.168.123.104:/volume5/docker5/quantix-openstock-import/

# Load image (built locally, NAS Docker Hub is blocked)
docker save quantix-openstock-import:nas | \
  sshpass -p '<nas-pass>' ssh -p 223 john@192.168.123.104 \
  "echo '<nas-pass>' | sudo -S /usr/local/bin/docker load"
```

## Run (one-shot)

```bash
# On NAS, via Synology DSM scheduled task (15:30 Asia/Shanghai daily):
cd /volume5/docker5/quantix-openstock-import && \
  echo '<nas-pass>' | sudo -S /usr/local/bin/docker compose run --rm \
  quantix-openstock-import data openstock import-minute-all \
  --date "$(date +%Y-%m-%d)" --format json
```

Stateless — no volumes, no ports. All state lives in PostgreSQL.
