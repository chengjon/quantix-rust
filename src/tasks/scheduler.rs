#![allow(clippy::collapsible_if)]

/// 任务调度器
///
/// 从短线侠项目迁移 - 基于 tokio-cron-scheduler 的异步任务调度
use crate::tasks::cron::CronExpression;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{info, warn};
use uuid::Uuid;

/// 定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// 任务名称
    pub name: String,
    /// Cron 表达式
    pub cron_expr: String,
    /// 任务命令/描述
    pub command: String,
    /// 是否启用
    pub enabled: bool,
    /// 任务ID（用于调度器）
    #[serde(skip)]
    pub job_id: Option<Uuid>,
}

impl ScheduledTask {
    /// 创建新任务
    pub fn new(name: String, cron_expr: String, command: String) -> Self {
        Self {
            name,
            cron_expr,
            command,
            enabled: true,
            job_id: None,
        }
    }
}

/// 任务执行回调
pub type TaskCallback = Arc<dyn Fn() + Send + Sync + 'static>;

/// 任务调度器
#[derive(Clone)]
pub struct TaskScheduler {
    /// tokio-cron-scheduler 实例
    scheduler: Arc<RwLock<Option<JobScheduler>>>,
    /// 任务列表
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    /// 回调映射
    callbacks: Arc<RwLock<HashMap<String, TaskCallback>>>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl TaskScheduler {
    /// 创建新的任务调度器
    pub async fn new() -> Result<Self, JobSchedulerError> {
        let scheduler = JobScheduler::new().await?;
        info!("任务调度器初始化完成");

        Ok(Self {
            scheduler: Arc::new(RwLock::new(Some(scheduler))),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            callbacks: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 添加任务
    pub async fn add_task(&self, task: ScheduledTask) -> Result<(), String> {
        // 验证 cron 表达式
        let _cron = CronExpression::new(&task.cron_expr)
            .map_err(|e| format!("无效的 cron 表达式: {}", e))?;

        info!("添加任务: {} ({})", task.name, task.cron_expr);

        let mut tasks = self.tasks.write().await;
        let task_name = task.name.clone();

        // 如果调度器正在运行，需要先删除旧任务
        if let Some(scheduler) = &*self.scheduler.read().await {
            if let Some(old_task) = tasks.get(&task_name) {
                if let Some(job_id) = &old_task.job_id {
                    let _ = scheduler.remove(job_id).await;
                }
            }
        }

        tasks.insert(task_name.clone(), task);

        // 如果调度器正在运行，立即添加到调度器
        if *self.running.read().await {
            // 重新获取 cron 用于调度
            let cron = CronExpression::new(&tasks[&task_name].cron_expr).unwrap();
            self.schedule_task(&task_name, &cron).await?;
        }

        Ok(())
    }

    /// 删除任务
    pub async fn remove_task(&self, name: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.remove(name) {
            // 从调度器中删除
            if let Some(scheduler) = &*self.scheduler.read().await {
                if let Some(job_id) = &task.job_id {
                    scheduler
                        .remove(job_id)
                        .await
                        .map_err(|e| format!("删除任务失败: {}", e))?;
                }
            }

            info!("删除任务: {}", name);
            Ok(())
        } else {
            Err(format!("任务不存在: {}", name))
        }
    }

    /// 获取所有任务
    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.write().await;

        if *running {
            return Ok(());
        }

        *running = true;

        // 启动底层的 JobScheduler
        if let Some(scheduler) = &*self.scheduler.read().await {
            scheduler
                .start()
                .await
                .map_err(|e| format!("启动调度器失败: {}", e))?;
        }

        // 添加所有启用的任务
        let tasks = self.tasks.read().await;
        for (name, task) in tasks.iter() {
            if task.enabled {
                if let Ok(cron) = CronExpression::new(&task.cron_expr) {
                    if let Err(e) = self.schedule_task(name, &cron).await {
                        warn!("添加任务 {} 到调度器失败: {}", name, e);
                    }
                }
            }
        }

        info!("任务调度器已启动");
        Ok(())
    }

    /// 将任务添加到调度器
    async fn schedule_task(&self, name: &str, _cron: &CronExpression) -> Result<(), String> {
        let tasks = self.tasks.read().await;
        let task = tasks
            .get(name)
            .ok_or_else(|| format!("任务不存在: {}", name))?;

        if let Some(scheduler) = &*self.scheduler.read().await {
            // 创建 Job - 使用 tokio-cron-scheduler 的 cron 语法
            let task_name_owned = name.to_string();
            let scheduler_handle = self.clone();
            let job = Job::new_async(task.cron_expr.as_str(), {
                move |_uuid, _l| {
                    let task_name = task_name_owned.clone();
                    let scheduler_handle = scheduler_handle.clone();
                    Box::pin(async move {
                        if let Err(err) = scheduler_handle
                            .execute_registered_callback(&task_name)
                            .await
                        {
                            warn!("执行任务 {} 失败: {}", task_name, err);
                        }
                    })
                }
            })
            .map_err(|e| format!("创建 Job 失败: {}", e))?;

            // 添加到调度器
            let job_id = scheduler
                .add(job)
                .await
                .map_err(|e| format!("添加到调度器失败: {}", e))?;

            // 更新任务的 job_id
            drop(tasks);
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(name) {
                task.job_id = Some(job_id);
            }

            info!("任务 {} 已添加到调度器 (ID: {})", name, job_id);
        }

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> Result<(), String> {
        let mut running = self.running.write().await;

        if !*running {
            return Ok(());
        }

        *running = false;

        // 获取 scheduler 并停止
        let mut scheduler_lock = self.scheduler.write().await;
        if let Some(mut scheduler) = scheduler_lock.take() {
            scheduler
                .shutdown()
                .await
                .map_err(|e| format!("停止调度器失败: {}", e))?;
        }

        info!("任务调度器已停止");
        Ok(())
    }

    /// 检查调度器是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 设置任务回调
    pub async fn set_callback<F>(&self, name: String, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.write().await;
        callbacks.insert(name, Arc::new(callback));
    }

    /// 移除任务回调
    pub async fn remove_callback(&self, name: &str) {
        let mut callbacks = self.callbacks.write().await;
        callbacks.remove(name);
    }

    /// 执行已注册的任务回调
    pub async fn execute_registered_callback(&self, name: &str) -> Result<(), String> {
        let callback = {
            let callbacks = self.callbacks.read().await;
            callbacks.get(name).cloned()
        };

        match callback {
            Some(callback) => {
                info!("执行任务: {}", name);
                (callback)();
            }
            None => warn!("任务 {} 未注册执行回调，仅记录调度事件", name),
        }

        Ok(())
    }

    /// 获取下次执行时间
    pub async fn next_run_time(&self, name: &str) -> Option<NaiveDateTime> {
        let tasks = self.tasks.read().await;
        let task = tasks.get(name)?;

        if let Ok(_cron) = CronExpression::new(&task.cron_expr) {
            // 使用我们自己的 CronExpression 来计算下次执行时间
            let cron = CronExpression::new(&task.cron_expr).ok()?;
            Some(cron.next_run_after(Utc::now().naive_utc()))
        } else {
            None
        }
    }

    /// 获取任务统计
    pub async fn stats(&self) -> SchedulerStats {
        let tasks = self.tasks.read().await;
        let total = tasks.len();
        let enabled = tasks.values().filter(|t| t.enabled).count();
        let running = *self.running.read().await;

        SchedulerStats {
            total_tasks: total,
            enabled_tasks: enabled,
            running,
        }
    }
}

/// 调度器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// 总任务数
    pub total_tasks: usize,
    /// 启用的任务数
    pub enabled_tasks: usize,
    /// 是否正在运行
    pub running: bool,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        // Note: This creates a scheduler without initializing the JobScheduler
        // Users should call `new()` instead for proper initialization
        Self {
            scheduler: Arc::new(RwLock::new(None)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            callbacks: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
}

/// 预定义的任务模板
pub struct TaskTemplates;

impl TaskTemplates {
    /// 盘前数据检查（每天 8:50）
    pub fn pre_market_check() -> ScheduledTask {
        ScheduledTask::new(
            "pre_market_check".to_string(),
            "0 8 * * 1-5".to_string(),
            "检查盘前数据".to_string(),
        )
    }

    /// 竞价数据采集（每天 9:15-9:25，每30秒）
    pub fn auction_collection() -> ScheduledTask {
        ScheduledTask::new(
            "auction_collection".to_string(),
            "30,0 9 * * 1-5".to_string(),
            "竞价数据采集".to_string(),
        )
    }

    /// 开盘检查（每天 9:30）
    pub fn market_open() -> ScheduledTask {
        ScheduledTask::new(
            "market_open".to_string(),
            "30 9 * * 1-5".to_string(),
            "开盘检查".to_string(),
        )
    }

    /// 收盘检查（每天 15:00）
    pub fn market_close() -> ScheduledTask {
        ScheduledTask::new(
            "market_close".to_string(),
            "0 15 * * 1-5".to_string(),
            "收盘检查".to_string(),
        )
    }

    /// 盘后数据处理（每天 15:30）
    pub fn post_market_process() -> ScheduledTask {
        ScheduledTask::new(
            "post_market_process".to_string(),
            "30 15 * * 1-5".to_string(),
            "盘后数据处理".to_string(),
        )
    }

    /// 数据同步（每天 16:00）
    pub fn data_sync() -> ScheduledTask {
        ScheduledTask::new(
            "data_sync".to_string(),
            "0 16 * * *".to_string(),
            "数据同步".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn test_task_scheduler_creation() {
        let scheduler = TaskScheduler::new().await;
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_task_scheduler_add() {
        let scheduler = TaskScheduler::new().await.unwrap();
        let task = ScheduledTask::new(
            "test_task".to_string(),
            "0 * * * *".to_string(),
            "测试任务".to_string(),
        );

        let result = scheduler.add_task(task).await;
        assert!(result.is_ok());

        let tasks = scheduler.list_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test_task");
    }

    #[test]
    fn test_task_templates() {
        let task = TaskTemplates::market_open();
        assert_eq!(task.name, "market_open");
        assert_eq!(task.cron_expr, "30 9 * * 1-5");
    }

    #[tokio::test]
    async fn test_execute_registered_callback_runs_callback() {
        let scheduler = TaskScheduler::new().await.unwrap();
        let hit = Arc::new(AtomicBool::new(false));
        let hit_clone = hit.clone();

        scheduler
            .set_callback("demo".to_string(), move || {
                hit_clone.store(true, Ordering::SeqCst);
            })
            .await;

        scheduler.execute_registered_callback("demo").await.unwrap();
        assert!(hit.load(Ordering::SeqCst));
    }
}
