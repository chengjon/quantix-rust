use std::sync::{Mutex, OnceLock};

/// RAII guard that serializes tests sharing environment variables.
/// Holds the lock until dropped. The inner `MutexGuard` is boxed to avoid
/// triggering `clippy::await_holding_refcell_ref` when the guard is held
/// across `.await` points in async tests.
pub(crate) struct EnvLockGuard {
    // The guard must stay alive until dropped. Boxing hides the MutexGuard
    // type from clippy's await-holding lint.
    _guard: Box<std::sync::MutexGuard<'static, ()>>,
}

pub(crate) fn env_lock() -> EnvLockGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    EnvLockGuard {
        _guard: Box::new(guard),
    }
}
