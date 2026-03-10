use chrono::{TimeZone, Utc};
use quantix_cli::watchlist::{
    WatchlistAction, WatchlistHistoryEvent, WatchlistService, WatchlistStore,
};

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 10, 13, 0, 0).unwrap()
}

fn service() -> WatchlistService {
    WatchlistService::new(10)
}

#[test]
fn add_puts_code_into_default_group() {
    let mut store = WatchlistStore::default();
    let now = fixed_ts();

    service().add(&mut store, "000001", None, now).unwrap();

    assert_eq!(
        store.groups.get("default").unwrap(),
        &vec!["000001".to_string()]
    );
    assert!(store.entries.contains_key("000001"));
}

#[test]
fn add_rejects_duplicate_code_in_same_group() {
    let mut store = WatchlistStore::default();
    let now = fixed_ts();
    let service = service();

    service.add(&mut store, "000001", None, now).unwrap();
    let err = service.add(&mut store, "000001", None, now).unwrap_err();

    assert!(err.to_string().contains("已存在"));
}

#[test]
fn move_between_groups_records_history() {
    let mut store = WatchlistStore::default();
    let now = fixed_ts();
    let service = service();

    service.create_group(&mut store, "core", now).unwrap();
    service.add(&mut store, "000001", None, now).unwrap();
    service.move_code(&mut store, "000001", "core", now).unwrap();

    assert_eq!(store.groups.get("default").unwrap(), &Vec::<String>::new());
    assert_eq!(store.groups.get("core").unwrap(), &vec!["000001".to_string()]);

    let latest = service.history(&store, Some("000001"), None);
    assert_eq!(latest[0].action, WatchlistAction::Move);
}

#[test]
fn add_and_remove_tag_updates_entry() {
    let mut store = WatchlistStore::default();
    let now = fixed_ts();
    let service = service();

    service.add(&mut store, "000001", None, now).unwrap();
    service.add_tag(&mut store, "000001", "bank", now).unwrap();
    service.remove_tag(&mut store, "000001", "bank", now).unwrap();

    assert_eq!(store.entries.get("000001").unwrap().tags, Vec::<String>::new());
}

#[test]
fn list_filters_by_tag() {
    let mut store = WatchlistStore::default();
    let now = fixed_ts();
    let service = service();

    service.add(&mut store, "000001", None, now).unwrap();
    service.add(&mut store, "600519", None, now).unwrap();
    service.add_tag(&mut store, "000001", "bank", now).unwrap();

    let filtered = service.list(&store, None, Some("bank"));

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].code, "000001");
}

#[test]
fn history_contains_mutation_events_newest_first() {
    let mut store = WatchlistStore::default();
    let now = fixed_ts();
    let service = service();

    service.create_group(&mut store, "core", now).unwrap();
    service.add(&mut store, "000001", None, now).unwrap();
    service.move_code(&mut store, "000001", "core", now).unwrap();
    service.add_tag(&mut store, "000001", "bank", now).unwrap();
    service.remove_tag(&mut store, "000001", "bank", now).unwrap();
    service.remove(&mut store, "000001", now).unwrap();

    let history = service.history(&store, Some("000001"), None);
    let actions: Vec<WatchlistAction> = history.into_iter().map(|event| event.action).collect();

    assert_eq!(
        actions,
        vec![
            WatchlistAction::Remove,
            WatchlistAction::TagRemove,
            WatchlistAction::TagAdd,
            WatchlistAction::Move,
            WatchlistAction::Add,
        ]
    );
}

#[test]
fn history_limit_keeps_latest_events() {
    let mut store = WatchlistStore::default();
    let service = WatchlistService::new(3);

    for index in 0..5 {
        let ts = fixed_ts() + chrono::Duration::seconds(index);
        let group_name = format!("g{}", index);
        service.create_group(&mut store, &group_name, ts).unwrap();
    }

    assert_eq!(store.history.len(), 3);
    assert_eq!(
        store.history,
        vec![
            WatchlistHistoryEvent {
                ts: fixed_ts() + chrono::Duration::seconds(2),
                action: WatchlistAction::GroupCreate,
                code: None,
                group: Some("g2".to_string()),
                tag: None,
            },
            WatchlistHistoryEvent {
                ts: fixed_ts() + chrono::Duration::seconds(3),
                action: WatchlistAction::GroupCreate,
                code: None,
                group: Some("g3".to_string()),
                tag: None,
            },
            WatchlistHistoryEvent {
                ts: fixed_ts() + chrono::Duration::seconds(4),
                action: WatchlistAction::GroupCreate,
                code: None,
                group: Some("g4".to_string()),
                tag: None,
            },
        ]
    );
}
