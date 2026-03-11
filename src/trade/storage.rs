use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::Result;
use crate::trade::{PaperTradeState, PaperTradeStore};

#[derive(Debug, Clone)]
pub struct JsonPaperTradeStore {
    path: PathBuf,
}

impl JsonPaperTradeStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl PaperTradeStore for JsonPaperTradeStore {
    async fn load_state(&self) -> Result<Option<PaperTradeState>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&self.path)?;
        let state = serde_json::from_str(&raw)?;
        Ok(Some(state))
    }

    async fn save_state(&self, state: &PaperTradeState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = serde_json::to_string_pretty(state)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }
}
