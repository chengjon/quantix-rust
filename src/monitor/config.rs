#![allow(clippy::collapsible_if)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub interval_seconds: u64,
    pub watchlist_group: Option<String>,
    pub persist_events: bool,
    pub max_event_history: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            watchlist_group: None,
            persist_events: true,
            max_event_history: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonMonitorConfigStore {
    path: PathBuf,
}

impl JsonMonitorConfigStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn load_or_create(&self) -> Result<MonitorConfig> {
        if !self.path.exists() {
            let config = MonitorConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save(&self, config: &MonitorConfig) -> Result<()> {
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
