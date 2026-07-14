//! Stop-service integration tests, split by concern.
//!
//! Historical note: the original `stop_service_test.rs` was 738 lines and
//! was split into submodules per the file-size limits in CLAUDE.md:
//! - [`evaluate`] — `evaluate_rule` / `evaluate_rules` trigger-detection
//!   tests plus the six `status_rows` anchor/diagnostic cases,
//! - [`upsert_history`] — set-rule persistence + validation, list/remove,
//!   update-patch + clear-threshold, and the history-event append cases.
//!
//! The fakes (`FakeStopRuleStore`, `FakeStopRuleStoreState`) and the three
//! helpers (`sample_time`, `sample_rule`, `quote_row`) are used by both
//! submodules, so they live here at the `stop_service_test` module level.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::monitor::MonitorQuoteRow;
use quantix_cli::stop::{
    StopHistoryEvent, StopHistoryEventType, StopRule, StopRuleStore, StopRuleUpdate, StopService,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(super) struct FakeStopRuleStore {
    pub(super) state: Arc<Mutex<FakeStopRuleStoreState>>,
}

#[derive(Default)]
pub(super) struct FakeStopRuleStoreState {
    pub(super) rules: Vec<StopRule>,
    pub(super) history: Vec<StopHistoryEvent>,
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

    async fn append_history(&self, event: StopHistoryEvent) -> Result<()> {
        self.state.lock().unwrap().history.push(event);
        Ok(())
    }

    async fn list_history(
        &self,
        filter: quantix_cli::stop::StopHistoryFilter,
    ) -> Result<Vec<StopHistoryEvent>> {
        let state = self.state.lock().unwrap();
        let mut history: Vec<StopHistoryEvent> = state
            .history
            .iter()
            .filter(|event| {
                filter
                    .code
                    .as_deref()
                    .map(|code| event.code == code)
                    .unwrap_or(true)
                    && filter
                        .date
                        .map(|date: NaiveDate| event.created_at.date_naive() == date)
                        .unwrap_or(true)
                    && filter
                        .event_type
                        .map(|event_type| event.event_type == event_type)
                        .unwrap_or(true)
            })
            .cloned()
            .collect();
        history.sort_by_key(|event| std::cmp::Reverse(event.created_at));
        if let Some(limit) = filter.limit {
            history.truncate(limit);
        }
        Ok(history)
    }

    async fn remove_rule(&self, code: &str) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.rules.len();
        state.rules.retain(|rule| rule.code != code);
        Ok(before != state.rules.len())
    }
}

pub(super) fn sample_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 11, 0, 0).unwrap()
}

pub(super) fn sample_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: None,
        take_profit_price: None,
        stop_loss_pct: None,
        take_profit_pct: None,
        trailing_pct: None,
        highest_price: None,
        reference_price: None,
        last_triggered_at: None,
        created_at: sample_time(),
        updated_at: sample_time(),
    }
}

pub(super) fn quote_row(code: &str, last_price: f64) -> MonitorQuoteRow {
    MonitorQuoteRow {
        code: code.to_string(),
        group: "core".to_string(),
        tags: Vec::new(),
        last_price: Some(last_price),
        change_pct: None,
        quote_time: Some(sample_time()),
        note: None,
    }
}

mod evaluate;
mod upsert_history;
