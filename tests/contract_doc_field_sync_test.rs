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
