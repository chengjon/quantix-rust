/// quantix-cli - A股量化交易 CLI 工具
///
/// 与 Python quantix 项目共享数据源和数据库
///
/// 运行方式:
///   - cargo run -- init          # 初始化配置
///   - cargo run -- data query   # 查询数据
///   - cargo run -- menu         # 交互菜单
use clap::Parser;
use quantix_cli::{Cli, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "quantix_cli=info,sqlx=warn".to_string()),
        )
        .init();

    // 解析 CLI 命令
    Cli::parse().run().await
}
