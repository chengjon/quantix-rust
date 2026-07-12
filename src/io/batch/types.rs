/// 批处理配置与进度类型。
///
/// 从 [`super::mod`] 的原生 `batch.rs` 中拆出，仅承载 `BatchConfig`
/// 与 `BatchProgress` 的数据定义与方法，供处理器与测试共享。
use std::time::Instant;

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
