use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const WATCHLIST_STORE_VERSION: u32 = 1;
pub const DEFAULT_WATCHLIST_GROUP: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WatchlistAction {
    Add,
    Remove,
    Move,
    TagAdd,
    TagRemove,
    GroupCreate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchlistEntry {
    pub tags: Vec<String>,
    pub added_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchlistHistoryEvent {
    pub ts: DateTime<Utc>,
    pub action: WatchlistAction,
    pub code: Option<String>,
    pub group: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchlistStore {
    pub version: u32,
    pub default_group: String,
    pub groups: HashMap<String, Vec<String>>,
    pub entries: HashMap<String, WatchlistEntry>,
    pub history: Vec<WatchlistHistoryEvent>,
    pub updated_at: DateTime<Utc>,
}

impl Default for WatchlistStore {
    fn default() -> Self {
        let mut groups = HashMap::new();
        groups.insert(DEFAULT_WATCHLIST_GROUP.to_string(), Vec::new());

        Self {
            version: WATCHLIST_STORE_VERSION,
            default_group: DEFAULT_WATCHLIST_GROUP.to_string(),
            groups,
            entries: HashMap::new(),
            history: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchlistListItem {
    pub code: String,
    pub group: String,
    pub tags: Vec<String>,
}
