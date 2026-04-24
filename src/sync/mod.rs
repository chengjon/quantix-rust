/// 数据同步模块
///
/// Python quantix ↔ quantix-rust 数据同步
pub mod etl;

pub use etl::{DataSync, MarketFundamentalSyncRecord, SyncConfig, SyncStats};
