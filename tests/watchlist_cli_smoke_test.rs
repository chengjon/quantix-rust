#![allow(clippy::await_holding_lock)]

use clap::Parser;
use quantix_cli::Cli;
use quantix_cli::watchlist::{WatchlistAction, WatchlistStorage};
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

struct WatchlistEnvGuard {
    watchlist_path: Option<String>,
    postgres_url: Option<String>,
}

impl WatchlistEnvGuard {
    fn capture() -> Self {
        Self {
            watchlist_path: std::env::var("QUANTIX_WATCHLIST_PATH").ok(),
            postgres_url: std::env::var("POSTGRES_URL").ok(),
        }
    }
}

impl Drop for WatchlistEnvGuard {
    fn drop(&mut self) {
        match &self.watchlist_path {
            Some(value) => unsafe { std::env::set_var("QUANTIX_WATCHLIST_PATH", value) },
            None => unsafe { std::env::remove_var("QUANTIX_WATCHLIST_PATH") },
        }

        match &self.postgres_url {
            Some(value) => unsafe { std::env::set_var("POSTGRES_URL", value) },
            None => unsafe { std::env::remove_var("POSTGRES_URL") },
        }
    }
}

async fn run_cli(args: &[&str]) {
    let cli = Cli::try_parse_from(std::iter::once("quantix").chain(args.iter().copied())).unwrap();
    cli.run().await.unwrap();
}

#[tokio::test]
async fn smoke_add_list_remove_flow() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
        std::env::remove_var("POSTGRES_URL");
    }

    run_cli(&["watchlist", "add", "--code", "000001"]).await;
    run_cli(&["watchlist", "list"]).await;
    run_cli(&["watchlist", "remove", "--code", "000001"]).await;

    let store = WatchlistStorage::new(&path).load().unwrap().unwrap();
    assert_eq!(store.groups.get("default").unwrap(), &Vec::<String>::new());
    assert!(!store.entries.contains_key("000001"));
}

#[tokio::test]
async fn smoke_add_tag_and_filter_list_flow() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
        std::env::remove_var("POSTGRES_URL");
    }

    run_cli(&["watchlist", "add", "--code", "000001"]).await;
    run_cli(&[
        "watchlist",
        "tag",
        "add",
        "--code",
        "000001",
        "--tag",
        "bank",
    ])
    .await;
    run_cli(&["watchlist", "list", "--tag", "bank"]).await;

    let store = WatchlistStorage::new(&path).load().unwrap().unwrap();
    assert_eq!(
        store.entries.get("000001").unwrap().tags,
        vec!["bank".to_string()]
    );
}

#[tokio::test]
async fn smoke_move_and_history_flow() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
        std::env::remove_var("POSTGRES_URL");
    }

    run_cli(&["watchlist", "group", "create", "--name", "core"]).await;
    run_cli(&["watchlist", "add", "--code", "000001"]).await;
    run_cli(&["watchlist", "move", "--code", "000001", "--group", "core"]).await;
    run_cli(&["watchlist", "history", "--code", "000001", "--limit", "5"]).await;

    let store = WatchlistStorage::new(&path).load().unwrap().unwrap();
    assert_eq!(
        store.groups.get("core").unwrap(),
        &vec!["000001".to_string()]
    );
    let actions: Vec<WatchlistAction> = store
        .history
        .iter()
        .filter(|event| event.code.as_deref() == Some("000001"))
        .map(|event| event.action.clone())
        .collect();
    assert_eq!(actions, vec![WatchlistAction::Add, WatchlistAction::Move]);
}

#[tokio::test]
async fn smoke_list_with_price_flow_returns_ok() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
        std::env::remove_var("POSTGRES_URL");
    }

    run_cli(&["watchlist", "add", "--code", "000001"]).await;
    run_cli(&["watchlist", "list", "--with-price"]).await;

    let store = WatchlistStorage::new(&path).load().unwrap().unwrap();
    assert_eq!(
        store.groups.get("default").unwrap(),
        &vec!["000001".to_string()]
    );
}
