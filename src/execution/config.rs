#![allow(clippy::collapsible_if)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

/// 自动审批模式：Manual 人工批准（默认）、Always daemon 自动批准所有 pending request。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoApprovalMode {
    Manual,
    Always,
}

/// 自动审批配置：mode 决定 daemon 对 pending request 的批准策略。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoApprovalConfig {
    pub mode: AutoApprovalMode,
}

/// 执行 daemon 配置：poll_interval_secs 轮询间隔、max_requests_per_iteration 单轮处理上限、auto_approval 自动审批策略。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionDaemonConfig {
    pub poll_interval_secs: u64,
    pub max_requests_per_iteration: usize,
    pub auto_approval: AutoApprovalConfig,
}

impl Default for ExecutionDaemonConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 10,
            max_requests_per_iteration: 1,
            auto_approval: AutoApprovalConfig {
                mode: AutoApprovalMode::Manual,
            },
        }
    }
}

/// JSON 文件执行的配置存储：load/load_or_create/save，原子保存（.tmp + rename）。默认路径 `$HOME/.quantix/execution/config.json`。
#[derive(Debug, Clone)]
pub struct JsonExecutionConfigStore {
    path: PathBuf,
}

impl JsonExecutionConfigStore {
    /// 用显式路径构造 JSON 配置存储；文件不要求存在（load/save 时按需创建）。
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 用 `$HOME/.quantix/execution/config.json` 构造；HOME 未设置时返回 Config 错误。
    pub fn with_default_path() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| QuantixError::Config("HOME is required for execution config".into()))?;
        Ok(Self::new(
            home.join(".quantix").join("execution").join("config.json"),
        ))
    }

    /// 返回底层配置文件路径（只读）。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 读取并反序列化配置；文件不存在或 JSON 非法时返回错误（与 strategy config 不同，此处不自动创建）。
    pub fn load(&self) -> Result<ExecutionDaemonConfig> {
        if !self.path.exists() {
            return Err(QuantixError::Config(format!(
                "execution config 不存在: {}",
                self.path.display()
            )));
        }

        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    /// 文件存在则加载，否则写入默认配置并返回（首次启动引导）。
    pub fn load_or_create(&self) -> Result<ExecutionDaemonConfig> {
        if self.path.exists() {
            return self.load();
        }

        let config = ExecutionDaemonConfig::default();
        self.save(&config)?;
        Ok(config)
    }

    /// 原子保存：先写 .tmp 再 rename；父目录按需创建，避免半截文件污染配置。
    pub fn save(&self, config: &ExecutionDaemonConfig) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, serde_json::to_string_pretty(config)?)?;
        std::fs::rename(tmp_path, &self.path)?;
        Ok(())
    }
}
