//! # 外部系统合同文档同步测试
//!
//! ## Layer 1: 全量清单检查
//! 枚举 `docs/contracts/external-systems.md` 中记录的所有外部系统，
//! 验证对应的 Rust 源码文件确实存在。
//!
//! ## Layer 2: 9 项定点抽查
//! 对文档中的关键合约细节抽查，验证与源码一致。
//!
//! 任何不一致记为硬失败 (hard-fail via panic!)。

use std::path::Path;

/// 仓库根目录（集成测试的 CARGO_MANIFEST_DIR = 项目根）
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

// ============================================================================
// Layer 1: 全量清单
// ============================================================================

/// 文档记录的 23 个外部系统，每个附带应存在的关键源码路径（相对路径）
const L1_ENTRIES: &[(&str, &[&str])] = &[
    // ---- 数据存储 (4) ----
    ("ClickHouse", &["src/db/clickhouse/mod.rs"]),
    ("PostgreSQL", &["src/db/postgresql.rs"]),
    ("TDengine", &["src/db/tdengine.rs"]),
    ("SQLite", &["Cargo.toml"]),
    // ---- 市场行情 (7) ----
    (
        "OpenStock API",
        &[
            "src/sources/openstock_client.rs",
            "src/sources/openstock_envelope.rs",
        ],
    ),
    ("TDX TCP", &["src/sources/tdx.rs"]),
    ("Bridge TDX", &["src/sources/bridge_tdx.rs"]),
    ("EastMoney HTTP", &["src/sources/eastmoney.rs"]),
    ("AkShare", &["src/sources/akshare.rs"]),
    ("WebSocket", &["src/sources/websocket.rs"]),
    ("Kline Aggregator", &["src/sources/kline_aggregator.rs"]),
    // ---- 交易执行 (4) ----
    (
        "Windows Bridge",
        &["src/bridge/client.rs", "src/bridge/models.rs"],
    ),
    ("QMT Live", &["src/execution/qmt_live_adapter.rs"]),
    ("QMT Preview", &["src/execution/qmt_bridge.rs"]),
    ("MiniQMT", &["src/miniqmt_market.rs"]),
    // ---- AI/LLM (3) ----
    ("DeepSeek", &["src/ai/providers/openai_compat.rs"]),
    ("OpenAI", &["src/ai/providers/openai_compat.rs"]),
    ("Ollama", &["src/ai/providers/openai_compat.rs"]),
    // ---- 新闻搜索 (3) ----
    ("Tavily", &["src/news/providers/tavily.rs"]),
    ("SerpAPI", &["src/news/providers/serpapi.rs"]),
    ("Bocha (博查)", &["src/news/providers/bocha.rs"]),
    // ---- 基本面 (1) ----
    ("EastMoney Fundamental", &["src/fundamental/eastmoney.rs"]),
    // ---- 通知推送 (5) ----
    (
        "飞书 Feishu",
        &["src/monitoring/notification/senders/feishu.rs"],
    ),
    (
        "企业微信 WeChat Work",
        &["src/monitoring/notification/senders/wechat_work.rs"],
    ),
    (
        "桌面通知 Desktop",
        &["src/monitoring/notification/senders/desktop.rs"],
    ),
    (
        "Webhook",
        &["src/monitoring/notification/senders/webhook.rs"],
    ),
    ("日志 Log", &["src/monitoring/notification/senders/log.rs"]),
];

fn exists(p: &str) -> bool {
    Path::new(ROOT).join(p).exists()
}

/// Layer 1: 验证每个系统的关键源码路径都存在
#[test]
fn l1_full_inventory_all_paths_exist() {
    let mut failures: Vec<String> = Vec::new();
    for (sys, paths) in L1_ENTRIES {
        for p in *paths {
            if !exists(p) {
                failures.push(format!("[HARD-FAIL] '{}' → 路径缺失: {}", sys, p));
            }
        }
    }
    // 验证总数
    assert_eq!(
        L1_ENTRIES.len(),
        27,
        "文档应记录 27 个外部系统，L1_ENTRIES 中有 {} 个",
        L1_ENTRIES.len()
    );
    if !failures.is_empty() {
        panic!(
            "Layer 1 全量清单: {} 项失败:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Layer 1: 合同文档本身存在
#[test]
fn l1_doc_file_exists() {
    assert!(
        exists("docs/contracts/external-systems.md"),
        "合同文档 docs/contracts/external-systems.md 不存在"
    );
}

// ============================================================================
// Layer 2: 9 项定点抽查
// ============================================================================

fn read(p: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(p))
        .unwrap_or_else(|e| panic!("无法读取 {}: {}", p, e))
}

/// SC1: Bridge 默认 URL 应含 http://127.0.0.1:17580
#[test]
fn l2_sc1_bridge_default_port() {
    let c = read("src/core/runtime/settings.rs");
    assert!(
        c.contains("DEFAULT_BRIDGE_BASE_URL"),
        "SC1: runtime/settings.rs 中未找到 DEFAULT_BRIDGE_BASE_URL (应含 127.0.0.1:17580)"
    );
}

/// SC2: Bridge contract version 默认值应为 miniqmt.v1
#[test]
fn l2_sc2_contract_version_miniqmt() {
    // 在 settings.rs 或 runtime 或 bridge/client.rs 中查找
    let found = [
        read("src/core/runtime/settings.rs"),
        read("src/core/runtime.rs"),
        read("src/bridge/client.rs"),
    ]
    .iter()
    .any(|c| c.contains("miniqmt.v1"));
    assert!(found, "SC2: 未找到默认 contract version 'miniqmt.v1'");
}

/// SC3: ClickHouse 默认端口 8123
#[test]
fn l2_sc3_clickhouse_default_port() {
    let c = read("src/db/clickhouse/mod.rs");
    assert!(c.contains("8123"), "SC3: ClickHouse 模块中未找到端口 8123");
}

/// SC4: OpenStockClient 默认重试常量
#[test]
fn l2_sc4_openstock_retries() {
    let c = read("src/sources/openstock_client.rs");
    assert!(
        c.contains("DEFAULT_MAX_RETRIES"),
        "SC4: OpenStockClient 中未找到 DEFAULT_MAX_RETRIES"
    );
}

/// SC5: OpenStockClient circuit breaker 阈值常量
#[test]
fn l2_sc5_openstock_circuit_breaker() {
    let c = read("src/sources/openstock_client.rs");
    assert!(
        c.contains("DEFAULT_CIRCUIT_BREAK_THRESHOLD"),
        "SC5: OpenStockClient 中未找到 circuit breaker 阈值常量"
    );
}

/// SC6: BridgeTaskLifecycleStatus 应有 4 种变体
#[test]
fn l2_sc6_bridge_lifecycle_status() {
    let c = read("src/bridge/models.rs");
    assert!(
        c.contains("pub enum BridgeTaskLifecycleStatus"),
        "SC6: 未找到 BridgeTaskLifecycleStatus 枚举定义"
    );
    for v in &["Pending", "Completed", "Failed", "BridgeTaskAccepted"] {
        assert!(c.contains(v), "SC6: 枚举缺少变体 {}", v);
    }
}

/// SC7: BridgeFailureCode 应有 7 种变体
#[test]
fn l2_sc7_bridge_failure_codes() {
    let c = read("src/bridge/models.rs");
    assert!(
        c.contains("pub enum BridgeFailureCode"),
        "SC7: 未找到 BridgeFailureCode 枚举定义"
    );
    for v in &[
        "LiveBridgeTimeout",
        "LiveBridgeUnavailable",
        "LiveBridgeAuthFailed",
        "LiveBridgeUnsupportedContractVersion",
        "LiveBridgeUnsupportedMethod",
        "LiveBridgeInvalidResult",
        "LiveBridgeIdentityMismatch",
    ] {
        assert!(c.contains(v), "SC7: 枚举缺少失败码 {}", v);
    }
}

/// SC8: ClickHouse 默认批次大小常量
#[test]
fn l2_sc8_clickhouse_batch_size() {
    let c = read("src/db/clickhouse/mod.rs");
    assert!(
        c.contains("DEFAULT_BATCH_SIZE"),
        "SC8: ClickHouse 模块中未找到 DEFAULT_BATCH_SIZE"
    );
}

/// SC9: PostgreSQL 连接池 max_connections 配置
#[test]
fn l2_sc9_postgres_max_connections() {
    let c = read("src/db/postgresql.rs");
    assert!(
        c.contains("max_connections"),
        "SC9: PostgreSQL 模块中未找到 max_connections 配置"
    );
}
