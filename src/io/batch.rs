/// 批处理模块
///
/// 大数据量导入导出优化处理
use crate::core::Result;
use crate::data::models::Kline;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// 批处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批处理大小
    pub batch_size: usize,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 是否启用进度条
    pub enable_progress: bool,
    /// 内存限制（MB）
    pub memory_limit_mb: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            max_concurrent_tasks: 4,
            enable_progress: true,
            memory_limit_mb: 512,
        }
    }
}

/// 批处理进度
#[derive(Debug, Clone)]
pub struct BatchProgress {
    /// 总记录数
    pub total_records: usize,
    /// 已处理记录数
    pub processed_records: usize,
    /// 成功记录数
    pub success_count: usize,
    /// 失败记录数
    pub error_count: usize,
    /// 开始时间
    pub start_time: Instant,
    /// 当前批次数
    pub current_batch: usize,
    /// 总批次数
    pub total_batches: usize,
}

impl BatchProgress {
    /// 创建新的进度追踪器
    pub fn new(total_records: usize, batch_size: usize) -> Self {
        let total_batches = total_records.div_ceil(batch_size);
        Self {
            total_records,
            processed_records: 0,
            success_count: 0,
            error_count: 0,
            start_time: Instant::now(),
            current_batch: 0,
            total_batches,
        }
    }

    /// 更新进度
    pub fn update(&mut self, success: usize, errors: usize) {
        self.processed_records += success + errors;
        self.success_count += success;
        self.error_count += errors;
        self.current_batch =
            self.processed_records / self.total_records.div_ceil(self.total_batches);
    }

    /// 是否完成
    pub fn is_complete(&self) -> bool {
        self.processed_records >= self.total_records
    }

    /// 获取进度百分比
    pub fn progress_percent(&self) -> f64 {
        if self.total_records == 0 {
            return 100.0;
        }
        (self.processed_records as f64 / self.total_records as f64) * 100.0
    }

    /// 获取已用时间（秒）
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// 估算剩余时间（秒）
    pub fn estimated_remaining_secs(&self) -> f64 {
        let progress = self.progress_percent();
        if progress <= 0.0 {
            return 0.0;
        }
        let elapsed = self.elapsed_secs();
        elapsed * (100.0 - progress) / progress
    }

    /// 获取处理速度（记录/秒）
    pub fn records_per_second(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.processed_records as f64 / elapsed
        } else {
            0.0
        }
    }
}

/// 批处理器
pub struct BatchProcessor {
    config: BatchConfig,
}

impl BatchProcessor {
    /// 创建新的批处理器
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(BatchConfig::default())
    }

    /// 批量导出数据
    pub async fn batch_export<F>(
        &self,
        data: &[Kline],
        export_fn: F,
        output_prefix: &str,
    ) -> Result<BatchProgress>
    where
        F: Fn(&[Kline], &str) -> Result<()> + Send + Sync,
    {
        let total_records = data.len();
        let batch_size = self.config.batch_size;
        let mut progress = BatchProgress::new(total_records, batch_size);

        let progress_bar = if self.config.enable_progress {
            Some(Arc::new(ProgressBar::new(total_records as u64)))
        } else {
            None
        };

        if let Some(bar) = &progress_bar {
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap(),
            );
        }

        // 分批处理
        for chunk in data.chunks(batch_size) {
            let batch_num = progress.current_batch + 1;
            let output_file = format!("{}_batch_{}.csv", output_prefix, batch_num);

            if let Some(bar) = &progress_bar {
                bar.set_message(format!("处理批次 {}/{}", batch_num, progress.total_batches));
            }

            match export_fn(chunk, &output_file) {
                Ok(()) => {
                    progress.update(chunk.len(), 0);
                }
                Err(e) => {
                    progress.update(0, chunk.len());
                    if let Some(bar) = &progress_bar {
                        bar.println(format!("批次数 {} 失败: {}", batch_num, e));
                    }
                }
            }

            if let Some(bar) = &progress_bar {
                bar.inc(chunk.len() as u64);
            }
        }

        if let Some(bar) = &progress_bar {
            bar.finish_with_message("批处理完成");
        }

        Ok(progress)
    }

    /// 批量导入数据
    pub async fn batch_import<F, R>(
        &self,
        data_sources: Vec<R>,
        import_fn: F,
    ) -> Result<BatchProgress>
    where
        F: Fn(R) -> Result<Vec<Kline>> + Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        let total_sources = data_sources.len();
        let mut progress = BatchProgress::new(total_sources, 1);

        let progress_bar = if self.config.enable_progress {
            Some(Arc::new(ProgressBar::new(total_sources as u64)))
        } else {
            None
        };

        if let Some(bar) = &progress_bar {
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap(),
            );
        }

        // 使用信号量限制并发
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_tasks));
        let import_fn = Arc::new(import_fn);
        let mut handles = Vec::new();

        for (i, source) in data_sources.into_iter().enumerate() {
            let semaphore = semaphore.clone();
            let import_fn = import_fn.clone();

            if let Some(bar) = &progress_bar {
                bar.set_message(format!("导入源 {}/{}", i + 1, total_sources));
            }

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = import_fn(source);
                let (success, errors) = match result {
                    Ok(klines) => (klines.len(), 0),
                    Err(_) => (0, 1),
                };
                (success, errors)
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            match handle.await {
                Ok((success, errors)) => {
                    progress.update(success, errors);
                }
                Err(e) => {
                    progress.update(0, 1);
                    if let Some(bar) = &progress_bar {
                        bar.println(format!("任务失败: {}", e));
                    }
                }
            }
        }

        if let Some(bar) = &progress_bar {
            bar.finish_with_message("批导入完成");
        }

        Ok(progress)
    }

    /// 分批处理数据（内存优化版本）
    pub fn process_in_batches<T, F, R>(&self, data: Vec<T>, process_fn: F) -> Result<BatchProgress>
    where
        T: Send + Sync + 'static,
        F: Fn(&[T]) -> R + Send + Sync,
        R: Send + Sync,
    {
        let batch_size = self.config.batch_size;
        let mut progress = BatchProgress::new(data.len(), batch_size);

        for chunk in data.chunks(batch_size) {
            let _ = process_fn(chunk);
            progress.update(chunk.len(), 0);
        }

        Ok(progress)
    }

    /// 流式处理（适用于超大文件）
    pub async fn stream_process<F, Fut>(
        &self,
        stream_fn: F,
        batch_size: usize,
    ) -> Result<BatchProgress>
    where
        F: Fn(usize) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<Kline>>>,
    {
        let mut progress = BatchProgress::new(0, batch_size);
        let mut batch_num = 0;

        loop {
            // TODO: 添加流式进度显示
            let _ = &self.config.enable_progress;

            let chunk = stream_fn(batch_num).await?;

            if chunk.is_empty() {
                break;
            }

            batch_num += 1;
            progress.total_records += chunk.len();
            progress.update(chunk.len(), 0);

            // 内存控制：定期释放
            if batch_num % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }

        Ok(progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::AdjustType;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn create_test_klines(count: usize) -> Vec<Kline> {
        (0..count)
            .map(|i| Kline {
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
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap().processed_records, 50);
    }

    #[test]
    fn test_estimated_remaining() {
        let mut progress = BatchProgress::new(100, 10);
        progress.update(25, 0);

        let remaining = progress.estimated_remaining_secs();
        // 25% done, remaining should be roughly 3x elapsed
        assert!(remaining > 0.0);
    }
}
