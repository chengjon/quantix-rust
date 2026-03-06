/// 数据源适配器
///
/// 统一数据源接口，支持多数据源切换

pub mod tdx;
pub mod akshare;
pub mod quote_collector;
pub mod auction_collector;
pub mod kline_aggregator;

pub use tdx::{TdxSource, StockQuote};
pub use akshare::AkShareSource;
pub use quote_collector::{QuoteCollector, StockInfo as QuoteStockInfo};
pub use auction_collector::{AuctionCollector, AuctionQuote, WatchlistStock};
pub use kline_aggregator::{KlineAggregator, KlineData, KlineWindow, KlinePeriod};
