//! Field-level sync tests for `docs/contracts/external-systems.md` Appendix C.
//!
//! Spec source: `docs/superpowers/specs/2026-07-11-external-systems-contract-design.md`
//! Layer 2 spot-check contract. Hard-fail on any field-set mismatch.
//!
//! How it works:
//! 1. Parse Appendix C marker blocks `<!-- L2:TAG name=X -->...<!-- /L2 -->`
//!    from the contract doc to get the documented field set.
//! 2. Parse the corresponding source (CREATE TABLE DDL or `pub struct` body)
//!    to get the actual field set.
//! 3. Assert the two sets are equal (unordered).
//!
//! When this test fails, the failure message names exactly which fields
//! are missing on each side, plus the source location to update.

use std::collections::HashSet;

const DOC: &str = include_str!("../docs/contracts/external-systems.md");

const CLICKHOUSE_SCHEMA: &str = include_str!("../src/db/clickhouse/schema.rs");
const BRIDGE_MODELS: &str = include_str!("../src/bridge/models.rs");
const OPENSTOCK_ENVELOPE: &str = include_str!("../src/sources/openstock_envelope.rs");
const DATA_MODELS: &str = include_str!("../src/data/models.rs");
const IMPORT_STATE_STORE: &str = include_str!("../src/tasks/openstock_import/state.rs");

// ---------------------------------------------------------------------------
// Doc-side parser: extract the field list inside a marker block.
// ---------------------------------------------------------------------------

fn parse_doc_block(tag: &str, name: &str) -> HashSet<String> {
    let open = format!("<!-- L2:{tag} name={name} -->");
    let close = "<!-- /L2 -->";

    let start_idx = DOC
        .find(&open)
        .unwrap_or_else(|| panic!("doc marker {open} not found"));
    let rest = &DOC[start_idx + open.len()..];
    let end_idx = rest
        .find(close)
        .unwrap_or_else(|| panic!("doc closing marker not found for {open}"));
    let body = &rest[..end_idx];

    // Strip fenced code block fencing (``` on its own line), take first non-empty line.
    body.trim()
        .lines()
        .find(|line| !line.trim().starts_with("```") && !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or_else(|| panic!("doc block {open} has no field line"))
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

// ---------------------------------------------------------------------------
// Source-side parsers.
// ---------------------------------------------------------------------------

/// Parse column names from a `CREATE TABLE IF NOT EXISTS <table> (...)` block
/// inside `src/db/clickhouse/schema.rs`.
///
/// Returns the set of column names (excluding `MATERIALIZED` computed columns
/// is optional — we include them, since the doc lists them too).
fn parse_clickhouse_columns(src: &str, table: &str) -> HashSet<String> {
    let needle = format!("CREATE TABLE IF NOT EXISTS {table} ");
    let start = src
        .find(&needle)
        .unwrap_or_else(|| panic!("table {table} not found in schema.rs"));
    let after_create = &src[start..];
    let open_paren = after_create
        .find('(')
        .unwrap_or_else(|| panic!("no '(' after CREATE TABLE {table}"));
    // Find matching close paren at depth 0.
    let mut depth: i32 = 0;
    let mut close_idx = None;
    for (i, ch) in after_create[open_paren..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(open_paren + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = close_idx.unwrap_or_else(|| panic!("unbalanced parens for {table}"));
    let body = &after_create[open_paren + 1..close_idx];

    body.split(',')
        .map(str::trim)
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // Skip nested ENGINE/PARTITION/ORDER/SETTINGS — they appear in the
            // body but are not column defs. Heuristic: column defs start with
            // an identifier followed by a type. PARTITION/ORDER/SETTINGS/ENGINE
            // are uppercase keywords.
            if line.starts_with("ENGINE")
                || line.starts_with("PARTITION")
                || line.starts_with("ORDER")
                || line.starts_with("SETTINGS")
            {
                return None;
            }
            // First token is column name.
            let name = line.split_whitespace().next()?;
            // Filter out non-identifier lines just in case.
            if !name
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_')
                .unwrap_or(false)
            {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

/// Parse field names from a `pub struct <Name> { ... }` or `pub struct <Name><T> { ... }` block.
/// Captures `pub <field>: <type>` lines.
fn parse_struct_fields(src: &str, struct_name: &str) -> HashSet<String> {
    // Match either `pub struct Name {` or `pub struct Name<T> {`. We anchor on
    // the start of the declaration, then find the first `{`.
    let needle_prefix = format!("pub struct {struct_name}");
    let start = src
        .find(&needle_prefix)
        .unwrap_or_else(|| panic!("struct {struct_name} not found"));
    let after_struct = &src[start..];
    // Sanity: the next char after the prefix is `<`, `{`, or whitespace.
    let after_prefix = &after_struct[needle_prefix.len()..];
    let next_ch = after_prefix
        .chars()
        .find(|c| !c.is_whitespace())
        .unwrap_or('{');
    assert!(
        next_ch == '{' || next_ch == '<',
        "struct {struct_name} declaration not followed by '{{' or '<'"
    );
    let open_brace = after_struct
        .find('{')
        .unwrap_or_else(|| panic!("no '{{' after struct {struct_name}"));
    let mut depth: i32 = 0;
    let mut close_idx = None;
    for (i, ch) in after_struct[open_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(open_brace + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = close_idx.unwrap_or_else(|| panic!("unbalanced braces for {struct_name}"));
    let body = &after_struct[open_brace + 1..close_idx];

    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            // Match `pub <field>: <type>` OR `pub <field>: Option<...>`
            if !line.starts_with("pub ") {
                return None;
            }
            // Skip attribute lines and the struct decl itself.
            if line.starts_with("pub struct") || line.starts_with("#[") {
                return None;
            }
            // Strip trailing comma for safety.
            let line = line.trim_end_matches(',');
            // Take the token after `pub ` and before `:`.
            let after_pub = &line[4..];
            let colon = after_pub.find(':')?;
            let name = after_pub[..colon].trim();
            if name.is_empty() {
                return None;
            }
            // Field name must be a valid Rust ident.
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

/// Specialized parser for `import_state` — the DDL lives in a doc-comment
/// inside `state.rs`. We pull column names from the `CREATE TABLE` in that
/// doc-comment.
///
/// Edge case handled: the `CHECK (kind IN ('klines', 'share'))` clause has
/// commas *inside* parens. Our paren-depth-tracking splitter ignores those
/// inner commas so column boundaries are correct.
fn parse_import_state_columns_from_doc_comment() -> HashSet<String> {
    let needle = "CREATE TABLE quantix.import_state (";
    let start = IMPORT_STATE_STORE
        .find(needle)
        .unwrap_or_else(|| panic!("import_state DDL not found in state.rs"));
    let after = &IMPORT_STATE_STORE[start..];
    let open_paren = after.find('(').unwrap();

    // Track paren depth to split columns correctly even when a column has
    // an inner parenthesised clause with commas (CHECK (...) etc.).
    let mut depth: i32 = 0;
    let mut close_idx = None;
    for (i, ch) in after[open_paren..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(open_paren + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = close_idx.unwrap();
    let body = &after[open_paren + 1..close_idx];

    // Now split on commas only at depth 0 (relative to body start).
    let mut columns: Vec<String> = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    for ch in body.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                columns.push(std::mem::take(&mut current));
            }
            other => current.push(other),
        }
    }
    if !current.trim().is_empty() {
        columns.push(current);
    }

    columns
        .into_iter()
        .filter_map(|raw| {
            // Each raw column line is like `///     code         VARCHAR(16) NOT NULL`.
            // Strip leading `///` markers and whitespace.
            let mut line = raw.trim().to_string();
            while line.starts_with("///") {
                line = line[3..].trim_start().to_string();
            }
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // Skip PRIMARY KEY / CHECK / CONSTRAINT lines.
            let upper = line.to_uppercase();
            if upper.starts_with("PRIMARY")
                || upper.starts_with("CHECK")
                || upper.starts_with("CONSTRAINT")
            {
                return None;
            }
            let name = line.split_whitespace().next()?;
            // Reject if name is not a valid Postgres identifier.
            if !name
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_')
                .unwrap_or(false)
            {
                return None;
            }
            Some(name.to_string())
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Test helpers.
// ---------------------------------------------------------------------------

fn assert_field_sets_equal(
    doc_set: HashSet<String>,
    src_set: HashSet<String>,
    context: &str,
    hint: &str,
) {
    if doc_set == src_set {
        return;
    }
    let mut missing_in_doc: Vec<_> = src_set.difference(&doc_set).cloned().collect();
    missing_in_doc.sort();
    let mut missing_in_src: Vec<_> = doc_set.difference(&src_set).cloned().collect();
    missing_in_src.sort();
    panic!(
        "\n[{context}] field-set mismatch\n\
         Missing in doc:    {missing_in_doc:?}\n\
         Missing in source: {missing_in_src:?}\n\
         Hint: {hint}\n"
    );
}

// ---------------------------------------------------------------------------
// ClickHouse table column tests.
// ---------------------------------------------------------------------------

#[test]
fn l2_kline_data_columns_match() {
    let doc = parse_doc_block("CLICKHOUSE_TABLE", "kline_data");
    let src = parse_clickhouse_columns(CLICKHOUSE_SCHEMA, "kline_data");
    assert_field_sets_equal(
        doc,
        src,
        "kline_data",
        "update docs/contracts/external-systems.md Appendix C or src/db/clickhouse/schema.rs",
    );
}

#[test]
fn l2_minute_klines_columns_match() {
    let doc = parse_doc_block("CLICKHOUSE_TABLE", "minute_klines");
    let src = parse_clickhouse_columns(CLICKHOUSE_SCHEMA, "minute_klines");
    assert_field_sets_equal(
        doc,
        src,
        "minute_klines",
        "update docs/contracts/external-systems.md Appendix C or src/db/clickhouse/schema.rs",
    );
}

#[test]
fn l2_minute_shares_columns_match() {
    let doc = parse_doc_block("CLICKHOUSE_TABLE", "minute_shares");
    let src = parse_clickhouse_columns(CLICKHOUSE_SCHEMA, "minute_shares");
    assert_field_sets_equal(
        doc,
        src,
        "minute_shares",
        "update docs/contracts/external-systems.md Appendix C or src/db/clickhouse/schema.rs",
    );
}

#[test]
fn l2_import_state_columns_match() {
    let doc = parse_doc_block("CLICKHOUSE_TABLE", "import_state");
    let src = parse_import_state_columns_from_doc_comment();
    assert_field_sets_equal(
        doc,
        src,
        "import_state",
        "update docs/contracts/external-systems.md Appendix C or src/tasks/openstock_import/state.rs DDL doc-comment",
    );
}

// ---------------------------------------------------------------------------
// Bridge struct field tests.
// ---------------------------------------------------------------------------

#[test]
fn l2_bridge_qmt_order_request_fields_match() {
    let doc = parse_doc_block("BRIDGE_STRUCT", "BridgeQmtOrderRequest");
    let src = parse_struct_fields(BRIDGE_MODELS, "BridgeQmtOrderRequest");
    assert_field_sets_equal(
        doc,
        src,
        "BridgeQmtOrderRequest",
        "update docs/contracts/external-systems.md Appendix C or src/bridge/models.rs",
    );
}

#[test]
fn l2_bridge_task_execute_request_fields_match() {
    let doc = parse_doc_block("BRIDGE_STRUCT", "BridgeTaskExecuteRequest");
    let src = parse_struct_fields(BRIDGE_MODELS, "BridgeTaskExecuteRequest");
    assert_field_sets_equal(
        doc,
        src,
        "BridgeTaskExecuteRequest",
        "update docs/contracts/external-systems.md Appendix C or src/bridge/models.rs",
    );
}

#[test]
fn l2_bridge_kline_bar_payload_fields_match() {
    let doc = parse_doc_block("BRIDGE_STRUCT", "BridgeKlineBarPayload");
    let src = parse_struct_fields(BRIDGE_MODELS, "BridgeKlineBarPayload");
    assert_field_sets_equal(
        doc,
        src,
        "BridgeKlineBarPayload",
        "update docs/contracts/external-systems.md Appendix C or src/bridge/models.rs",
    );
}

// ---------------------------------------------------------------------------
// OpenStock struct field tests.
// ---------------------------------------------------------------------------

#[test]
fn l2_openstock_envelope_fields_match() {
    let doc = parse_doc_block("OPENSTOCK_STRUCT", "OpenStockEnvelope");
    // OpenStockEnvelope is generic: `pub struct OpenStockEnvelope<T> {`.
    // The generic struct parser handles the brace matching.
    let src = parse_struct_fields(OPENSTOCK_ENVELOPE, "OpenStockEnvelope");
    assert_field_sets_equal(
        doc,
        src,
        "OpenStockEnvelope",
        "update docs/contracts/external-systems.md Appendix C or src/sources/openstock_envelope.rs",
    );
}

#[test]
fn l2_openstock_kline_fields_match() {
    let doc = parse_doc_block("OPENSTOCK_STRUCT", "Kline");
    let src = parse_struct_fields(DATA_MODELS, "Kline");
    assert_field_sets_equal(
        doc,
        src,
        "Kline",
        "update docs/contracts/external-systems.md Appendix C or src/data/models.rs",
    );
}

// ===========================================================================
// Layer 2b — Type-class sync (catches silent drift: Decimal→Float64, i64→f64,
// DateTime↔String). Per spec §字段语义变更 = 破坏性变更.
//
// Doc side: the second line inside each `<!-- L2:TAG -->` code block (after
// the field-name line) carries one type-class token per field. Token set:
//   Time | String | Float | Int | Decimal | Date | Bool | Custom | Custom(...)
//
// Source side: parse ClickHouse DDL column types / Rust struct field types
// and bucket them into the same token set.
// ===========================================================================

/// Parse the type-class line (second non-fence line) from an Appendix C block.
fn parse_doc_type_classes(tag: &str, name: &str) -> Vec<String> {
    let open = format!("<!-- L2:{tag} name={name} -->");
    let close = "<!-- /L2 -->";
    let start = DOC
        .find(&open)
        .unwrap_or_else(|| panic!("{open} not found"));
    let rest = &DOC[start + open.len()..];
    let end = rest
        .find(close)
        .unwrap_or_else(|| panic!("close marker for {open}"));
    let body = &rest[..end];
    let mut lines = body
        .trim()
        .lines()
        .filter(|line| !line.trim().starts_with("```"))
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let _field_line = lines.next().expect("missing field-name line");
    lines
        .next()
        .unwrap_or_else(|| panic!("missing type-class line in {open}"))
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Parse the field-name line (first non-fence line) from an Appendix C block.
fn parse_doc_field_names(tag: &str, name: &str) -> Vec<String> {
    let open = format!("<!-- L2:{tag} name={name} -->");
    let close = "<!-- /L2 -->";
    let start = DOC
        .find(&open)
        .unwrap_or_else(|| panic!("{open} not found"));
    let rest = &DOC[start + open.len()..];
    let end = rest
        .find(close)
        .unwrap_or_else(|| panic!("close marker for {open}"));
    let body = &rest[..end];
    let mut lines = body
        .trim()
        .lines()
        .filter(|line| !line.trim().starts_with("```"))
        .map(str::trim)
        .filter(|line| !line.is_empty());
    lines
        .next()
        .unwrap_or_else(|| panic!("missing field-name line in {open}"))
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Map a ClickHouse DDL type string to a type-class token.
fn classify_clickhouse_type(raw: &str) -> String {
    let upper = raw.to_uppercase();
    // Strip Nullable(...) to its inner type before classification, so
    // `Nullable(Decimal(...))` classifies as Decimal, not Custom.
    let inner = if let Some(rest) = upper
        .strip_prefix("NULLABLE(")
        .and_then(|s| s.strip_suffix(')'))
    {
        rest.trim()
    } else {
        &upper
    };
    let bare = inner.split('(').next().unwrap_or("").trim();
    match bare {
        "DATETIME" | "DATETIME64" | "TIMESTAMP" => "Time".to_string(),
        "DATE" => "Date".to_string(),
        "STRING" | "FIXEDSTRING" | "UUID" => "String".to_string(),
        "FLOAT32" | "FLOAT64" => "Float".to_string(),
        "INT8" | "INT16" | "INT32" | "INT64" | "UINT8" | "UINT16" | "UINT32" | "UINT64"
        | "INTEGER" | "BIGINT" => "Int".to_string(),
        "DECIMAL" => "Decimal".to_string(),
        _ if bare.starts_with("DECIMAL") => "Decimal".to_string(),
        _ => "Custom".to_string(),
    }
}

/// Map a PostgreSQL DDL type string to a type-class token.
/// Same token set as `classify_clickhouse_type`: Time | String | Float | Int |
/// Decimal | Date | Bool | Custom.
fn classify_postgres_type(raw: &str) -> String {
    let upper = raw.to_uppercase();
    let bare = upper.split('(').next().unwrap_or("").trim();
    match bare {
        "TIMESTAMPTZ"
        | "TIMESTAMP"
        | "TIMESTAMP_WITHOUT_TIME_ZONE"
        | "TIMESTAMP_WITH_TIME_ZONE" => "Time".to_string(),
        "DATE" => "Date".to_string(),
        "VARCHAR" | "TEXT" | "CHAR" | "BPCHAR" | "CITEXT" | "UUID" => "String".to_string(),
        "REAL" | "FLOAT4" | "DOUBLE_PRECISION" | "FLOAT8" => "Float".to_string(),
        "SMALLINT" | "INT2" | "INTEGER" | "INT" | "INT4" | "BIGINT" | "INT8" | "SERIAL"
        | "BIGSERIAL" => "Int".to_string(),
        "DECIMAL" | "NUMERIC" => "Decimal".to_string(),
        "BOOLEAN" | "BOOL" => "Bool".to_string(),
        _ if bare.starts_with("DECIMAL") || bare.starts_with("NUMERIC") => "Decimal".to_string(),
        _ => "Custom".to_string(),
    }
}

/// Parse `field:type` pairs from the import_state DDL doc-comment in
/// `state.rs`. The DDL lives in a `///` block, not a real SQL string.
fn parse_import_state_field_types() -> Vec<(String, String)> {
    // Reuse the existing body-extraction logic from
    // `parse_import_state_columns_from_doc_comment`, but capture type too.
    let needle = "CREATE TABLE quantix.import_state (";
    let start = IMPORT_STATE_STORE
        .find(needle)
        .unwrap_or_else(|| panic!("import_state DDL not found in state.rs"));
    let after = &IMPORT_STATE_STORE[start..];
    let open_paren = after.find('(').unwrap();
    let mut depth: i32 = 0;
    let mut close_idx = None;
    for (i, ch) in after[open_paren..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(open_paren + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = close_idx.unwrap();
    let body = &after[open_paren + 1..close_idx];

    let mut columns: Vec<String> = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    for ch in body.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                columns.push(std::mem::take(&mut current));
            }
            other => current.push(other),
        }
    }
    if !current.trim().is_empty() {
        columns.push(current);
    }

    columns
        .into_iter()
        .filter_map(|raw| {
            let mut line = raw.trim().to_string();
            while line.starts_with("///") {
                line = line[3..].trim_start().to_string();
            }
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let upper = line.to_uppercase();
            if upper.starts_with("PRIMARY")
                || upper.starts_with("CHECK")
                || upper.starts_with("CONSTRAINT")
            {
                return None;
            }
            let mut tokens = line.split_whitespace();
            let name = tokens.next()?.to_string();
            if !name
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_')
                .unwrap_or(false)
            {
                return None;
            }
            let type_token = tokens.next()?;
            // Strip NOT NULL / DEFAULT / etc. — classify_postgres_type takes
            // only the type name.
            Some((name, classify_postgres_type(type_token)))
        })
        .collect()
}

/// Map a Rust struct field type to a type-class token.
/// Bare types: String, f32/f64, i8..i64/u8..u64, bool. Custom types → Custom.
fn classify_rust_type(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches(',');
    // Strip `Option<...>` to its inner for classification.
    let inner = if let Some(rest) = trimmed
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        rest.trim()
    } else {
        trimmed
    };
    match inner {
        "String" | "&str" | "&'static str" => "String".to_string(),
        "f32" | "f64" => "Float".to_string(),
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize" | "isize" => {
            "Int".to_string()
        }
        "bool" => "Bool".to_string(),
        "Decimal" | "rust_decimal::Decimal" => "Decimal".to_string(),
        // Custom(type) form is preserved as Custom; downstream compares as "Custom".
        _ if inner.starts_with("Custom(") || inner == "Custom" => "Custom".to_string(),
        // NaiveDate / NaiveDateTime / DateTime / OffsetDateTime → Date / Time
        "NaiveDate" | "chrono::NaiveDate" => "Date".to_string(),
        "NaiveDateTime"
        | "chrono::NaiveDateTime"
        | "DateTime"
        | "chrono::DateTime<Utc>"
        | "OffsetDateTime"
        | "time::OffsetDateTime" => "Time".to_string(),
        // Everything else (Vec<...>, serde_json::Value, named struct types,
        // generic params) collapses to Custom — by spec, these are flagged
        // for manual review when they change.
        _ => "Custom".to_string(),
    }
}

/// Parse `field:type` pairs from a CREATE TABLE block.
fn parse_clickhouse_field_types(src: &str, table: &str) -> Vec<(String, String)> {
    let needle = format!("CREATE TABLE IF NOT EXISTS {table} ");
    let start = src
        .find(&needle)
        .unwrap_or_else(|| panic!("table {table} not found"));
    let after = &src[start..];
    let open = after.find('(').unwrap();
    let mut depth: i32 = 0;
    let mut close = None;
    for (i, ch) in after[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.unwrap();
    let body = &after[open + 1..close];

    let mut out: Vec<(String, String)> = Vec::new();
    let mut depth: i32 = 0;
    let mut cur = String::new();
    for ch in body.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                if let Some(pair) = parse_clickhouse_column(&cur) {
                    out.push(pair);
                }
                cur.clear();
            }
            other => cur.push(other),
        }
    }
    if let Some(pair) = parse_clickhouse_column(&cur) {
        out.push(pair);
    }
    out
}

fn parse_clickhouse_column(raw: &str) -> Option<(String, String)> {
    let line = raw.trim();
    if line.is_empty() {
        return None;
    }
    let upper = line.to_uppercase();
    if upper.starts_with("ENGINE")
        || upper.starts_with("PARTITION")
        || upper.starts_with("ORDER")
        || upper.starts_with("SETTINGS")
    {
        return None;
    }
    let mut tokens = line.split_whitespace();
    let name = tokens.next()?;
    let rest: String = tokens.collect::<Vec<_>>().join(" ");
    if rest.is_empty() {
        return None;
    }
    // `date MATERIALIZED toDate(timestamp)` — derive the type from the
    // expression: toDate(...) → Date. We include MATERIALIZED columns
    // because the doc lists them and the field-set test covers them.
    if let Some(expr) = rest.strip_prefix("MATERIALIZED ") {
        let derived = if expr.trim().starts_with("toDate(") {
            "Date"
        } else if expr.trim().starts_with("toDateTime(") {
            "DateTime"
        } else {
            // Unknown MATERIALIZED expression — surface as Custom for review.
            return Some((name.to_string(), "Custom".to_string()));
        };
        return Some((name.to_string(), classify_clickhouse_type(derived)));
    }
    let type_segment = rest
        .split_whitespace()
        .take_while(|t| !matches!(*t, "MATERIALIZED" | "DEFAULT" | "CODEC" | "TTL"))
        .collect::<Vec<_>>()
        .join(" ");
    if type_segment.is_empty() {
        return None;
    }
    let type_token = type_segment.split_whitespace().next()?;
    Some((name.to_string(), classify_clickhouse_type(type_token)))
}

/// Parse `pub field: type` pairs from a struct body.
fn parse_struct_field_types(src: &str, struct_name: &str) -> Vec<(String, String)> {
    let needle = format!("pub struct {struct_name}");
    let start = src
        .find(&needle)
        .unwrap_or_else(|| panic!("struct {struct_name} not found"));
    let after = &src[start..];
    let open = after.find('{').unwrap();
    let mut depth: i32 = 0;
    let mut close = None;
    for (i, ch) in after[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.unwrap();
    let body = &after[open + 1..close];

    let mut out: Vec<(String, String)> = Vec::new();
    for raw_line in body.lines() {
        let line = raw_line.trim();
        if !line.starts_with("pub ") || line.starts_with("pub struct") || line.starts_with("#[") {
            continue;
        }
        let line = line.trim_end_matches(',');
        let after_pub = &line[4..];
        let colon = match after_pub.find(':') {
            Some(c) => c,
            None => continue,
        };
        let name = after_pub[..colon].trim();
        let type_str = after_pub[colon + 1..].trim();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        out.push((name.to_string(), classify_rust_type(type_str)));
    }
    out
}

fn assert_type_classes_match(
    doc_names: &[String],
    doc_classes: &[String],
    src_pairs: &[(String, String)],
    context: &str,
    hint: &str,
) {
    use std::collections::HashMap;
    if doc_names.len() != doc_classes.len() {
        panic!(
            "\n[doc-internal mismatch] {}: doc names {} ≠ doc type-classes {} — \
             Appendix C second-line token count must match field count\n",
            context,
            doc_names.len(),
            doc_classes.len()
        );
    }
    let doc_map: HashMap<&str, &String> = doc_names
        .iter()
        .zip(doc_classes.iter())
        .map(|(n, c)| (n.as_str(), c))
        .collect();
    let src_map: HashMap<&str, &String> = src_pairs.iter().map(|(n, c)| (n.as_str(), c)).collect();

    if doc_map.len() != doc_names.len() {
        panic!(
            "\n[doc-internal mismatch] {}: doc field-name line has duplicates — {:?}",
            context, doc_names
        );
    }

    let mut missing_in_doc: Vec<String> = src_map
        .keys()
        .filter(|k| !doc_map.contains_key(*k))
        .map(|k| k.to_string())
        .collect();
    missing_in_doc.sort();
    let mut missing_in_src: Vec<String> = doc_map
        .keys()
        .filter(|k| !src_map.contains_key(*k))
        .map(|k| k.to_string())
        .collect();
    missing_in_src.sort();

    let mut mismatches: Vec<String> = Vec::new();
    let mut names: Vec<&str> = src_map.keys().copied().collect();
    names.sort_unstable();
    for name in &names {
        let src_class = src_map[name];
        let doc_class = match doc_map.get(name) {
            Some(c) => c.as_str(),
            None => continue,
        };
        if src_class != doc_class {
            mismatches.push(format!(
                "{context}.{name}: doc={doc_class} source={src_class}"
            ));
        }
    }

    let mut parts: Vec<String> = Vec::new();
    if !missing_in_doc.is_empty() {
        parts.push(format!("missing in doc: {missing_in_doc:?}"));
    }
    if !missing_in_src.is_empty() {
        parts.push(format!("missing in source: {missing_in_src:?}"));
    }
    if !mismatches.is_empty() {
        parts.push(format!(
            "type-class divergences:\n  - {}",
            mismatches.join("\n  - ")
        ));
    }
    if !parts.is_empty() {
        panic!(
            "\n[type-class mismatch] {}\n{}\n  Hint: {}\n",
            context,
            parts.join("\n"),
            hint
        );
    }
}

// --- ClickHouse table type-class tests ------------------------------------

#[test]
fn l2b_kline_data_type_classes_match() {
    let doc_names = parse_doc_field_names("CLICKHOUSE_TABLE", "kline_data");
    let doc_classes = parse_doc_type_classes("CLICKHOUSE_TABLE", "kline_data");
    let src = parse_clickhouse_field_types(CLICKHOUSE_SCHEMA, "kline_data");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "kline_data",
        "Decimal→Float64 / Int→Float / DateTime↔String are silent-drift breakers; bump Major",
    );
}

#[test]
fn l2b_minute_klines_type_classes_match() {
    let doc_names = parse_doc_field_names("CLICKHOUSE_TABLE", "minute_klines");
    let doc_classes = parse_doc_type_classes("CLICKHOUSE_TABLE", "minute_klines");
    let src = parse_clickhouse_field_types(CLICKHOUSE_SCHEMA, "minute_klines");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "minute_klines",
        "Decimal→Float64 / Int→Float / DateTime↔String are silent-drift breakers; bump Major",
    );
}

#[test]
fn l2b_minute_shares_type_classes_match() {
    let doc_names = parse_doc_field_names("CLICKHOUSE_TABLE", "minute_shares");
    let doc_classes = parse_doc_type_classes("CLICKHOUSE_TABLE", "minute_shares");
    let src = parse_clickhouse_field_types(CLICKHOUSE_SCHEMA, "minute_shares");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "minute_shares",
        "Decimal→Float64 / Int→Float / DateTime↔String are silent-drift breakers; bump Major",
    );
}

#[test]
fn l2b_bridge_qmt_order_request_type_classes_match() {
    let doc_names = parse_doc_field_names("BRIDGE_STRUCT", "BridgeQmtOrderRequest");
    let doc_classes = parse_doc_type_classes("BRIDGE_STRUCT", "BridgeQmtOrderRequest");
    let src = parse_struct_field_types(BRIDGE_MODELS, "BridgeQmtOrderRequest");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "BridgeQmtOrderRequest",
        "quantity Int→Float / price Custom(String)→Float are silent-drift breakers; bump Major",
    );
}

#[test]
fn l2b_bridge_kline_bar_payload_type_classes_match() {
    let doc_names = parse_doc_field_names("BRIDGE_STRUCT", "BridgeKlineBarPayload");
    let doc_classes = parse_doc_type_classes("BRIDGE_STRUCT", "BridgeKlineBarPayload");
    let src = parse_struct_field_types(BRIDGE_MODELS, "BridgeKlineBarPayload");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "BridgeKlineBarPayload",
        "volume Int→Float / open Float→Int are silent-drift breakers; bump Major",
    );
}

#[test]
fn l2b_openstock_envelope_type_classes_match() {
    let doc_names = parse_doc_field_names("OPENSTOCK_STRUCT", "OpenStockEnvelope");
    let doc_classes = parse_doc_type_classes("OPENSTOCK_STRUCT", "OpenStockEnvelope");
    let src = parse_struct_field_types(OPENSTOCK_ENVELOPE, "OpenStockEnvelope");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "OpenStockEnvelope",
        "latency_ms Int→Float / received_at Time→String are silent-drift breakers; bump Major",
    );
}

#[test]
fn l2b_kline_type_classes_match() {
    let doc_names = parse_doc_field_names("OPENSTOCK_STRUCT", "Kline");
    let doc_classes = parse_doc_type_classes("OPENSTOCK_STRUCT", "Kline");
    let src = parse_struct_field_types(DATA_MODELS, "Kline");
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "Kline",
        "open/high/low/close Decimal→Float are silent-precision-loss breakers; bump Major",
    );
}

#[test]
fn l2b_import_state_type_classes_match() {
    let doc_names = parse_doc_field_names("CLICKHOUSE_TABLE", "import_state");
    let doc_classes = parse_doc_type_classes("CLICKHOUSE_TABLE", "import_state");
    let src = parse_import_state_field_types();
    assert_type_classes_match(
        &doc_names,
        &doc_classes,
        &src,
        "import_state",
        "imported_at Time→String / status Int→String drift would silently break idempotency; bump Major",
    );
}
