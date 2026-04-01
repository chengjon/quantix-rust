pub mod akshare;
pub mod auction_collector;
mod auction_collector_support;
pub mod bridge_tdx;
mod bridge_tdx_support;
pub mod eastmoney;
mod eastmoney_support;
pub mod kline_aggregator;
pub mod quote_collector;
mod quote_collector_support;
/// 数据源适配器
///
/// 统一数据源接口，支持多数据源切换
pub mod tdx;
pub mod tdx_file;
pub mod websocket;

pub use akshare::AkShareSource;
pub use auction_collector::{AuctionCollector, AuctionQuote, WatchlistStock};
pub use bridge_tdx::BridgeTdxSource;
pub use eastmoney::{
    Board, EastMoneySource, FinancialData, MoneyFlowData, Quote, StockInfo as EastMoneyStockInfo,
};
pub use kline_aggregator::{KlineAggregator, KlineData, KlinePeriod, KlineWindow};
pub use quote_collector::{QuoteCollector, StockInfo as QuoteStockInfo};
pub use tdx::{StockQuote, TdxSource};
pub use tdx_file::{
    FuquanCalculator, FuquanFactor, FuquanType, TdxDataImporter, TdxDayData, TdxDayFile,
    TdxDayRecord, TdxGbbqFile, TdxGbbqRecord,
};
pub use websocket::{
    ConnectionState, RealtimeQuote, Subscription, WebSocketClient, WebSocketConfig,
};
