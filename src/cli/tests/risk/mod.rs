//! Risk command tests, split by concern.
//!
//! Historical note: the original `risk.rs` was 615 lines and was split into
//! submodules per the file-size limits in CLAUDE.md:
//! - [`live_import`] — live-trades import + rebuild + live_import-status
//!   dispatch tests, the invalid daily-loss-rule rejection case, and the
//!   `--account` flag requirement case,
//! - [`parse`] — CLI parse tests for every risk subcommand, the rejected
//!   missing `--value`/`--type` case, and the three rule-set dispatch
//!   handlers.
//!
//! The env-lock + env-guard helpers are shared by both submodules, so they
//! live here at the `risk` module level as `pub(super)`.

pub(super) use super::*;

use std::sync::{Mutex, OnceLock};

pub(super) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

pub(super) struct RiskEnvGuard {
    risk_path: Option<String>,
    trade_path: Option<String>,
}

impl RiskEnvGuard {
    pub(super) fn capture() -> Self {
        Self {
            risk_path: std::env::var("QUANTIX_RISK_PATH").ok(),
            trade_path: std::env::var("QUANTIX_TRADE_PATH").ok(),
        }
    }
}

impl Drop for RiskEnvGuard {
    fn drop(&mut self) {
        match &self.risk_path {
            Some(value) => unsafe { std::env::set_var("QUANTIX_RISK_PATH", value) },
            None => unsafe { std::env::remove_var("QUANTIX_RISK_PATH") },
        }

        match &self.trade_path {
            Some(value) => unsafe { std::env::set_var("QUANTIX_TRADE_PATH", value) },
            None => unsafe { std::env::remove_var("QUANTIX_TRADE_PATH") },
        }
    }
}

mod live_import;
mod parse;
