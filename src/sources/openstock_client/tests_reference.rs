//! Tests for the reference-data family (fetch_stock_codes,
//! fetch_trade_dates, fetch_all_stocks, fetch_workdays).
//!
//! These methods are thin wrappers over [`OpenStockClient::fetch`]; their
//! wire behavior is covered by the core fetch tests in tests_core.rs. This
//! file is reserved for future reference-specific regression coverage.
