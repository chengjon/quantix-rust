pub mod akshare;
pub mod auction_collector;
pub mod bridge_tdx;
pub mod eastmoney;
pub mod kline_aggregator;
pub mod openstock;
pub mod openstock_calendar;
pub mod openstock_client;
pub mod openstock_codes;
pub mod openstock_envelope;
pub mod openstock_index;
pub mod openstock_market;
pub mod openstock_shadow;
pub mod openstock_ticks;
pub mod quote_collector;
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
pub use openstock::{
    LiveShadowDrift, LiveShadowReport, LiveShadowRequest, LiveShadowStatus,
    OpenStockKlineParseError, live_shadow_error_into_quantix, parse_daily_kline_json,
    validate_live_shadow_payload,
};
pub use openstock_calendar::{
    CalendarParseError, TradeDate, TradeDateRecord, Workday, WorkdayRecord,
    calendar_error_into_quantix, parse_calendar_date, parse_trade_dates, parse_workdays,
};
pub use openstock_client::{OpenStockClient, OpenStockClientConfig, OpenStockResponse};
pub use openstock_codes::{
    StockCode, StockCodeParseError, StockCodeRecord, StockListEntry, StockListRecord,
    parse_all_stocks, parse_stock_codes, stock_code_error_into_quantix,
};
pub use openstock_envelope::{OpenStockEnvelope, OpenStockErrorEnvelope};
pub use openstock_index::{
    IndexKlineParseError, IndexKlineRecord, index_kline_error_into_quantix, parse_index_klines,
};
pub use openstock_market::OpenStockMarketReader;
pub use openstock_shadow::artifact_hash as openstock_artifact_hash;
pub use openstock_ticks::{
    TickEntry, TickEnvelopeRecord, TickMeta, TickParseError, parse_tick_data,
    tick_error_into_quantix,
};
pub use quote_collector::{QuoteCollector, StockInfo as QuoteStockInfo};
pub use tdx::{StockQuote, TdxSource};
pub use tdx_file::{
    FuquanCalculator, FuquanFactor, FuquanType, TdxDataImporter, TdxDayData, TdxDayFile,
    TdxDayRecord, TdxGbbqFile, TdxGbbqRecord,
};
pub use websocket::{
    ConnectionState, RealtimeQuote, Subscription, WebSocketClient, WebSocketConfig,
};
