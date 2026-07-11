#![allow(clippy::collapsible_if)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

/// 策略 systemd service 配置：quantix_bin_path 可执行文件绝对路径、environment_file_path 可选 env 文件路径（写入 systemd unit 的 EnvironmentFile=）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyServiceConfig {
    pub quantix_bin_path: PathBuf,
    pub environment_file_path: Option<PathBuf>,
}

/// 策略 service 配置 JSON 文件 store：持有 path，load/save 围绕该路径读写 StrategyServiceConfig。
#[derive(Debug, Clone)]
pub struct JsonStrategyServiceConfigStore {
    path: PathBuf,
}

impl JsonStrategyServiceConfigStore {
    /// 构造指向指定 path 的 JSON 配置 store；不读不写，仅记录路径。
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 构造指向 $HOME/.quantix/strategy/service.json 的默认 store；HOME 环境变量缺失返回 Config 错误。
    pub fn with_default_path() -> Result<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
            QuantixError::Config("HOME is required for strategy service config".into())
        })?;
        Ok(Self::new(
            home.join(".quantix").join("strategy").join("service.json"),
        ))
    }

    /// 从 path 读取并反序列化为 StrategyServiceConfig；文件不存在返回 Config 错误，读取或 JSON 解析失败透传。
    pub fn load(&self) -> Result<StrategyServiceConfig> {
        if !self.path.exists() {
            return Err(QuantixError::Config(format!(
                "strategy service config 不存在: {}",
                self.path.display()
            )));
        }

        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    /// 原子写入：先序列化为 pretty JSON 写入 .tmp 临时文件，再 rename 到目标路径；自动创建父目录。序列化、目录创建、写入或 rename 失败透传。
    pub fn save(&self, config: &StrategyServiceConfig) -> Result<()> {
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

    /// 校验 quantix_bin_path 必须是绝对路径、文件存在、（unix）至少含一位可执行权限位（mode & 0o111 != 0）。任一不满足返回带路径的 Config 错误。environment_file_path 若设置，仅作为运行参数透传，此函数不校验其存在性。
    pub fn validate(config: &StrategyServiceConfig) -> Result<()> {
        let path = &config.quantix_bin_path;

        if !path.is_absolute() {
            return Err(QuantixError::Config(format!(
                "strategy service quantix_bin_path 必须是绝对路径: {}",
                path.display()
            )));
        }

        if !path.exists() {
            return Err(QuantixError::Config(format!(
                "strategy service quantix_bin_path 不存在: {}",
                path.display()
            )));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(path)?.permissions().mode();
            if mode & 0o111 == 0 {
                return Err(QuantixError::Config(format!(
                    "strategy service quantix_bin_path 不可执行: {}",
                    path.display()
                )));
            }
        }

        Ok(())
    }
}
