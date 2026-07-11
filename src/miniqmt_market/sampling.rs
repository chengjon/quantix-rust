//! Local payload sampling and hash verification helpers.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactPayloadSample {
    pub(super) computed_row_count: Option<u64>,
    pub(super) sample_symbols: Vec<String>,
    pub(super) sample_dates: Vec<String>,
}

const MAX_ARTIFACT_SAMPLE_VALUES: usize = 5;

pub(super) fn sample_artifact_payload(
    path: &Path,
    artifact_type: &str,
) -> Result<ArtifactPayloadSample, String> {
    match artifact_type {
        "parquet" => sample_parquet_payload(path),
        other => Err(format!(
            "artifact payload sampling unsupported for type {other}"
        )),
    }
}

fn sample_parquet_payload(path: &Path) -> Result<ArtifactPayloadSample, String> {
    use arrow::array::{Array, LargeStringArray, PrimitiveArray, StringArray};
    use arrow::datatypes::Date32Type;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let file = fs::File::open(path)
        .map_err(|err| format!("failed to open parquet artifact {}: {err}", path.display()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|err| format!("failed to create parquet reader {}: {err}", path.display()))?;
    let computed_row_count = Some(
        u64::try_from(builder.metadata().file_metadata().num_rows()).map_err(|err| {
            format!(
                "failed to read parquet row count from metadata {}: {err}",
                path.display()
            )
        })?,
    );
    let mut reader = builder
        .build()
        .map_err(|err| format!("failed to build parquet reader {}: {err}", path.display()))?;

    let mut sample = ArtifactPayloadSample {
        computed_row_count,
        ..ArtifactPayloadSample::default()
    };
    while !artifact_sample_is_full(&sample) {
        let Some(batch) = reader.next() else {
            break;
        };
        let batch = batch
            .map_err(|err| format!("failed to read parquet batch {}: {err}", path.display()))?;

        if let Some(symbols) = batch
            .column_by_name("symbol")
            .or_else(|| batch.column_by_name("code"))
            .or_else(|| batch.column_by_name("ts_code"))
            .or_else(|| batch.column_by_name("ticker"))
        {
            if let Some(values) = symbols.as_any().downcast_ref::<StringArray>() {
                collect_string_samples(values, &mut sample.sample_symbols);
            } else if let Some(values) = symbols.as_any().downcast_ref::<LargeStringArray>() {
                collect_large_string_samples(values, &mut sample.sample_symbols);
            }
        }

        if let Some(dates) = batch
            .column_by_name("date")
            .or_else(|| batch.column_by_name("trade_date"))
            .or_else(|| batch.column_by_name("datetime"))
            .or_else(|| batch.column_by_name("timestamp"))
        {
            if let Some(values) = dates.as_any().downcast_ref::<PrimitiveArray<Date32Type>>() {
                collect_date32_samples(values, &mut sample.sample_dates);
            } else if let Some(values) = dates.as_any().downcast_ref::<StringArray>() {
                collect_string_samples(values, &mut sample.sample_dates);
            } else if let Some(values) = dates.as_any().downcast_ref::<LargeStringArray>() {
                collect_large_string_samples(values, &mut sample.sample_dates);
            }
        }
    }

    Ok(sample)
}

fn artifact_sample_is_full(sample: &ArtifactPayloadSample) -> bool {
    sample.sample_symbols.len() >= MAX_ARTIFACT_SAMPLE_VALUES
        && sample.sample_dates.len() >= MAX_ARTIFACT_SAMPLE_VALUES
}

fn collect_string_samples(values: &arrow::array::StringArray, output: &mut Vec<String>) {
    use arrow::array::Array;

    for row in 0..values.len() {
        if values.is_null(row) {
            continue;
        }
        push_unique_sample(output, values.value(row));
        if output.len() >= MAX_ARTIFACT_SAMPLE_VALUES {
            break;
        }
    }
}

fn collect_large_string_samples(values: &arrow::array::LargeStringArray, output: &mut Vec<String>) {
    use arrow::array::Array;

    for row in 0..values.len() {
        if values.is_null(row) {
            continue;
        }
        push_unique_sample(output, values.value(row));
        if output.len() >= MAX_ARTIFACT_SAMPLE_VALUES {
            break;
        }
    }
}

fn collect_date32_samples(
    values: &arrow::array::PrimitiveArray<arrow::datatypes::Date32Type>,
    output: &mut Vec<String>,
) {
    use arrow::array::Array;

    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date");
    for row in 0..values.len() {
        if values.is_null(row) {
            continue;
        }
        if let Some(date) =
            epoch.checked_add_signed(chrono::Duration::days(values.value(row) as i64))
        {
            push_unique_sample(output, &date.format("%Y-%m-%d").to_string());
        }
        if output.len() >= MAX_ARTIFACT_SAMPLE_VALUES {
            break;
        }
    }
}

fn push_unique_sample(output: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if value.is_empty()
        || output.len() >= MAX_ARTIFACT_SAMPLE_VALUES
        || output.iter().any(|existing| existing == value)
    {
        return;
    }
    output.push(value.to_string());
}

pub(super) fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|err| format!("failed to open artifact file {}: {err}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| format!("failed to read artifact file {}: {err}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn normalize_sha256_hash(hash: &str) -> &str {
    hash.strip_prefix("sha256:").unwrap_or(hash)
}

pub(super) fn format_sha256_hash_like_expected(expected_hash: &str, computed_hash: &str) -> String {
    if expected_hash.starts_with("sha256:") {
        format!("sha256:{computed_hash}")
    } else {
        computed_hash.to_string()
    }
}

pub(super) fn file_uri_local_path_candidates(uri: &str) -> Result<Vec<PathBuf>, String> {
    let raw_path = uri
        .strip_prefix("file://")
        .ok_or_else(|| format!("artifact uri is not a file uri: {uri}"))?;
    if raw_path.is_empty() {
        return Err("artifact file uri is empty".to_string());
    }

    let decoded = urlencoding::decode(raw_path)
        .map_err(|err| format!("failed to decode artifact file uri {uri}: {err}"))?
        .into_owned();

    if let Some((drive, tail)) = windows_drive_uri_tail(&decoded) {
        let drive = drive.to_ascii_lowercase();
        let tail = tail.trim_start_matches(['/', '\\']);
        return Ok(vec![
            PathBuf::from(format!("/mnt/{drive}/{tail}")),
            PathBuf::from(format!("/{drive}/{tail}")),
            PathBuf::from(decoded),
        ]);
    }

    Ok(vec![PathBuf::from(decoded)])
}

fn windows_drive_uri_tail(path: &str) -> Option<(char, &str)> {
    let normalized = path.strip_prefix('/').unwrap_or(path);
    let mut chars = normalized.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() || chars.next()? != ':' {
        return None;
    }
    let separator = chars.next()?;
    if separator != '/' && separator != '\\' {
        return None;
    }
    Some((drive, chars.as_str()))
}
