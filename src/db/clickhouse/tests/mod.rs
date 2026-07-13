//! ClickHouse 单元测试与集成测试。
//!
//! 从原生 `clickhouse/tests.rs` 拆出。承载模型派生、行情行映射、分钟线
//! 辅助函数、mock sink 与 live round-trip 测试。Mock sink 结构体与其测试
//! 放在同一子模块，以便读取私有字段（私有字段对兄弟子模块不可见）。

mod live;
mod market;
mod minute;
mod sinks;
