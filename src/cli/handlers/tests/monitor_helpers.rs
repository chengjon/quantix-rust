use super::*;

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
        let id = state.next_id;
        state.next_id += 1;
        let alert = PriceAlert {
            id,
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

    async fn mark_triggered(&self, id: i64, triggered_at: chrono::DateTime<Utc>) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        if let Some(alert) = state.alerts.iter_mut().find(|alert| alert.id == id) {
            alert.last_triggered_at = Some(triggered_at);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FakeStopRuleState {
    pub(super) rules: Vec<StopRule>,
    pub(super) history: Vec<crate::stop::StopHistoryEvent>,
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

    async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .rules
            .iter()
            .find(|rule| rule.code == code)
            .cloned())
    }

    async fn append_history(&self, _event: crate::stop::StopHistoryEvent) -> Result<()> {
        self.state.lock().unwrap().history.push(_event);
        Ok(())
    }

    async fn list_history(
        &self,
        _filter: crate::stop::StopHistoryFilter,
    ) -> Result<Vec<crate::stop::StopHistoryEvent>> {
        Ok(self.state.lock().unwrap().history.clone())
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
