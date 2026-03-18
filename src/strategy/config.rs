use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapPolicy {
    LatestOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredStrategyInstance {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredStock {
    pub code: String,
    pub enabled: bool,
    pub strategies: Vec<ConfiguredStrategyInstance>,
}

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

#[derive(Debug, Clone)]
pub struct JsonStrategyConfigStore {
    path: PathBuf,
}

impl JsonStrategyConfigStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn with_default_path() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| QuantixError::Config("HOME is required for strategy config".into()))?;
        Ok(Self::new(
            home.join(".quantix").join("strategy").join("config.json"),
        ))
    }

    pub fn load_or_create(&self) -> Result<StrategyDaemonConfig> {
        if !self.path.exists() {
            let config = StrategyDaemonConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        self.load()
    }

    pub fn load(&self) -> Result<StrategyDaemonConfig> {
        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

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

    pub fn path(&self) -> &Path {
        &self.path
    }
}
