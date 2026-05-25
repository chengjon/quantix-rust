pub mod models;
pub mod sentiment;
pub mod service;
pub mod strength;
pub mod strength_runtime;

pub use models::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketOverview,
    MarketSentimentSnapshot, NorthFlowSnapshot,
};
pub use sentiment::{SentimentAggregator, SentimentData, SentimentProvider};
pub use service::{MarketDataReader, MarketService};
pub use strength::{
    AShareIndustryRow, MarketAnalysisFoundation, MarketFoundationSummary,
    MarketIndustryClassificationRow, MarketSnapshotRow, MarketStrengthReport, SectorCoverageRow,
    StrongSectorStockRow, build_market_analysis_foundation,
};
pub use strength_runtime::{analyze_market_strength_with_reader, load_market_analysis_foundation};
