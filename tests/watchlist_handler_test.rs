use quantix_cli::cli::{
    handlers::run_watchlist_command, WatchlistCommands, WatchlistGroupCommands,
    WatchlistTagCommands,
};
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
}

impl WatchlistEnvGuard {
    fn capture() -> Self {
        Self {
            watchlist_path: std::env::var("QUANTIX_WATCHLIST_PATH").ok(),
        }
    }
}

impl Drop for WatchlistEnvGuard {
    fn drop(&mut self) {
        match &self.watchlist_path {
            Some(value) => unsafe { std::env::set_var("QUANTIX_WATCHLIST_PATH", value) },
            None => unsafe { std::env::remove_var("QUANTIX_WATCHLIST_PATH") },
        }
    }
}

#[tokio::test]
async fn add_command_persists_entry_to_configured_store() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
    }

    run_watchlist_command(WatchlistCommands::Add {
        code: "000001".to_string(),
        group: None,
    })
    .await
    .unwrap();

    let store = WatchlistStorage::new(&path).load().unwrap().unwrap();
    assert_eq!(
        store.groups.get("default").unwrap(),
        &vec!["000001".to_string()]
    );
    assert_eq!(store.history.len(), 1);
    assert_eq!(store.history[0].action, WatchlistAction::Add);
}

#[tokio::test]
async fn group_move_and_tag_commands_share_same_store() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
    }

    run_watchlist_command(WatchlistCommands::Group(WatchlistGroupCommands::Create {
        name: "core".to_string(),
    }))
    .await
    .unwrap();
    run_watchlist_command(WatchlistCommands::Add {
        code: "000001".to_string(),
        group: None,
    })
    .await
    .unwrap();
    run_watchlist_command(WatchlistCommands::Move {
        code: "000001".to_string(),
        group: "core".to_string(),
    })
    .await
    .unwrap();
    run_watchlist_command(WatchlistCommands::Tag(WatchlistTagCommands::Add {
        code: "000001".to_string(),
        tag: "bank".to_string(),
    }))
    .await
    .unwrap();

    let store = WatchlistStorage::new(&path).load().unwrap().unwrap();
    assert_eq!(store.groups.get("default").unwrap(), &Vec::<String>::new());
    assert_eq!(store.groups.get("core").unwrap(), &vec!["000001".to_string()]);
    assert_eq!(
        store.entries.get("000001").unwrap().tags,
        vec!["bank".to_string()]
    );
}

#[tokio::test]
async fn read_commands_succeed_when_store_file_is_missing() {
    let _lock = env_lock();
    let _guard = WatchlistEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.json");
    unsafe {
        std::env::set_var("QUANTIX_WATCHLIST_PATH", &path);
    }

    run_watchlist_command(WatchlistCommands::List {
        group: None,
        tag: None,
        with_price: false,
    })
    .await
    .unwrap();
    run_watchlist_command(WatchlistCommands::Group(WatchlistGroupCommands::List))
        .await
        .unwrap();
    run_watchlist_command(WatchlistCommands::History {
        code: None,
        limit: 20,
    })
    .await
    .unwrap();
}
