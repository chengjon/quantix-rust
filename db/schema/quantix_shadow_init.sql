-- OpenStock shadow persistence schema (P0.8g-impl)
--
-- Operator manual runbook: execute this once per fresh ClickHouse
-- instance before invoking `quantix data openstock persist-live --apply`.
-- See README.md > "OpenStock shadow persistence" for the full
-- dry-run/apply/rollback flow.
--
-- This file is intentionally NOT applied by any automated migration
-- in CI — the shadow namespace is opt-in and must be created by the
-- operator who has confirmed they want the write path enabled.

CREATE DATABASE IF NOT EXISTS quantix_shadow;

CREATE TABLE IF NOT EXISTS quantix_shadow.openstock_daily_kline_shadow
(
    source         LowCardinality(String),
    period         LowCardinality(String),
    code           LowCardinality(String),
    date           Date,
    open           Float64,
    high           Float64,
    low            Float64,
    close          Float64,
    volume         Float64,
    amount         Float64,
    adjust_type    LowCardinality(String),
    batch_id       String,
    artifact_hash  String,
    ingested_by    LowCardinality(String),
    ingested_at    DateTime64(3, 'UTC')
)
ENGINE = ReplacingMergeTree(ingested_at)
PARTITION BY toYYYYMM(date)
ORDER BY (source, period, code, date, adjust_type, batch_id);
