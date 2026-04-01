use rust_decimal::Decimal;
use std::collections::HashSet;

use crate::core::Result;
use crate::screener::{
    PresetInvocation, PresetKind, RuleMatchDetail, ScreenRow, ScreenSortBy, ScreenUniverse,
};
use crate::watchlist::{WatchlistService, WatchlistStorage};

pub(super) fn resolve_codes(
    universe: ScreenUniverse,
    storage: &WatchlistStorage,
    watchlist_service: &WatchlistService,
) -> Result<Vec<String>> {
    match universe {
        ScreenUniverse::Codes(codes) => Ok(normalize_codes(codes)),
        ScreenUniverse::Watchlist { group } => {
            let store = storage.load_or_create()?;
            let items = watchlist_service.list(&store, group.as_deref(), None);
            Ok(normalize_codes(
                items.into_iter().map(|item| item.code).collect(),
            ))
        }
    }
}

pub(super) fn normalize_codes(codes: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for code in codes {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }

        let candidate = trimmed.to_string();
        if seen.insert(candidate.clone()) {
            normalized.push(candidate);
        }
    }

    normalized
}

pub(super) fn sort_rows(rows: &mut [ScreenRow], sort_by: ScreenSortBy) {
    match sort_by {
        ScreenSortBy::Code => rows.sort_by(|left, right| left.code.cmp(&right.code)),
        ScreenSortBy::Score => rows.sort_by(|left, right| {
            right
                .matched
                .cmp(&left.matched)
                .then_with(|| right.score.cmp(&left.score))
                .then_with(|| left.code.cmp(&right.code))
        }),
    }
}

pub(super) fn score_for_detail(
    preset: &PresetInvocation,
    detail: &RuleMatchDetail,
) -> Decimal {
    match (detail.actual_value, detail.threshold_value) {
        (Some(actual), Some(threshold)) => match preset.kind {
            PresetKind::CloseAboveMa | PresetKind::RsiGte | PresetKind::VolumeRatioGte => {
                actual - threshold
            }
            PresetKind::CloseBelowMa | PresetKind::RsiLte => threshold - actual,
        },
        _ => Decimal::ZERO,
    }
}
