use async_trait::async_trait;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::core::Result;
use crate::risk::{RiskState, RiskStore};

/// JSON 文件后端 RiskStore 实现：持有 path，load/save 围绕该路径读写 RiskState（单账户 JSON）。
#[derive(Debug, Clone)]
pub struct JsonRiskStore {
    path: PathBuf,
}

impl JsonRiskStore {
    /// 创建 store：注入 JSON 路径，load/save 时才读写文件。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 返回底层 JSON 文件路径（用于诊断/日志）。
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl RiskStore for JsonRiskStore {
    async fn load_state(&self) -> Result<Option<RiskState>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&self.path)?;
        let state = serde_json::from_str(&raw)?;
        Ok(Some(state))
    }

    async fn save_state(&self, state: &RiskState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = serde_json::to_vec_pretty(state)?;
        let temp_path = temporary_path_for(&self.path);

        let write_result = (|| -> Result<()> {
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(&raw)?;
            file.sync_all()?;
            replace_file(&temp_path, &self.path)?;
            Ok(())
        })();

        if write_result.is_err() && temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        write_result?;
        Ok(())
    }
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("risk_state.json");

    path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()))
}

fn replace_file(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        if to.exists() {
            fs::remove_file(to)?;
        }
    }

    fs::rename(from, to)
}
