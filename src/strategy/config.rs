#![allow(clippy::collapsible_if)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

/// 启动 bootstrap 策略：当前仅 LatestOnly（启动时只加载最新一根 K 线做评估，不回放历史）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapPolicy {
    LatestOnly,
}

/// 单个策略实例配置：id 实例唯一键、name 策略类型（如 "ma_cross"）、enabled 是否启用、params 透传给策略实现的参数（JSON）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredStrategyInstance {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub params: serde_json::Value,
}

/// 单只股票配置：code 标的代码、enabled 是否参与 daemon 调度、strategies 该股票上挂载的策略实例列表。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredStock {
    pub code: String,
    pub enabled: bool,
    pub strategies: Vec<ConfiguredStrategyInstance>,
}

/// 策略 daemon 配置：check_interval_secs 轮询间隔（秒）、bootstrap_policy 启动策略、stocks 待评估的股票列表。Default 提供一只 000001 + ma_cross(5,20) 的样例。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyDaemonConfig {
    pub check_interval_secs: u64,
    pub bootstrap_policy: BootstrapPolicy,
    pub stocks: Vec<ConfiguredStock>,
}

impl Default for StrategyDaemonConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
            bootstrap_policy: BootstrapPolicy::LatestOnly,
            stocks: vec![ConfiguredStock {
                code: "000001".to_string(),
                enabled: true,
                strategies: vec![ConfiguredStrategyInstance {
                    id: "ma_fast_5_slow_20".to_string(),
                    name: "ma_cross".to_string(),
                    enabled: true,
                    params: json!({
                        "fast": 5,
                        "slow": 20
                    }),
                }],
            }],
        }
    }
}

/// JSON 文件后端 strategy daemon 配置 store：持有 path，load/save 围绕该路径读写 StrategyDaemonConfig。
#[derive(Debug, Clone)]
pub struct JsonStrategyConfigStore {
    path: PathBuf,
}

impl JsonStrategyConfigStore {
    /// 用显式路径构造 JSON 配置存储，文件不要求存在（load/save 时创建）。
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 用 `$HOME/.quantix/strategy/config.json` 构造；HOME 未设置时返回 Config 错误。
    pub fn with_default_path() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| QuantixError::Config("HOME is required for strategy config".into()))?;
        Ok(Self::new(
            home.join(".quantix").join("strategy").join("config.json"),
        ))
    }

    /// 加载配置；文件不存在时写入默认配置并返回（首次启动引导）。
    pub fn load_or_create(&self) -> Result<StrategyDaemonConfig> {
        if !self.path.exists() {
            let config = StrategyDaemonConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        self.load()
    }

    /// 读取并反序列化配置文件；文件缺失或 JSON 非法时返回错误。
    pub fn load(&self) -> Result<StrategyDaemonConfig> {
        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    /// 原子保存：先写 .tmp 再 rename；父目录按需创建，避免半截文件污染配置。
    pub fn save(&self, config: &StrategyDaemonConfig) -> Result<()> {
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

    /// 返回底层配置文件路径（只读）。
    pub fn path(&self) -> &Path {
        &self.path
    }
}
