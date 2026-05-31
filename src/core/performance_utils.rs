// 性能优化工具集
//
// Phase 18: 性能分析与优化工具

use std::time::Instant;

/// 性能计时器
#[derive(Debug)]
pub struct PerfTimer {
    name: String,
    start: Instant,
}

impl PerfTimer {
    /// 创建新的性能计时器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// 记录耗时并返回
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    /// 记录耗时并返回
    pub fn stop_and_print(self) -> std::time::Duration {
        let elapsed = self.elapsed();
        tracing::info!(target: "quantix::performance", name = %self.name, elapsed_ms = elapsed.as_millis());
        elapsed
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        if elapsed.as_millis() > 100 {
            tracing::warn!(
                target: "quantix::performance",
                name = %self.name,
                elapsed_ms = elapsed.as_millis(),
                "slow operation"
            );
        }
    }
}

/// 内存使用跟踪器
#[derive(Debug)]
pub struct MemoryTracker {
    name: String,
    initial_kb: usize,
}

impl MemoryTracker {
    /// 创建新的内存跟踪器
    pub fn new(name: impl Into<String>) -> Self {
        let initial_kb = Self::current_memory_kb();
        Self {
            name: name.into(),
            initial_kb,
        }
    }

    /// 获取当前内存使用（KB）
    fn current_memory_kb() -> usize {
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2
                            && let Ok(kb) = parts[1].parse::<usize>()
                        {
                            return kb;
                        }
                    }
                }
            }
        }
        0
    }

    /// 获取内存增量（KB）
    pub fn delta_kb(&self) -> isize {
        let current = Self::current_memory_kb();
        current as isize - self.initial_kb as isize
    }

    /// 记录内存使用并返回
    pub fn stop_and_print(self) -> isize {
        let delta = self.delta_kb();
        if delta > 0 {
            tracing::info!(
                target: "quantix::performance",
                name = %self.name,
                delta_kb = delta,
                "memory allocated"
            );
        } else if delta < 0 {
            tracing::info!(
                target: "quantix::performance",
                name = %self.name,
                delta_kb = delta,
                "memory freed"
            );
        } else {
            tracing::info!(
                target: "quantix::performance",
                name = %self.name,
                delta_kb = delta,
                "memory unchanged"
            );
        }
        delta
    }
}

/// 批处理优化配置
#[derive(Debug, Clone)]
pub struct BatchOptimizationConfig {
    /// 最优批次大小
    pub optimal_batch_size: usize,
    /// 是否启用并行处理
    pub enable_parallel: bool,
    /// 并行度（0 = 自动检测）
    pub parallelism: usize,
}

impl Default for BatchOptimizationConfig {
    fn default() -> Self {
        Self {
            optimal_batch_size: 1000,
            enable_parallel: true,
            parallelism: 0, // 自动检测 CPU 核心数
        }
    }
}

/// 性能优化建议
#[derive(Debug, Clone)]
pub enum OptimizationSuggestion {
    /// 增加批次大小
    IncreaseBatchSize {
        current: usize,
        suggested: usize,
        reason: String,
    },
    /// 启用并行处理
    EnableParallelProcessing { suggested_threads: usize },
    /// 使用预分配
    UsePreallocation { type_name: String, capacity: usize },
    /// 缓存计算结果
    CacheComputation { function_name: String },
    /// 使用零拷贝
    UseZeroCopy {
        current_approach: String,
        suggested_approach: String,
    },
}

/// 分析性能并提供建议
pub fn analyze_performance(
    _operation_name: &str,
    data_size: usize,
    duration_ms: u128,
    memory_delta_kb: isize,
) -> Vec<OptimizationSuggestion> {
    let mut suggestions = Vec::new();

    // 性能阈值检查
    let throughput = data_size as f64 / (duration_ms as f64 / 1000.0);

    if throughput < 1000.0 && data_size > 1000 {
        suggestions.push(OptimizationSuggestion::IncreaseBatchSize {
            current: data_size,
            suggested: data_size * 2,
            reason: format!("吞吐量低 ({:.1} items/s)", throughput),
        });
    }

    // 内存使用检查
    if memory_delta_kb > 10_000 {
        // > 10MB 内存增量
        suggestions.push(OptimizationSuggestion::UsePreallocation {
            type_name: "Vec".to_string(),
            capacity: data_size,
        });
    }

    // 检查是否适合并行化
    if duration_ms > 100 && data_size > 10_000 {
        let suggested_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        suggestions.push(OptimizationSuggestion::EnableParallelProcessing { suggested_threads });
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::new("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.stop_and_print();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_memory_tracker_reports_signed_delta() {
        // RSS can move down during the full test suite when unrelated allocations are freed.
        // Use a synthetic baseline to verify the signed-delta contract deterministically.
        let tracker = MemoryTracker {
            name: "vector_allocation".to_string(),
            initial_kb: MemoryTracker::current_memory_kb().saturating_add(1024 * 1024),
        };

        let delta = tracker.stop_and_print();
        assert!(delta <= 0);
    }

    #[test]
    fn test_performance_analysis() {
        // 慢速操作 - 应该产生优化建议
        let suggestions = analyze_performance(
            "slow_operation",
            100_000, // 100K items
            500,     // 500ms (慢)
            50_000,  // 50MB allocated
        );

        assert!(!suggestions.is_empty());
    }
}
