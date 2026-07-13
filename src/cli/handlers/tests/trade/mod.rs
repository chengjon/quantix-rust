//! Trade command tests, split by concern.
//!
//! Historical note: the original `trade.rs` was 726 lines and was split
//! into submodules per the file-size limits in CLAUDE.md:
//! - [`commands`] — init/reset/buy/sell write-path tests, plus the
//!   `FakeRiskIndustryResolver` helper,
//! - [`queries`] — position/cash/history/fees/overview read-path tests.
//!
//! The shared fakes (`FakePaperTradeStore`, `FakeTradeQuoteLookup`,
//! `trade_service`) are `pub(super)` and reused by other test modules, so
//! they live here at the `trade` module level.

// Re-export the parent test harness namespace so submodules can write
// `use super::*;` and pick up everything `tests/mod.rs` already imports.
pub(super) use super::*;

#[derive(Clone, Default)]
pub(super) struct FakePaperTradeStore {
    pub(super) state: Arc<Mutex<Option<PaperTradeState>>>,
}

impl FakePaperTradeStore {
    fn snapshot(&self) -> Option<PaperTradeState> {
        self.state.lock().unwrap().clone()
    }
}

#[async_trait]
impl PaperTradeStore for FakePaperTradeStore {
    async fn load_state(&self) -> Result<Option<PaperTradeState>> {
        Ok(self.snapshot())
    }

    async fn save_state(&self, state: &PaperTradeState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }
}

pub(super) fn trade_service() -> (TradeService<FakePaperTradeStore>, FakePaperTradeStore) {
    let store = FakePaperTradeStore::default();
    (TradeService::new(store.clone()), store)
}

#[derive(Clone, Default)]
pub(super) struct FakeTradeQuoteLookup {
    pub(super) quotes: HashMap<String, WatchlistQuoteSnapshot>,
    pub(super) fail: bool,
}

#[async_trait]
impl WatchlistQuoteLookup for FakeTradeQuoteLookup {
    async fn lookup_quotes(
        &self,
        _codes: &[String],
    ) -> Result<HashMap<String, WatchlistQuoteSnapshot>> {
        if self.fail {
            Err(QuantixError::Other("quote lookup failed".to_string()))
        } else {
            Ok(self.quotes.clone())
        }
    }
}

mod commands;
mod queries;
