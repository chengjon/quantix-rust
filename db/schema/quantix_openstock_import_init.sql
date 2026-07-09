-- OpenStock daily minute-import state schema (P0.15b-pre + P0.15b)
--
-- Operator manual runbook: execute this once per fresh PostgreSQL
-- instance before invoking `quantix data openstock import-minute-all`
-- or `quantix data openstock import-status`. Both commands read/write
-- state under the `quantix` schema.
--
-- Run on the production `quantix` database AND on the `quantix_test`
-- database (the latter for live integration tests in
-- tests/openstock_live_import_all.rs).
--
-- This file is intentionally NOT applied by any automated migration
-- in CI — the schema is opt-in and must be created by the operator
-- who has confirmed they want the batch-import write path enabled.
-- Same convention as quantix_shadow_init.sql.

CREATE SCHEMA IF NOT EXISTS quantix;

-- =============================================================================
-- quantix.stock_info  (P0.15b-pre delivers; P0.15b consumes)
--
-- Catalog of stock codes that the batch scheduler should import each
-- trading day. Populated from OpenStock's /data/all_stocks endpoint
-- (or manually for testing). `trade_status='1'` means active; the
-- StockListFetcher only reads active codes.
-- =============================================================================

CREATE TABLE IF NOT EXISTS quantix.stock_info (
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

-- =============================================================================
-- quantix.import_state  (P0.15b-pre delivers; P0.15b consumes)
--
-- Per (code, trade_date, kind) import outcome. Latest-wins semantics:
-- a rerun inserts a new row, and the scheduler reads the most recent
-- status to decide skip-or-retry. PRIMARY KEY includes imported_at so
-- multiple attempts on the same day are preserved for audit.
-- =============================================================================

CREATE TABLE IF NOT EXISTS quantix.import_state (
    code         VARCHAR(16) NOT NULL,
    trade_date   DATE NOT NULL,
    kind         VARCHAR(8) NOT NULL CHECK (kind IN ('klines', 'share')),
    status       VARCHAR(8) NOT NULL CHECK (status IN ('success', 'failed')),
    reason       TEXT,
    batch_id     VARCHAR(40) NOT NULL,
    imported_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (code, trade_date, kind, imported_at)
);

CREATE INDEX IF NOT EXISTS idx_import_state_status
    ON quantix.import_state(trade_date, status);

-- =============================================================================
-- Latest-wins query pattern (reference; not a materialized view)
--
-- SELECT status, reason
-- FROM quantix.import_state
-- WHERE code = $1 AND trade_date = $2 AND kind = $3
-- ORDER BY imported_at DESC
-- LIMIT 1;
-- =============================================================================
