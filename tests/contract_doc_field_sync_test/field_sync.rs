use super::helpers::*;

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
