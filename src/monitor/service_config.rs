use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorServiceConfig {
    pub quantix_bin_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct JsonMonitorServiceConfigStore {
    path: PathBuf,
}

impl JsonMonitorServiceConfigStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn with_default_path() -> Result<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
            QuantixError::Config("HOME is required for monitor service config".into())
        })?;
        Ok(Self::new(
            home.join(".quantix").join("monitor").join("service.json"),
        ))
    }

    pub fn load(&self) -> Result<MonitorServiceConfig> {
        if !self.path.exists() {
            return Err(QuantixError::Config(format!(
                "monitor service config 不存在: {}",
                self.path.display()
            )));
        }

        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save(&self, config: &MonitorServiceConfig) -> Result<()> {
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

    pub fn validate(config: &MonitorServiceConfig) -> Result<()> {
        let path = &config.quantix_bin_path;

        if !path.is_absolute() {
            return Err(QuantixError::Config(format!(
                "monitor service quantix_bin_path 必须是绝对路径: {}",
                path.display()
            )));
        }

        if !path.exists() {
            return Err(QuantixError::Config(format!(
                "monitor service quantix_bin_path 不存在: {}",
                path.display()
            )));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(path)?.permissions().mode();
            if mode & 0o111 == 0 {
                return Err(QuantixError::Config(format!(
                    "monitor service quantix_bin_path 不可执行: {}",
                    path.display()
                )));
            }
        }

        Ok(())
    }
}
