use chrono::{TimeZone, Utc};
use quantix_cli::watchlist::{
    WatchlistAction, WatchlistEntry, WatchlistHistoryEvent, WatchlistStorage, WatchlistStore,
};
use std::collections::HashMap;
use tempfile::tempdir;

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 10, 12, 0, 0).unwrap()
}

#[test]
fn load_or_create_creates_default_store() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    let storage = WatchlistStorage::new(path.clone());

    let store = storage.load_or_create().unwrap();

    assert_eq!(store.version, 1);
    assert_eq!(store.default_group, "default");
    assert_eq!(store.groups.get("default"), Some(&Vec::<String>::new()));
    assert!(store.entries.is_empty());
    assert!(store.history.is_empty());
    assert!(path.exists());
}

#[test]
fn save_and_load_round_trip_preserves_groups() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    let storage = WatchlistStorage::new(path);
    let ts = fixed_ts();

    let mut groups = HashMap::new();
    groups.insert("default".to_string(), vec!["000001".to_string()]);
    groups.insert("core".to_string(), vec!["600519".to_string()]);

    let store = WatchlistStore {
        version: 1,
        default_group: "default".to_string(),
        groups,
        entries: HashMap::new(),
        history: Vec::new(),
        updated_at: ts,
    };

    storage.save(&store).unwrap();
    let loaded = storage.load().unwrap().unwrap();

    assert_eq!(loaded.groups, store.groups);
    assert_eq!(loaded.updated_at, ts);
}

#[test]
fn save_and_load_round_trip_preserves_tags_and_history() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    let storage = WatchlistStorage::new(path);
    let ts = fixed_ts();

    let mut groups = HashMap::new();
    groups.insert("default".to_string(), vec!["000001".to_string()]);

    let mut entries = HashMap::new();
    entries.insert(
        "000001".to_string(),
        WatchlistEntry {
            tags: vec!["bank".to_string(), "longterm".to_string()],
            added_at: ts,
            updated_at: ts,
        },
    );

    let history = vec![WatchlistHistoryEvent {
        ts,
        action: WatchlistAction::TagAdd,
        code: Some("000001".to_string()),
        group: Some("default".to_string()),
        tag: Some("bank".to_string()),
    }];

    let store = WatchlistStore {
        version: 1,
        default_group: "default".to_string(),
        groups,
        entries,
        history,
        updated_at: ts,
    };

    storage.save(&store).unwrap();
    let loaded = storage.load().unwrap().unwrap();

    assert_eq!(loaded.entries, store.entries);
    assert_eq!(loaded.history, store.history);
}

#[test]
fn load_or_create_creates_missing_parent_directory() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested").join("watchlist").join("store.json");
    let storage = WatchlistStorage::new(path.clone());

    let store = storage.load_or_create().unwrap();

    assert_eq!(store.default_group, "default");
    assert!(path.exists());
}
