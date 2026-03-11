use super::*;

pub(super) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

pub(super) struct ClickHouseDbEnvGuard(Option<String>);

impl ClickHouseDbEnvGuard {
    pub(super) fn capture() -> Self {
        Self(std::env::var(CLICKHOUSE_DB_ENV).ok())
    }
}

impl Drop for ClickHouseDbEnvGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
        }
    }
}

pub(super) fn make_kline(code: &str, day: u32, close: Decimal, volume: i64) -> Kline {
    Kline {
        code: code.to_string(),
        date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
        open: close,
        high: close + dec!(1),
        low: close - dec!(1),
        close,
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[derive(Clone, Default)]
pub(super) struct FakeLoader {
    pub(super) data: HashMap<String, Vec<Kline>>,
}

#[async_trait]
impl DailyKlineLoader for FakeLoader {
    async fn load_daily_klines(
        &self,
        code: &str,
        lookback: usize,
    ) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }
        Ok(rows)
    }
}

pub(super) fn monitor_sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 10, 30, 0).unwrap()
}

pub(super) fn monitor_watchlist_item(code: &str, group: &str, tags: &[&str]) -> WatchlistListItem {
    WatchlistListItem {
        code: code.to_string(),
        group: group.to_string(),
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

pub(super) fn monitor_quote_row(code: &str, last_price: f64, change_pct: f64) -> MonitorQuoteRow {
    MonitorQuoteRow {
        code: code.to_string(),
        group: String::new(),
        tags: Vec::new(),
        last_price: Some(last_price),
        change_pct: Some(change_pct),
        quote_time: Some(monitor_sample_time()),
        note: None,
    }
}

pub(super) fn monitor_alert(
    id: i64,
    code: &str,
    kind: PriceAlertKind,
    target_price: f64,
) -> PriceAlert {
    PriceAlert {
        id,
        code: code.to_string(),
        kind,
        target_price,
        created_at: monitor_sample_time(),
        last_triggered_at: None,
    }
}

#[derive(Clone, Default)]
pub(super) struct FakeMonitorWatchlistReader {
    pub(super) items: Vec<WatchlistListItem>,
}

#[async_trait]
impl MonitorWatchlistReader for FakeMonitorWatchlistReader {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
        Ok(self.items.clone())
    }
}

#[derive(Clone, Default)]
pub(super) struct FakeMonitorQuoteReader {
    pub(super) rows: Vec<MonitorQuoteRow>,
}

#[async_trait]
impl MonitorQuoteReader for FakeMonitorQuoteReader {
    async fn load_quotes(&self, _codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
        Ok(self.rows.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FakeMonitorAlertState {
    pub(super) next_id: i64,
    pub(super) alerts: Vec<PriceAlert>,
    pub(super) removed_ids: Vec<i64>,
}

#[derive(Clone, Default)]
pub(super) struct FakeMonitorAlertStore {
    pub(super) state: Arc<Mutex<FakeMonitorAlertState>>,
}

#[async_trait]
impl MonitorAlertStore for FakeMonitorAlertStore {
    async fn add_alert(
        &self,
        code: &str,
        kind: PriceAlertKind,
        target_price: f64,
        now: chrono::DateTime<Utc>,
    ) -> Result<PriceAlert> {
        let mut state = self.state.lock().unwrap();
        state.next_id += 1;
        let alert = PriceAlert {
            id: state.next_id,
            code: code.to_string(),
            kind,
            target_price,
            created_at: now,
            last_triggered_at: None,
        };
        state.alerts.push(alert.clone());
        Ok(alert)
    }

    async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
        Ok(self.state.lock().unwrap().alerts.clone())
    }

    async fn remove_alert(&self, id: i64) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.alerts.len();
        state.alerts.retain(|alert| alert.id != id);
        if before != state.alerts.len() {
            state.removed_ids.push(id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn mark_triggered(
        &self,
        id: i64,
        triggered_at: chrono::DateTime<Utc>,
    ) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let Some(alert) = state.alerts.iter_mut().find(|alert| alert.id == id) else {
            return Ok(false);
        };
        alert.last_triggered_at = Some(triggered_at);
        Ok(true)
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FakeStopRuleState {
    pub(super) rules: Vec<StopRule>,
    pub(super) removed_codes: Vec<String>,
}

#[derive(Clone, Default)]
pub(super) struct FakeStopRuleStore {
    pub(super) state: Arc<Mutex<FakeStopRuleState>>,
}

#[async_trait]
impl StopRuleStore for FakeStopRuleStore {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
        let mut state = self.state.lock().unwrap();
        if let Some(existing) = state
            .rules
            .iter_mut()
            .find(|existing| existing.code == rule.code)
        {
            *existing = rule.clone();
        } else {
            state.rules.push(rule.clone());
        }
        Ok(rule)
    }

    async fn list_rules(&self) -> Result<Vec<StopRule>> {
        Ok(self.state.lock().unwrap().rules.clone())
    }

    async fn remove_rule(&self, code: &str) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.rules.len();
        state.rules.retain(|rule| rule.code != code);
        if before != state.rules.len() {
            state.removed_codes.push(code.to_string());
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub(super) fn stop_sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
}

pub(super) fn stop_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: Some(14.5),
        take_profit_price: None,
        trailing_pct: None,
        highest_price: None,
        last_triggered_at: None,
        created_at: stop_sample_time(),
        updated_at: stop_sample_time(),
    }
}

pub(super) fn stop_watchlist_storage(codes: &[&str]) -> (tempfile::TempDir, WatchlistStorage) {
    let dir = tempfile::tempdir().unwrap();
    let storage = WatchlistStorage::new(dir.path().join("watchlist.json"));
    let service = WatchlistService::default();
    let mut store = storage.load_or_create().unwrap();
    for code in codes {
        service.add(&mut store, code, None, Utc::now()).unwrap();
    }
    storage.save(&store).unwrap();
    (dir, storage)
}

#[derive(Clone, Default)]
pub(super) struct FakePaperTradeStore {
    pub(super) state: Arc<Mutex<Option<PaperTradeState>>>,
}

impl FakePaperTradeStore {
    pub(super) fn snapshot(&self) -> Option<PaperTradeState> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarketBoardRequest {
    pub(super) board_type: BoardType,
    pub(super) date: Option<NaiveDate>,
    pub(super) limit: usize,
    pub(super) sort_by: BoardSortBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarketLeaderRequest {
    pub(super) filter: LeaderFilter,
    pub(super) limit: usize,
    pub(super) date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct FakeMarketState {
    pub(super) board_requests: Vec<MarketBoardRequest>,
    pub(super) leader_requests: Vec<MarketLeaderRequest>,
}

#[derive(Clone)]
pub(super) struct FakeMarketReader {
    pub(super) state: Arc<Mutex<FakeMarketState>>,
}

impl FakeMarketReader {
    pub(super) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeMarketState::default())),
        }
    }
}

#[async_trait]
impl MarketDataReader for FakeMarketReader {
    async fn load_board_rankings(
        &self,
        board_type: BoardType,
        date: Option<NaiveDate>,
        limit: usize,
        sort_by: BoardSortBy,
    ) -> Result<Vec<BoardRankRow>> {
        self.state
            .lock()
            .unwrap()
            .board_requests
            .push(MarketBoardRequest {
                board_type,
                date,
                limit,
                sort_by,
            });

        let rows = match board_type {
            BoardType::Sector => vec![BoardRankRow::new("BK001", "银行", board_type, 1, 2.1)],
            BoardType::Concept => {
                vec![BoardRankRow::new("GN001", "人工智能", board_type, 1, 4.2)]
            }
        };

        Ok(rows.into_iter().take(limit).collect())
    }

    async fn load_north_flow(&self, date: Option<NaiveDate>) -> Result<Option<NorthFlowSnapshot>> {
        Ok(Some(NorthFlowSnapshot::new(
            date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
            12.3,
            8.6,
            20.9,
            100.0,
        )))
    }

    async fn load_market_sentiment(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Option<MarketSentimentSnapshot>> {
        Ok(Some(MarketSentimentSnapshot::new(
            date.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()),
            3210,
            1875,
            87,
            4,
            0.81,
            0.19,
            23,
        )))
    }

    async fn load_leaders(
        &self,
        filter: LeaderFilter,
        limit: usize,
        date: Option<NaiveDate>,
    ) -> Result<Vec<LeaderRow>> {
        self.state
            .lock()
            .unwrap()
            .leader_requests
            .push(MarketLeaderRequest {
                filter: filter.clone(),
                limit,
                date,
            });

        let rows = match filter {
            LeaderFilter::Sector(name) => {
                vec![LeaderRow::new("600000", "浦发银行", Some(name), None, 5.6)]
            }
            LeaderFilter::Concept(name) => {
                vec![LeaderRow::new("300024", "机器人", None, Some(name), 7.1)]
            }
            LeaderFilter::All => vec![
                LeaderRow::new("300024", "机器人", None, Some("人工智能".to_string()), 7.1),
                LeaderRow::new("600000", "浦发银行", Some("银行".to_string()), None, 5.6),
            ],
        };

        Ok(rows.into_iter().take(limit).collect())
    }
}
