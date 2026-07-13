//! live_import_* 表的 DDL 常量与 `ensure_schema` 所需的常量列表。

pub const CREATE_LIVE_IMPORT_BATCHES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_batches (
    batch_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    input_name TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    total_rows INTEGER NOT NULL,
    inserted_rows INTEGER NOT NULL,
    skipped_duplicates INTEGER NOT NULL,
    conflict_rows INTEGER NOT NULL
);
"#;

pub const CREATE_LIVE_IMPORT_RECORDS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_records (
    account_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    batch_id TEXT NOT NULL,
    PRIMARY KEY (account_id, external_id)
);
"#;

pub const CREATE_LIVE_IMPORT_CONFLICTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_conflicts (
    id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    existing_record_json TEXT NOT NULL,
    incoming_record_json TEXT NOT NULL,
    detail TEXT NOT NULL,
    created_at TEXT NOT NULL
);
"#;

pub const CREATE_LIVE_IMPORT_MIRROR_ACCOUNTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_mirror_accounts (
    account_id TEXT PRIMARY KEY,
    trading_date TEXT NOT NULL,
    as_of TEXT NOT NULL,
    starting_total_assets TEXT NOT NULL,
    current_total_assets TEXT NOT NULL,
    cash_balance TEXT NOT NULL,
    realized_pnl TEXT NOT NULL,
    total_fees TEXT NOT NULL,
    last_rebuild_at TEXT NOT NULL
);
"#;

pub const CREATE_LIVE_IMPORT_MIRROR_POSITIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_mirror_positions (
    account_id TEXT NOT NULL,
    code TEXT NOT NULL,
    volume INTEGER NOT NULL,
    avg_cost TEXT NOT NULL,
    last_trade_at TEXT NOT NULL,
    PRIMARY KEY (account_id, code)
);
"#;

pub const CREATE_LIVE_IMPORT_REBUILDS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS live_import_rebuilds (
    rebuild_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    status TEXT NOT NULL,
    detail TEXT,
    rebuilt_at TEXT NOT NULL
);
"#;

/// 按依赖顺序排列的全部 live_import_* 表 DDL；`ensure_schema` 依此执行确保表结构。
pub const SCHEMA_STATEMENTS: &[&str] = &[
    CREATE_LIVE_IMPORT_BATCHES_TABLE_SQL,
    CREATE_LIVE_IMPORT_RECORDS_TABLE_SQL,
    CREATE_LIVE_IMPORT_CONFLICTS_TABLE_SQL,
    CREATE_LIVE_IMPORT_MIRROR_ACCOUNTS_TABLE_SQL,
    CREATE_LIVE_IMPORT_MIRROR_POSITIONS_TABLE_SQL,
    CREATE_LIVE_IMPORT_REBUILDS_TABLE_SQL,
];
