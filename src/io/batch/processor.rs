/// 批处理器实现。
///
/// 从 [`super::mod`] 的原生 `batch.rs` 中拆出，承载 [`BatchProcessor`]
/// 结构体及其全部方法（批量导出/导入、分批处理、流式处理）。
use super::types::{BatchConfig, BatchProgress};

use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 批处理器
pub struct BatchProcessor {
    config: BatchConfig,
}

impl BatchProcessor {
    /// 创建新的批处理器
    pub fn new(config: BatchConfig) -> Result<Self> {
        if config.batch_size == 0 {
            return Err(QuantixError::Other(
                "batch_size must be greater than zero".to_string(),
            ));
        }
        if config.max_concurrent_tasks == 0 {
            return Err(QuantixError::Other(
                "max_concurrent_tasks must be greater than zero".to_string(),
            ));
        }

        Ok(Self { config })
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        let config = BatchConfig::default();
        if config.batch_size == 0 || config.max_concurrent_tasks == 0 {
            return Self {
                config: BatchConfig {
                    batch_size: 1,
                    max_concurrent_tasks: 1,
                    ..config
                },
            };
        }

        Self { config }
    }

    fn default_progress_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
    }

    fn stream_progress_style() -> ProgressStyle {
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {pos} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
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
            bar.set_style(Self::default_progress_style());
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
            bar.set_style(Self::default_progress_style());
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
                let Ok(_permit) = semaphore.acquire().await else {
                    return (0, 1);
                };
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
    pub fn process_in_batches<T, F>(&self, data: Vec<T>, process_fn: F) -> Result<BatchProgress>
    where
        T: Send + Sync + 'static,
        F: Fn(&[T]) -> Result<()> + Send + Sync,
    {
        let batch_size = self.config.batch_size;
        let mut progress = BatchProgress::new(data.len(), batch_size);

        for chunk in data.chunks(batch_size) {
            process_fn(chunk)?;
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
        let progress_bar = if self.config.enable_progress {
            let bar = ProgressBar::new_spinner();
            bar.set_style(Self::stream_progress_style());
            bar.set_message("等待流式批次");
            Some(bar)
        } else {
            None
        };

        loop {
            if let Some(bar) = &progress_bar {
                bar.set_message(format!("读取流式批次 {}", batch_num + 1));
            }

            let chunk = stream_fn(batch_num).await?;

            if chunk.is_empty() {
                break;
            }

            batch_num += 1;
            progress.total_records += chunk.len();
            progress.total_batches = batch_num;
            progress.update(chunk.len(), 0);
            progress.current_batch = batch_num;

            if let Some(bar) = &progress_bar {
                bar.inc(chunk.len() as u64);
                bar.set_message(format!(
                    "已处理 {} 批 / {} 条",
                    progress.current_batch, progress.processed_records
                ));
            }

            // 内存控制：定期释放
            if batch_num % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }

        if let Some(bar) = &progress_bar {
            bar.finish_with_message("流式处理完成");
        }

        Ok(progress)
    }
}
