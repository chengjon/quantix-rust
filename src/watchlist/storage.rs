use crate::core::Result;
use crate::watchlist::models::WatchlistStore;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WatchlistStorage {
    path: PathBuf,
}

impl WatchlistStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Option<WatchlistStore>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&self.path)?;
        let store = serde_json::from_str(&raw)?;
        Ok(Some(store))
    }

    pub fn load_or_create(&self) -> Result<WatchlistStore> {
        if let Some(store) = self.load()? {
            return Ok(store);
        }

        let store = WatchlistStore::default();
        self.save(&store)?;
        Ok(store)
    }

    pub fn save(&self, store: &WatchlistStore) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = serde_json::to_string_pretty(store)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }
}
