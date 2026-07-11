//! Fixture contract sync test for `tests/fixtures/openstock/*.json`.
//!
//! Spec: `docs/superpowers/specs/2026-07-11-external-systems-contract-design.md`
//! §2.5 测试夹具（Fixture Contract）. Hard-fail on any mismatch.
//!
//! Two layers:
//! 1. **Inventory match**: the `<!-- L2:FIXTURE_INVENTORY -->` block in
//!    `docs/contracts/external-systems.md` must list exactly the set of
//!    fixture files present under `tests/fixtures/openstock/`.
//! 2. **Envelope-shape parse**: every envelope-shape fixture (those with
//!    a top-level `data` array) must deserialize as `OpenStockEnvelope`
//!    and, if it declares `data_category`, that category must be one of
//!    the 5 P0 values documented in `openstock_envelope.rs`.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const DOC: &str = include_str!("../docs/contracts/external-systems.md");

/// The 5 P0 categories documented in `openstock_envelope.rs:30-32`.
const P0_CATEGORIES: &[&str] = &[
    "STOCK_CODES",
    "ALL_STOCKS",
    "TRADE_DATES",
    "WORKDAYS",
    "INDEX_KLINES",
];

// ---------------------------------------------------------------------------
// Doc-side parser: pull the fixture-inventory block.
// ---------------------------------------------------------------------------

fn parse_doc_inventory() -> HashSet<String> {
    let open = "<!-- L2:FIXTURE_INVENTORY -->";
    let close = "<!-- /L2 -->";
    let start = DOC.find(open).expect("FIXTURE_INVENTORY marker not found");
    let rest = &DOC[start + open.len()..];
    let end = rest
        .find(close)
        .unwrap_or_else(|| panic!("FIXTURE_INVENTORY block missing close"));
    let body = &rest[..end];
    body.trim()
        .lines()
        .find(|line| !line.trim().starts_with("```") && !line.trim().is_empty())
        .unwrap_or_else(|| panic!("FIXTURE_INVENTORY block has no file list"))
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

// ---------------------------------------------------------------------------
// Filesystem-side: list actual fixtures.
// ---------------------------------------------------------------------------

fn list_actual_fixtures() -> HashSet<String> {
    let dir = Path::new(ROOT).join("tests/fixtures/openstock");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("failed to read fixtures/openstock: {e}"));
    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? != "json" {
                return None;
            }
            path.file_name()?.to_str().map(str::to_string)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Envelope-shape parse — borrow the production envelope contract.
//
// We re-derive a minimal local envelope rather than `use`-ing the production
// type to keep this test independent of `src/sources/` internal refactors.
// The shape must match `OpenStockEnvelope<Value>` in `openstock_envelope.rs`.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FixtureEnvelope {
    /// Required: always a JSON array for P0 categories.
    data: Vec<Value>,
    #[serde(default)]
    data_category: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[test]
fn l2_fixture_inventory_matches_filesystem() {
    let doc = parse_doc_inventory();
    let fs_set = list_actual_fixtures();

    if doc == fs_set {
        return;
    }
    let mut missing_in_doc: Vec<_> = fs_set.difference(&doc).cloned().collect();
    missing_in_doc.sort();
    let mut missing_in_fs: Vec<_> = doc.difference(&fs_set).cloned().collect();
    missing_in_fs.sort();
    panic!(
        "\n[fixture inventory] mismatch\n\
         Missing in doc:        {missing_in_doc:?}\n\
         Missing on filesystem: {missing_in_fs:?}\n\
         Hint: update the `<!-- L2:FIXTURE_INVENTORY -->` block in §2.5 of\n\
         docs/contracts/external-systems.md to list the actual files in\n\
         tests/fixtures/openstock/.\n"
    );
}

#[test]
fn l2_fixture_envelope_shapes_parse_and_category_is_valid() {
    let dir = Path::new(ROOT).join("tests/fixtures/openstock");
    let valid_categories: HashSet<&str> = P0_CATEGORIES.iter().copied().collect();

    // Fixtures documented as shape B (legacy direct-records, not envelope).
    // These legitimately fail envelope parsing — list them explicitly.
    let legacy_shape: HashSet<&str> = ["daily_kline.json", "daily_kline_30d.json"]
        .into_iter()
        .collect();

    let mut failures: Vec<String> = Vec::new();

    for entry in fs::read_dir(&dir).expect("fixtures dir missing") {
        let entry = entry.expect("dirent ok");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        let body = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"));

        if legacy_shape.contains(name.as_str()) {
            // Sanity check: legacy fixtures must have a top-level `records` array,
            // NOT a `data` array — they are not envelope-shaped.
            let v: Value = serde_json::from_str(&body)
                .unwrap_or_else(|e| panic!("legacy {name} is not valid JSON: {e}"));
            if v.get("data").is_some() {
                failures.push(format!(
                    "{name}: documented as legacy shape B but contains top-level `data` — reclassify"
                ));
            }
            if v.get("records").is_none() {
                failures.push(format!(
                    "{name}: documented as legacy shape B but missing `records` array"
                ));
            }
            continue;
        }

        // Envelope-shape fixture.
        let env: FixtureEnvelope = match serde_json::from_str(&body) {
            Ok(env) => env,
            Err(e) => {
                failures.push(format!("{name}: failed envelope deserialize: {e}"));
                continue;
            }
        };

        // data array is required (already enforced by deserialization).
        if env.data.is_empty() && env.source.is_none() {
            failures.push(format!(
                "{name}: empty `data` array must still carry `source` (degenerate envelope)"
            ));
            continue;
        }

        if let Some(cat) = &env.data_category
            && !valid_categories.contains(cat.as_str())
        {
            failures.push(format!(
                "{name}: data_category `{cat}` is not in the 5 P0 categories {:?}",
                P0_CATEGORIES
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n[fixture envelope contract] {} failure(s):\n  - {}\n\
             Hint: see docs/contracts/external-systems.md §2.5 测试夹具.\n",
            failures.len(),
            failures.join("\n  - ")
        );
    }
}
