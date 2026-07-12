use super::*;

#[test]
fn importer_reuses_date_parser_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn parse_date(value: &str, format: &str) -> Result<NaiveDate>"),
        "expected {source_path} to define one reusable importer date parser helper"
    );
    assert!(
        source.contains("let date = parse_date(&row.date, &self.config.date_format)?;"),
        "expected csv_row_to_kline in {source_path} to use the reusable date parser"
    );
    assert!(
        source.contains("let date = parse_date(&row.date, \"%Y-%m-%d\")?;"),
        "expected json_row_to_kline in {source_path} to use the reusable date parser"
    );

    let csv_row_start = source
        .find("fn csv_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define csv_row_to_kline"));
    let csv_row_body = &source[csv_row_start
        ..source[csv_row_start..]
            .find("fn json_row_to_kline")
            .map(|offset| csv_row_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !csv_row_body.contains("NaiveDate::parse_from_str"),
        "expected csv_row_to_kline in {source_path} not to inline date parsing"
    );

    let json_row_start = source
        .find("fn json_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define json_row_to_kline"));
    let json_row_body = &source[json_row_start
        ..source[json_row_start..]
            .find("fn parse_date")
            .map(|offset| json_row_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !json_row_body.contains("NaiveDate::parse_from_str"),
        "expected json_row_to_kline in {source_path} not to inline date parsing"
    );
}

#[test]
fn importer_reuses_required_decimal_parser_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains(
            "fn parse_required_decimal(value: &str, field_name: &str) -> Result<Decimal>"
        ),
        "expected {source_path} to define one reusable required decimal parser helper"
    );

    for expected_call in [
        "parse_required_decimal(&row.open, \"open\")?",
        "parse_required_decimal(&row.high, \"high\")?",
        "parse_required_decimal(&row.low, \"low\")?",
        "parse_required_decimal(&row.close, \"close\")?",
        ".map(|s| parse_required_decimal(s, \"amount\"))",
    ] {
        assert!(
            source.contains(expected_call),
            "expected {source_path} to contain `{expected_call}`"
        );
    }

    let csv_row_start = source
        .find("fn csv_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define csv_row_to_kline"));
    let csv_row_body = &source[csv_row_start
        ..source[csv_row_start..]
            .find("fn json_row_to_kline")
            .map(|offset| csv_row_start + offset)
            .unwrap_or(source.len())];
    for inline_parse in [
        "Decimal::from_str(&row.open)",
        "Decimal::from_str(&row.high)",
        "Decimal::from_str(&row.low)",
        "Decimal::from_str(&row.close)",
        "Decimal::from_str(s).map_err",
    ] {
        assert!(
            !csv_row_body.contains(inline_parse),
            "expected csv_row_to_kline in {source_path} not to inline `{inline_parse}`"
        );
    }

    let json_row_start = source
        .find("fn json_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define json_row_to_kline"));
    let json_row_body = &source[json_row_start
        ..source[json_row_start..]
            .find("fn parse_date")
            .map(|offset| json_row_start + offset)
            .unwrap_or(source.len())];
    for inline_parse in [
        "Decimal::from_str(&row.open)",
        "Decimal::from_str(&row.close)",
    ] {
        assert!(
            !json_row_body.contains(inline_parse),
            "expected json_row_to_kline in {source_path} not to inline `{inline_parse}`"
        );
    }
}

#[test]
fn importer_reuses_optional_decimal_default_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn parse_optional_decimal_or_default(value: Option<&str>) -> Decimal"),
        "expected {source_path} to define one reusable optional decimal default helper"
    );
    for expected_call in [
        "high: parse_optional_decimal_or_default(row.high.as_deref()),",
        "low: parse_optional_decimal_or_default(row.low.as_deref()),",
    ] {
        assert!(
            source.contains(expected_call),
            "expected {source_path} to contain `{expected_call}`"
        );
    }

    let json_row_start = source
        .find("fn json_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define json_row_to_kline"));
    let json_row_body = &source[json_row_start
        ..source[json_row_start..]
            .find("fn parse_date")
            .map(|offset| json_row_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !json_row_body.contains(".map(|s| Decimal::from_str(s).unwrap_or_default())"),
        "expected json_row_to_kline in {source_path} not to inline optional decimal defaults"
    );
}

#[test]
fn importer_reuses_parquet_date_conversion_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn date_from_parquet_day(days: i32) -> Result<NaiveDate>"),
        "expected {source_path} to define one reusable Parquet Date32 conversion helper"
    );

    let import_parquet_start = source
        .find("fn import_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define import_parquet"));
    let import_parquet_end = [
        "fn csv_row_to_kline",
        "fn json_row_to_kline",
        "fn parse_date",
        "fn date_from_parquet_day",
        "fn parse_required_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[import_parquet_start..]
            .find(marker)
            .map(|offset| import_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let import_parquet_body = &source[import_parquet_start..import_parquet_end];

    assert!(
        import_parquet_body.contains("date_from_parquet_day(dates.value(i))?"),
        "expected import_parquet in {source_path} to call date_from_parquet_day"
    );
    assert!(
        !import_parquet_body.contains("NaiveDate::from_ymd_opt(1970, 1, 1)"),
        "expected import_parquet in {source_path} not to inline Parquet Date32 epoch conversion"
    );
    assert!(
        !import_parquet_body.contains("chrono::Duration::days("),
        "expected import_parquet in {source_path} not to inline Parquet Date32 day offsets"
    );
}

#[test]
fn importer_reuses_parquet_decimal_conversion_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn decimal_from_f64_or_default(value: f64) -> Decimal"),
        "expected {source_path} to define one reusable Parquet f64 Decimal conversion helper"
    );

    let import_parquet_start = source
        .find("fn import_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define import_parquet"));
    let import_parquet_end = [
        "fn csv_row_to_kline",
        "fn json_row_to_kline",
        "fn parse_date",
        "fn decimal_from_f64_or_default",
        "fn parse_required_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[import_parquet_start..]
            .find(marker)
            .map(|offset| import_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let import_parquet_body = &source[import_parquet_start..import_parquet_end];

    for expected_call in [
        "decimal_from_f64_or_default(opens.value(i))",
        "decimal_from_f64_or_default(0.0)",
        "decimal_from_f64_or_default(closes.value(i))",
    ] {
        assert!(
            import_parquet_body.contains(expected_call),
            "expected import_parquet in {source_path} to call `{expected_call}`"
        );
    }

    assert!(
        !import_parquet_body.contains("Decimal::from_f64_retain("),
        "expected import_parquet in {source_path} not to inline Parquet f64 Decimal conversion"
    );
}

#[test]
fn exporter_reuses_decimal_formatter_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn format_decimal(value: Decimal, precision: usize) -> String"),
        "expected {source_path} to define one reusable CSV decimal formatter helper"
    );
    for expected_call in [
        "format_decimal(kline.open, decimal_precision)",
        "format_decimal(kline.high, decimal_precision)",
        "format_decimal(kline.low, decimal_precision)",
        "format_decimal(kline.close, decimal_precision)",
    ] {
        assert!(
            source.contains(expected_call),
            "expected {source_path} to contain `{expected_call}`"
        );
    }

    let export_csv_start = source
        .find("fn export_csv")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_csv"));
    let export_csv_body = &source[export_csv_start
        ..source[export_csv_start..]
            .find("fn export_json")
            .map(|offset| export_csv_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !export_csv_body.contains("\"{:.prec$}\""),
        "expected export_csv in {source_path} not to inline Decimal precision formatting"
    );
}

#[test]
fn exporter_reuses_csv_kline_record_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains(
            "fn csv_kline_record(kline: &Kline, decimal_precision: usize) -> [String; 9]"
        ),
        "expected {source_path} to define one reusable CSV kline record helper"
    );
    assert!(
        source.contains("csv_kline_record(kline, self.config.decimal_precision)"),
        "expected export_csv in {source_path} to use the CSV kline record helper"
    );

    let export_csv_start = source
        .find("fn export_csv")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_csv"));
    let export_csv_body = &source[export_csv_start
        ..source[export_csv_start..]
            .find("fn export_json")
            .map(|offset| export_csv_start + offset)
            .unwrap_or(source.len())];

    for inline_detail in [
        "kline.date.to_string()",
        "format_decimal(kline.open",
        "format_decimal(kline.high",
        "format_decimal(kline.low",
        "format_decimal(kline.close",
        "format!(\"{:?}\", kline.adjust_type)",
    ] {
        assert!(
            !export_csv_body.contains(inline_detail),
            "expected export_csv in {source_path} not to inline CSV record field `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_csv_kline_header_constant() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    let header_start = source
        .find("const CSV_KLINE_HEADER: [&str; 9] = [")
        .unwrap_or_else(|| {
            panic!("expected {source_path} to define one reusable CSV kline header constant")
        });
    let header_body = &source[header_start
        ..source[header_start..]
            .find("];")
            .map(|offset| header_start + offset)
            .unwrap_or(source.len())];
    for expected_header in [
        "\"code\"",
        "\"date\"",
        "\"open\"",
        "\"high\"",
        "\"low\"",
        "\"close\"",
        "\"volume\"",
        "\"amount\"",
        "\"adjust_type\"",
    ] {
        assert!(
            header_body.contains(expected_header),
            "expected CSV_KLINE_HEADER in {source_path} to contain `{expected_header}`"
        );
    }

    assert!(
        source.contains("wtr.write_record(CSV_KLINE_HEADER)"),
        "expected export_csv in {source_path} to use CSV_KLINE_HEADER"
    );

    let export_csv_start = source
        .find("fn export_csv")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_csv"));
    let export_csv_body = &source[export_csv_start
        ..source[export_csv_start..]
            .find("fn export_json")
            .map(|offset| export_csv_start + offset)
            .unwrap_or(source.len())];

    for inline_header in [
        "\"code\"",
        "\"date\"",
        "\"open\"",
        "\"high\"",
        "\"low\"",
        "\"close\"",
        "\"volume\"",
        "\"amount\"",
        "\"adjust_type\"",
    ] {
        assert!(
            !export_csv_body.contains(inline_header),
            "expected export_csv in {source_path} not to inline CSV header field `{inline_header}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_decimal_conversion_helpers() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    for expected_helper in [
        "fn decimal_to_f64_or_zero(value: Decimal) -> f64",
        "fn optional_decimal_to_f64_or_zero(value: Option<Decimal>) -> f64",
    ] {
        assert!(
            source.contains(expected_helper),
            "expected {source_path} to define `{expected_helper}`"
        );
    }

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = ["fn decimal_to_f64_or_zero", "fn format_decimal"]
        .iter()
        .filter_map(|marker| {
            source[export_parquet_start..]
                .find(marker)
                .map(|offset| export_parquet_start + offset)
        })
        .min()
        .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    for expected_call in [
        "decimal_to_f64_or_zero(k.open)",
        "decimal_to_f64_or_zero(k.high)",
        "decimal_to_f64_or_zero(k.low)",
        "decimal_to_f64_or_zero(k.close)",
        "optional_decimal_to_f64_or_zero(k.amount)",
    ] {
        assert!(
            export_parquet_body.contains(expected_call),
            "expected export_parquet in {source_path} to call `{expected_call}`"
        );
    }

    for inline_detail in [
        ".to_f64().unwrap_or(0.0)",
        ".map(|a| a.to_f64().unwrap_or(0.0)).unwrap_or(0.0)",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Decimal conversion `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_date_conversion_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn date_to_parquet_day(date: NaiveDate) -> i32"),
        "expected {source_path} to define one reusable Parquet Date32 conversion helper"
    );

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body.contains(".map(|k| date_to_parquet_day(k.date))"),
        "expected export_parquet in {source_path} to map dates through date_to_parquet_day"
    );
    for inline_detail in [
        "NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()",
        ".signed_duration_since(",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Parquet date conversion `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_schema_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn parquet_kline_schema() -> arrow::datatypes::Schema"),
        "expected {source_path} to define one reusable Parquet kline schema helper"
    );

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn parquet_kline_schema",
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body.contains("let schema = parquet_kline_schema();"),
        "expected export_parquet in {source_path} to use parquet_kline_schema"
    );
    for inline_detail in ["Schema::new(vec![", "Field::new("] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Parquet schema detail `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_record_batch_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    for expected_signature_part in [
        "fn parquet_kline_record_batch(",
        "schema: &arrow::datatypes::Schema",
        "klines: &[Kline]",
        ") -> Result<RecordBatch>",
    ] {
        assert!(
            source.contains(expected_signature_part),
            "expected {source_path} to define one reusable Parquet kline RecordBatch helper containing `{expected_signature_part}`"
        );
    }

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn parquet_kline_record_batch",
        "fn parquet_kline_schema",
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body.contains("let batch = parquet_kline_record_batch(&schema, klines)?;"),
        "expected export_parquet in {source_path} to use parquet_kline_record_batch"
    );
    for inline_detail in [
        "StringArray::from(",
        "Float64Array::from(",
        "PrimitiveArray::<",
        "RecordBatch::try_new(",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline RecordBatch detail `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_writer_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    for expected_signature_part in [
        "fn write_parquet_record_batch(",
        "path: &Path",
        "schema: arrow::datatypes::Schema",
        "batch: &RecordBatch",
        ") -> Result<()>",
    ] {
        assert!(
            source.contains(expected_signature_part),
            "expected {source_path} to define one reusable Parquet writer helper containing `{expected_signature_part}`"
        );
    }

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn write_parquet_record_batch",
        "fn parquet_kline_record_batch",
        "fn parquet_kline_schema",
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body
            .contains("write_parquet_record_batch(output_path.as_ref(), schema, &batch)"),
        "expected export_parquet in {source_path} to use write_parquet_record_batch"
    );
    for inline_detail in [
        "File::create(",
        "WriterProperties::builder()",
        "ArrowWriter::try_new(",
        "writer.write(",
        "writer.close()",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Parquet writer detail `{inline_detail}`"
        );
    }
}
