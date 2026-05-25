#![allow(clippy::collapsible_if)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSwitchState {
    pub enabled: bool,
    pub reason: Option<String>,
    pub enabled_at: Option<DateTime<Utc>>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub updated_by: String,
}

impl Default for KillSwitchState {
    fn default() -> Self {
        Self {
            enabled: false,
            reason: None,
            enabled_at: None,
            disabled_at: None,
            updated_by: "cli".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonKillSwitchStore {
    path: PathBuf,
}

impl JsonKillSwitchStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn with_default_path() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| QuantixError::Config("HOME is required for kill switch state".into()))?;
        Ok(Self::new(
            home.join(".quantix")
                .join("safety")
                .join("kill_switch.json"),
        ))
    }

    pub fn load(&self) -> Result<KillSwitchState> {
        if !self.path.exists() {
            return Err(QuantixError::Config(format!(
                "kill switch state 不存在: {}",
                self.path.display()
            )));
        }

        let contents = std::fs::read_to_string(&self.path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn load_or_default(&self) -> Result<KillSwitchState> {
        if self.path.exists() {
            return self.load();
        }

        Ok(KillSwitchState::default())
    }

    pub fn save(&self, state: &KillSwitchState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, serde_json::to_string_pretty(state)?)?;
        std::fs::rename(tmp_path, &self.path)?;
        Ok(())
    }
}

pub fn kill_switch_blocks_target_mode(target_mode: &str) -> bool {
    matches!(target_mode, "mock_live" | "qmt_live")
}

pub fn load_blocking_kill_switch_state(
    store: &JsonKillSwitchStore,
    target_mode: &str,
) -> Result<Option<KillSwitchState>> {
    if !kill_switch_blocks_target_mode(target_mode) {
        return Ok(None);
    }

    let state = store.load_or_default()?;
    if state.enabled {
        return Ok(Some(state));
    }

    Ok(None)
}

pub fn format_execution_kill_switch_block_message(
    target_mode: &str,
    state: &KillSwitchState,
) -> String {
    let reason = state.reason.as_deref().unwrap_or("unspecified");
    format!("execution 被 kill switch 阻止: target_mode={target_mode}, reason={reason}")
}

pub fn build_kill_switch_payload(
    state: &KillSwitchState,
    target_mode: &str,
    blocked_at: DateTime<Utc>,
) -> Value {
    json!({
        "enabled": state.enabled,
        "reason": state.reason.clone(),
        "enabled_at": state.enabled_at.as_ref().map(|value| value.to_rfc3339()),
        "disabled_at": state.disabled_at.as_ref().map(|value| value.to_rfc3339()),
        "blocked_at": blocked_at.to_rfc3339(),
        "target_mode": target_mode,
        "updated_by": state.updated_by.clone(),
    })
}
