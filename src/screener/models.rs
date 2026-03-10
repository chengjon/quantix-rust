use rust_decimal::Decimal;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    CloseAboveMa,
    CloseBelowMa,
    RsiGte,
    RsiLte,
    VolumeRatioGte,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetInvocation {
    pub kind: PresetKind,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatchDetail {
    pub preset_name: String,
    pub params: BTreeMap<String, String>,
    pub actual_value: Option<Decimal>,
    pub threshold_value: Option<Decimal>,
    pub matched: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenUniverse {
    Codes(Vec<String>),
    Watchlist { group: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenSortBy {
    Code,
    Score,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenRunOptions {
    pub limit: Option<usize>,
    pub sort_by: ScreenSortBy,
}

impl Default for ScreenRunOptions {
    fn default() -> Self {
        Self {
            limit: None,
            sort_by: ScreenSortBy::Code,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenRow {
    pub code: String,
    pub matched: bool,
    pub score: Decimal,
    pub details: Vec<RuleMatchDetail>,
}
