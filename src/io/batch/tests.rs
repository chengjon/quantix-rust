//! 批处理器行为测试，从原生 `batch.rs` 的内联 `mod tests` 拆出。

use super::BatchProcessor;
use super::types::{BatchConfig, BatchProgress};
use crate::data::models::AdjustType;
use chrono::NaiveDate;
use rust_decimal_macros::dec;
use std::time::Duration;

fn create_test_klines(count: usize) -> Vec<crate::data::models::Kline> {
    (0..count)
        .map(|i| crate::data::models::Kline {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .checked_add_signed(chrono::Duration::days(i as i64))
                .unwrap(),
            open: dec!(10.0),
            high: dec!(11.0),
            low: dec!(9.0),
            close: dec!(10.5),
            volume: 1000000,
            amount: Some(dec!(10500000)),
            adjust_type: AdjustType::None,
        })
        .collect()
}

#[test]
fn test_batch_config_default() {
    let config = BatchConfig::default();
    assert_eq!(config.batch_size, 1000);
    assert_eq!(config.max_concurrent_tasks, 4);
}

#[test]
fn batch_processor_rejects_zero_batch_size() {
    let result = BatchProcessor::new(BatchConfig {
        batch_size: 0,
        enable_progress: false,
        ..Default::default()
    });

    let error = match result {
        Ok(_) => panic!("expected zero batch_size to be rejected"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("batch_size"));
}

#[test]
fn batch_processor_rejects_zero_max_concurrent_tasks() {
    let result = BatchProcessor::new(BatchConfig {
        max_concurrent_tasks: 0,
        enable_progress: false,
        ..Default::default()
    });

    let error = match result {
        Ok(_) => panic!("expected zero max_concurrent_tasks to be rejected"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("max_concurrent_tasks"));
}

#[test]
fn test_batch_progress() {
    let mut progress = BatchProgress::new(100, 10);
    assert_eq!(progress.total_records, 100);
    assert_eq!(progress.total_batches, 10);
    assert_eq!(progress.progress_percent(), 0.0);

    progress.update(10, 0);
    assert_eq!(progress.progress_percent(), 10.0);
}

#[test]
fn test_progress_complete() {
    let mut progress = BatchProgress::new(100, 10);
    progress.update(50, 0);
    progress.update(50, 0);

    assert!(progress.is_complete());
    assert_eq!(progress.progress_percent(), 100.0);
}

#[test]
fn test_batch_processor() {
    let processor = BatchProcessor::with_defaults();
    let data = create_test_klines(50);

    let result = processor.process_in_batches(data, |chunk| {
        // Just verify we can process the chunk
        assert!(!chunk.is_empty());
        Ok::<(), crate::core::QuantixError>(())
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap().processed_records, 50);
}

#[test]
fn process_in_batches_returns_callback_error() {
    let processor = BatchProcessor::new(BatchConfig {
        batch_size: 10,
        enable_progress: false,
        ..Default::default()
    })
    .unwrap();
    let data = create_test_klines(25);

    let result = processor.process_in_batches(data, |_chunk| {
        Err::<(), crate::core::QuantixError>(crate::core::QuantixError::Other(
            "simulated batch failure".to_string(),
        ))
    });

    assert!(result.is_err());
}

#[tokio::test]
async fn stream_process_tracks_progress_for_streamed_chunks() {
    let processor = BatchProcessor::new(BatchConfig {
        batch_size: 2,
        enable_progress: false,
        ..Default::default()
    })
    .unwrap();
    let chunks = std::sync::Arc::new(vec![
        create_test_klines(2),
        create_test_klines(3),
        Vec::new(),
    ]);

    let result = processor
        .stream_process(
            |batch_num| {
                let chunks = chunks.clone();
                async move {
                    Ok::<Vec<crate::data::models::Kline>, crate::core::QuantixError>(
                        chunks.get(batch_num).cloned().unwrap_or_default(),
                    )
                }
            },
            2,
        )
        .await
        .unwrap();

    assert_eq!(result.processed_records, 5);
    assert_eq!(result.success_count, 5);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.current_batch, 2);
    assert_eq!(result.total_batches, 2);
    assert!(result.is_complete());
}

#[test]
fn test_estimated_remaining() {
    let mut progress = BatchProgress::new(100, 10);
    progress.update(25, 0);
    // Simulate that some time has passed since the batch started.
    // Without this, the test is flaky under load: in release builds the
    // elapsed time between `new` and `estimated_remaining_secs` can be
    // sub-nanosecond, round to 0.0, and break the `> 0.0` assertion.
    progress.start_time = std::time::Instant::now() - Duration::from_millis(100);

    let remaining = progress.estimated_remaining_secs();
    // 25% done with 100 ms elapsed → remaining ≈ 300 ms
    assert!(
        remaining > 0.0,
        "remaining should be positive, got {remaining}"
    );
}
