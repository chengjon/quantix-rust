use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::screener::{
    PresetInvocation, RuleMatchDetail, ScreenRow, ScreenRunOptions, ScreenSortBy, ScreenUniverse,
    evaluate_preset, required_lookback,
};
use crate::watchlist::{WatchlistService, WatchlistStorage};

#[async_trait]
pub trait DailyKlineLoader: Send + Sync {
    async fn load_daily_klines(&self, code: &str, lookback: usize) -> Result<Vec<Kline>>;
}

#[derive(Debug, Clone)]
pub struct ScreenerService<L> {
    loader: L,
    storage: WatchlistStorage,
    watchlist_service: WatchlistService,
}

impl<L> ScreenerService<L>
where
    L: DailyKlineLoader,
{
    pub fn new(loader: L, storage: WatchlistStorage) -> Self {
        Self {
            loader,
            storage,
            watchlist_service: WatchlistService::default(),
        }
    }

    pub async fn run(
        &self,
        universe: ScreenUniverse,
        presets: &[PresetInvocation],
        options: ScreenRunOptions,
    ) -> Result<Vec<ScreenRow>> {
        if presets.is_empty() {
            return Err(QuantixError::Other("至少需要一个 preset".to_string()));
        }

        let codes = self.resolve_codes(universe)?;
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        let mut lookback = 0usize;
        for preset in presets {
            lookback = lookback.max(required_lookback(preset)?);
        }

        let mut rows = Vec::with_capacity(codes.len());
        for code in codes {
            let klines = self.loader.load_daily_klines(&code, lookback).await?;
            let mut details = Vec::with_capacity(presets.len());
            let mut matched = true;
            let mut score = Decimal::ZERO;

            for preset in presets {
                let detail = evaluate_preset(preset, &klines)?;
                score += score_for_detail(preset, &detail);
                matched &= detail.matched;
                details.push(detail);
            }

            rows.push(ScreenRow {
                code,
                matched,
                score,
                details,
            });
        }

        sort_rows(&mut rows, options.sort_by);
        if let Some(limit) = options.limit {
            rows.truncate(limit.min(rows.len()));
        }

        Ok(rows)
    }

    fn resolve_codes(&self, universe: ScreenUniverse) -> Result<Vec<String>> {
        super::service_support::resolve_codes(universe, &self.storage, &self.watchlist_service)
    }
}

fn sort_rows(rows: &mut [ScreenRow], sort_by: ScreenSortBy) {
    super::service_support::sort_rows(rows, sort_by)
}

fn score_for_detail(preset: &PresetInvocation, detail: &RuleMatchDetail) -> Decimal {
    super::service_support::score_for_detail(preset, detail)
}
