#![allow(clippy::collapsible_if)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};

/// Kill switch 持久化状态，启用后阻断实盘 / 模拟实盘执行。
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

/// 基于 JSON 文件的 kill switch 状态存储，原子写入（先写 `.tmp` 再 rename）。
#[derive(Debug, Clone)]
pub struct JsonKillSwitchStore {
    path: PathBuf,
}

impl JsonKillSwitchStore {
    /// 用指定路径构造存储。
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 使用默认路径 `~/.quantix/safety/kill_switch.json`。
    ///
    /// 返回 [`QuantixError::Config`] 当 `HOME` 环境变量未设置。
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

    /// 从磁盘读取并反序列化状态。
    ///
    /// 文件不存在时返回 [`QuantixError::Config`]；如需回退默认值请用 [`Self::load_or_default`]。
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

    /// 读取状态，文件缺失时返回 [`KillSwitchState::default()`]（未启用）。
    pub fn load_or_default(&self) -> Result<KillSwitchState> {
        if self.path.exists() {
            return self.load();
        }

        Ok(KillSwitchState::default())
    }

    /// 原子写入状态：先写 `.tmp` 再 rename，避免半写入导致 JSON 损坏。
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

/// 判断指定 `target_mode` 是否受 kill switch 约束（`mock_live` 与 `qmt_live`）。
pub fn kill_switch_blocks_target_mode(target_mode: &str) -> bool {
    matches!(target_mode, "mock_live" | "qmt_live")
}

/// 若 `target_mode` 受 kill switch 约束且当前已启用，返回阻塞状态；否则返回 `None`。
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

/// 生成面向用户的阻塞说明文本（含原因，缺省显示 `unspecified`）。
pub fn format_execution_kill_switch_block_message(
    target_mode: &str,
    state: &KillSwitchState,
) -> String {
    let reason = state.reason.as_deref().unwrap_or("unspecified");
    format!("execution 被 kill switch 阻止: target_mode={target_mode}, reason={reason}")
}

/// 构造用于诊断 / 日志的结构化 JSON payload（包含状态字段 + `blocked_at` + `target_mode`）。
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
