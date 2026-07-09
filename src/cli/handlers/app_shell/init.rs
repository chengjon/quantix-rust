use super::*;

pub async fn run_init(config_path: String) -> Result<()> {
    println!("🚀 初始化 Quantix CLI v{}", env!("CARGO_PKG_VERSION"));
    println!();

    // ── 1. 配置目录 ──
    let path = Path::new(&config_path);
    if path.exists() {
        println!("  ✅ 配置目录已存在: {}", config_path);
    } else {
        print!("  📁 创建配置目录: {} ... ", config_path);
        std::fs::create_dir_all(path).map_err(|e| {
            println!("❌");
            QuantixError::Other(format!("创建配置目录失败 ({}): {}", config_path, e))
        })?;
        println!("✅");
    }

    // ── 2. 加载运行时配置 ──
    println!("\n  ⚙️  加载运行时配置...");
    let rt = CliRuntime::load();
    println!(
        "    ClickHouse: {} db={}",
        rt.clickhouse.url, rt.clickhouse.database
    );
    println!(
        "    MySQL:      {} db={}",
        rt.upstream_mysql.url, rt.upstream_mysql.database
    );
    println!("    Bridge:     {}", rt.bridge.base_url);

    // ── 3. 创建数据目录 ──
    println!("\n  📂 准备数据目录...");
    let data_paths = [
        ("Watchlist", &rt.watchlist_path),
        ("Trade", &rt.trade_path),
        ("Risk", &rt.risk_path),
        ("Monitor DB", &rt.monitor_db_path),
        ("Monitor 配置", &rt.monitor_config_path),
        ("Strategy 配置", &rt.strategy_config_path),
        ("Strategy 运行时", &rt.strategy_runtime_db_path),
        ("Execution 配置", &rt.execution_config_path),
    ];

    let mut created = 0u32;
    let mut existing = 0u32;
    let mut failed = 0u32;
    for (label, file_path) in &data_paths {
        let parent = file_path.parent().unwrap_or(Path::new("."));
        if parent.exists() {
            existing += 1;
            println!("    ✅ {} -> {}", label, file_path.display());
        } else {
            match std::fs::create_dir_all(parent) {
                Ok(()) => {
                    created += 1;
                    println!("    🆕 {} -> {} (已创建)", label, file_path.display());
                }
                Err(e) => {
                    failed += 1;
                    println!("    ❌ {} -> {} (创建失败: {})", label, parent.display(), e);
                }
            }
        }
    }
    print!("    共 {} 个已存在, {} 个新建", existing, created);
    if failed > 0 {
        println!(", {} 个失败", failed);
        println!("    ⚠️  部分目录创建失败，后续操作可能出错");
    } else {
        println!();
    }

    // ── 4. 检查已有数据文件 ──
    println!("\n  📊 检查已有数据...");
    let mut has_data = false;
    for (label, file_path) in &data_paths {
        if file_path.exists() {
            has_data = true;
            let size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            println!(
                "    📄 {} ({}) - {} bytes",
                label,
                file_path.display(),
                size
            );
        }
    }
    if !has_data {
        println!("    ℹ️  暂无数据文件 (使用各子命令初始化时自动创建)");
    }

    // ── 5. 初始化 Polars ──
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    print!("\n  🔧 初始化 Polars 计算引擎... ");
    crate::analysis::polars_adapter::init_polars()?;
    println!("✅ ({} 线程)", cpu_count);

    // ── 6. 环境检查 ──
    println!("\n  🔍 环境检查...");
    if Path::new(".env").exists() {
        println!("    ✅ .env 文件已找到");
    } else {
        println!("    ℹ️  未找到 .env 文件 (使用默认配置)");
    }

    let home = std::env::var_os("HOME")
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|| "(未设置)".to_string());
    println!("    HOME: {}", home);
    println!("    CPU:  {} 核心", cpu_count);

    // ── 7. 数据库连通性探测 ──
    println!("\n  🌐 数据库连通性探测...");

    let (ch_ok, mysql_ok, bridge_ok) = tokio::join!(
        probe_tcp_async(&rt.clickhouse.url),
        probe_tcp_async(&rt.upstream_mysql.url),
        probe_tcp_async(&rt.bridge.base_url),
    );

    if ch_ok {
        println!(
            "    ✅ ClickHouse ({}) - 可达 (db: {})",
            rt.clickhouse.url, rt.clickhouse.database
        );
    } else {
        println!(
            "    ⚠️  ClickHouse ({}) - 不可达 (后续查询操作将失败, db: {})",
            rt.clickhouse.url, rt.clickhouse.database
        );
    }

    if mysql_ok {
        println!(
            "    ✅ MySQL ({}) - 可达 (db: {})",
            rt.upstream_mysql.url, rt.upstream_mysql.database
        );
    } else {
        println!(
            "    ⚠️  MySQL ({}) - 不可达 (db: {})",
            rt.upstream_mysql.url, rt.upstream_mysql.database
        );
    }

    if bridge_ok {
        println!("    ✅ Bridge ({}) - 可达", rt.bridge.base_url);
    } else {
        println!(
            "    ⚠️  Bridge ({}) - 不可达 (行情数据源不可用)",
            rt.bridge.base_url
        );
    }

    // ── 汇总 ──
    println!();
    println!("✅ 初始化完成！");
    println!();
    println!("  📝 可用命令:");
    println!("    quantix status --health   查看系统健康状态");
    println!("    quantix data query        查询K线数据");
    println!("    quantix strategy list     查看策略列表");
    println!("    quantix trade init        初始化模拟交易账户");
    println!("    quantix task start        启动任务调度器");
    println!("    quantix menu              进入交互菜单");

    Ok(())
}
