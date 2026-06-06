use super::*;
use crate::core::config::{AkShareConfig, AppConfig, TdxApiConfig, TdxConfig};
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::io::{DataExporter, ExportConfig, ExportFormat};
use crate::sync::{DataSync, MarketFundamentalSyncRecord};
use chrono::NaiveDate;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Default, Serialize, Deserialize)]
struct DataSourceOverlay {
    #[serde(default)]
    default: DataSourceDefault,
    #[serde(default)]
    data_sources: PersistedDataSources,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DataSourceDefault {
    name: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedDataSources {
    tdx: Option<PersistedTdxConfig>,
    tdx_api: Option<PersistedTdxApiConfig>,
    akshare: Option<PersistedAkShareConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedTdxConfig {
    hosts: Vec<String>,
    port: u16,
    timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedTdxApiConfig {
    #[serde(default = "default_tdx_api_base_url")]
    base_url: String,
    #[serde(default = "default_tdx_api_timeout_secs")]
    timeout_secs: u64,
    #[serde(default = "default_tdx_api_max_retries")]
    max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedAkShareConfig {
    base_url: String,
    rate_limit: u32,
}

/// 查询 K线数据
pub(crate) async fn query_kline_data(
    code: String,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
) -> Result<()> {
    println!("📊 查询 K线数据");
    println!("  代码: {}", code);
    println!("  周期: {}", period_type);
    println!("  限制: {}", limit);

    // 解析日期
    let start_date = start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());
    let end_date = end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(&code, &period_type, start_date, end_date, Some(limit))
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // 显示数据
    println!("\n📈 查询结果 (共 {} 条):", klines.len());
    println!(
        "{:<12} {:<10} {:<10} {:<10} {:<10} {:<12}",
        "日期", "开盘", "最高", "最低", "收盘", "成交量"
    );
    println!("{}", "-".repeat(70));

    for kline in klines.iter().take(20) {
        println!(
            "{:<12} {:<10.2} {:<10.2} {:<10.2} {:<10.2} {:<12}",
            kline.date, kline.open, kline.high, kline.low, kline.close, kline.volume,
        );
    }

    if klines.len() > 20 {
        println!("... (还有 {} 条)", klines.len() - 20);
    }

    Ok(())
}

pub(crate) async fn import_market_fundamentals(input: String) -> Result<()> {
    let records = load_market_fundamental_records(&input)?;
    if records.is_empty() {
        return Err(QuantixError::Other(
            "输入文件中没有可导入的市场基础面记录".to_string(),
        ));
    }

    println!("📥 导入市场基础面快照");
    println!("  文件: {}", input);
    println!("  记录数: {}", records.len());

    let sync = DataSync::with_default_config().await?;
    let stats = sync.sync_market_fundamentals(&records).await?;

    println!("✅ 市场基础面快照导入完成");
    println!("  已写入: {}", stats.records_synced);
    println!("  耗时(秒): {}", stats.elapsed_seconds);
    Ok(())
}

pub(crate) fn list_data_sources(config_dir: &str) -> Result<()> {
    let app_config = load_app_config(config_dir)?;
    let overlay = load_data_source_overlay(config_dir)?;
    let default_source = resolve_default_source(&overlay, &app_config);

    println!("🔌 当前数据源配置");
    println!("  配置目录: {}", config_dir);
    println!("  覆盖文件: {}", overlay_path(config_dir).display());
    println!(
        "  默认数据源: {}",
        default_source.as_deref().unwrap_or("未设置")
    );
    println!();
    println!("{:<10} {:<8} {:<10} 摘要", "名称", "默认", "已配置");
    println!("{}", "-".repeat(72));

    let tdx_summary = app_config
        .data_sources
        .tdx
        .as_ref()
        .map(format_tdx_summary)
        .unwrap_or_else(|| "-".to_string());
    println!(
        "{:<10} {:<8} {:<10} {}",
        "tdx",
        if default_source.as_deref() == Some("tdx") {
            "yes"
        } else {
            "no"
        },
        if app_config.data_sources.tdx.is_some() {
            "yes"
        } else {
            "no"
        },
        tdx_summary
    );

    let tdx_api_summary = format_tdx_api_source_summary(&app_config);
    println!(
        "{:<10} {:<8} {:<10} {}",
        "tdx_api",
        if default_source.as_deref() == Some("tdx_api") {
            "yes"
        } else {
            "no"
        },
        if tdx_api_configured(&app_config) {
            "yes"
        } else {
            "no"
        },
        tdx_api_summary
    );

    let akshare_summary = app_config
        .data_sources
        .akshare
        .as_ref()
        .map(format_akshare_summary)
        .unwrap_or_else(|| "-".to_string());
    println!(
        "{:<10} {:<8} {:<10} {}",
        "akshare",
        if default_source.as_deref() == Some("akshare") {
            "yes"
        } else {
            "no"
        },
        if app_config.data_sources.akshare.is_some() {
            "yes"
        } else {
            "no"
        },
        akshare_summary
    );

    Ok(())
}

pub(crate) fn add_data_source(
    config_dir: &str,
    source_type: DataSourceKind,
    hosts: Vec<String>,
    port: Option<u16>,
    timeout: Option<u64>,
    base_url: Option<String>,
    rate_limit: Option<u32>,
) -> Result<()> {
    let app_config = load_app_config(config_dir)?;
    let mut overlay = load_data_source_overlay(config_dir)?;

    match source_type {
        DataSourceKind::Tdx => {
            let existing = app_config.data_sources.tdx.as_ref();
            let hosts = if hosts.is_empty() {
                existing
                    .map(|cfg| cfg.hosts.clone())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        QuantixError::Other(
                            "data source add --type tdx 需要提供 --hosts，或先存在可复用的 tdx 配置"
                                .to_string(),
                        )
                    })?
            } else {
                hosts
            };

            overlay.data_sources.tdx = Some(PersistedTdxConfig {
                hosts,
                port: port
                    .or_else(|| existing.map(|cfg| cfg.port))
                    .unwrap_or(7709),
                timeout: timeout
                    .or_else(|| existing.map(|cfg| cfg.timeout))
                    .unwrap_or(5000),
            });

            if overlay.default.name.is_none() {
                overlay.default.name = Some("tdx".to_string());
            }
        }
        DataSourceKind::Akshare => {
            let existing = app_config.data_sources.akshare.as_ref();
            let base_url = base_url
                .or_else(|| existing.map(|cfg| cfg.base_url.clone()))
                .ok_or_else(|| {
                    QuantixError::Other(
                        "data source add --type akshare 需要提供 --base-url，或先存在可复用的 akshare 配置"
                            .to_string(),
                    )
                })?;

            overlay.data_sources.akshare = Some(PersistedAkShareConfig {
                base_url,
                rate_limit: rate_limit
                    .or_else(|| existing.map(|cfg| cfg.rate_limit))
                    .unwrap_or(100),
            });

            if overlay.default.name.is_none() {
                overlay.default.name = Some("akshare".to_string());
            }
        }
        DataSourceKind::TdxApi => {
            return Err(QuantixError::Other(
                "tdx-api 数据源通过环境变量 TDX_API_URL 配置，无需 add".to_string(),
            ));
        }
    }

    write_data_source_overlay(config_dir, &overlay)?;
    println!(
        "✅ 已写入数据源配置: {} ({})",
        source_type.as_str(),
        overlay_path(config_dir).display()
    );
    Ok(())
}

pub(crate) fn set_default_data_source(config_dir: &str, name: DataSourceKind) -> Result<()> {
    let app_config = load_app_config(config_dir)?;

    let configured = match name {
        DataSourceKind::Tdx => app_config.data_sources.tdx.is_some(),
        DataSourceKind::TdxApi => tdx_api_configured(&app_config),
        DataSourceKind::Akshare => app_config.data_sources.akshare.is_some(),
    };

    if !configured {
        return Err(QuantixError::Other(format!(
            "无法设置默认数据源: {} 尚未配置",
            name.as_str()
        )));
    }

    let mut overlay = load_data_source_overlay(config_dir)?;
    overlay.default.name = Some(name.as_str().to_string());
    write_data_source_overlay(config_dir, &overlay)?;

    println!(
        "✅ 默认数据源已更新为 {} ({})",
        name.as_str(),
        overlay_path(config_dir).display()
    );
    Ok(())
}

pub(crate) async fn test_data_source(config_dir: &str, name: DataSourceKind) -> Result<()> {
    let app_config = load_app_config(config_dir)?;

    match name {
        DataSourceKind::Tdx => {
            test_tdx_data_source(
                app_config
                    .data_sources
                    .tdx
                    .as_ref()
                    .ok_or_else(|| QuantixError::Other("tdx 数据源尚未配置".to_string()))?,
            )
            .await
        }
        DataSourceKind::Akshare => {
            test_akshare_data_source(
                app_config
                    .data_sources
                    .akshare
                    .as_ref()
                    .ok_or_else(|| QuantixError::Other("akshare 数据源尚未配置".to_string()))?,
            )
            .await
        }
        DataSourceKind::TdxApi => {
            let client = crate::sources::tdx_api::TdxApiClient::from_env()?;
            client.health().await?;
            println!("✅ tdx-api 连接正常");
            Ok(())
        }
    }
}

/// 导出数据
pub(crate) async fn export_data(code: String, format: String, output: String) -> Result<()> {
    validate_data_export_format(&format)?;

    println!("📤 导出数据");
    println!("  代码: {}", code);
    println!("  格式: {}", format);
    println!("  输出: {}", output);

    // 创建输出目录
    std::fs::create_dir_all(&output)
        .map_err(|e| QuantixError::Other(format!("创建输出目录失败: {}", e)))?;

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(
            &code,
            "1d",
            None,
            None,
            Some(10000), // 导出时使用较大的限制
        )
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    let output_path = Path::new(&output).join(format!("{}.{}", code, format));
    let progress = ProgressBar::new(3);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
            .unwrap(),
    );

    progress.set_message("准备导出...");

    match format.as_str() {
        "csv" => {
            progress.set_message("写入 CSV...");
            progress.inc(1);

            export_klines_to_file(&klines, "csv", &output_path).await?;

            progress.inc(1);
            progress.finish_with_message("CSV 导出完成");
        }
        "parquet" => {
            progress.set_message("写入 Parquet...");
            progress.inc(1);

            export_klines_to_file(&klines, "parquet", &output_path).await?;

            progress.inc(1);
            progress.finish_with_message("Parquet 导出完成");
        }
        _ => return unsupported_data_export_format(&format),
    }

    println!("✅ 数据已导出到: {}", output_path.display());
    Ok(())
}

async fn export_klines_to_file<P: AsRef<Path>>(
    klines: &[Kline],
    format: &str,
    output_path: P,
) -> Result<()> {
    match format {
        "csv" => {
            let mut wtr = csv::Writer::from_path(output_path.as_ref())
                .map_err(|e| QuantixError::Other(format!("创建 CSV 文件失败: {}", e)))?;

            wtr.write_record(["date", "open", "high", "low", "close", "volume"])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 头失败: {}", e)))?;

            for kline in klines {
                wtr.write_record(&[
                    kline.date.to_string(),
                    kline.open.to_string(),
                    kline.high.to_string(),
                    kline.low.to_string(),
                    kline.close.to_string(),
                    kline.volume.to_string(),
                ])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 数据失败: {}", e)))?;
            }

            wtr.flush()
                .map_err(|e| QuantixError::Other(format!("刷新 CSV 失败: {}", e)))?;
            Ok(())
        }
        "parquet" => {
            let exporter = DataExporter::new(ExportConfig {
                format: ExportFormat::Parquet,
                ..Default::default()
            });
            exporter.export_klines(klines, output_path).await?;
            Ok(())
        }
        _ => unsupported_data_export_format(format),
    }
}

fn validate_data_export_format(format: &str) -> Result<()> {
    match format {
        "csv" | "parquet" => Ok(()),
        _ => unsupported_data_export_format(format),
    }
}

fn unsupported_data_export_format(format: &str) -> Result<()> {
    Err(QuantixError::Unsupported(format!(
        "data export format 不支持: {format}；支持: csv, parquet"
    )))
}

fn load_app_config(config_dir: &str) -> Result<AppConfig> {
    AppConfig::load(config_dir).map_err(|e| QuantixError::Config(format!("读取配置失败: {e}")))
}

fn load_market_fundamental_records(path: &str) -> Result<Vec<MarketFundamentalSyncRecord>> {
    let raw = fs::read_to_string(path)
        .map_err(|e| QuantixError::Other(format!("读取市场基础面文件失败 ({}): {}", path, e)))?;

    serde_json::from_str(&raw)
        .map_err(|e| QuantixError::Config(format!("解析市场基础面文件失败 ({}): {}", path, e)))
}

fn overlay_path(config_dir: &str) -> std::path::PathBuf {
    Path::new(config_dir).join("data_sources.toml")
}

fn load_data_source_overlay(config_dir: &str) -> Result<DataSourceOverlay> {
    let path = overlay_path(config_dir);
    if !path.exists() {
        return Ok(DataSourceOverlay::default());
    }

    let raw = fs::read_to_string(&path)
        .map_err(|e| QuantixError::Other(format!("读取数据源配置失败: {}", e)))?;

    toml::from_str(&raw)
        .map_err(|e| QuantixError::Config(format!("解析 {} 失败: {}", path.display(), e)))
}

fn write_data_source_overlay(config_dir: &str, overlay: &DataSourceOverlay) -> Result<()> {
    let path = overlay_path(config_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| QuantixError::Other(format!("创建配置目录失败: {}", e)))?;
    }

    let content = toml::to_string_pretty(overlay)
        .map_err(|e| QuantixError::Other(format!("序列化数据源配置失败: {}", e)))?;
    fs::write(&path, content)
        .map_err(|e| QuantixError::Other(format!("写入 {} 失败: {}", path.display(), e)))
}

fn resolve_default_source(overlay: &DataSourceOverlay, app_config: &AppConfig) -> Option<String> {
    overlay.default.name.clone().or_else(|| {
        if app_config.data_sources.tdx.is_some() {
            Some("tdx".to_string())
        } else if tdx_api_configured(app_config) {
            Some("tdx_api".to_string())
        } else if app_config.data_sources.akshare.is_some() {
            Some("akshare".to_string())
        } else {
            None
        }
    })
}

fn format_tdx_summary(config: &TdxConfig) -> String {
    format!(
        "hosts={} port={} timeout={}ms",
        config.hosts.join(","),
        config.port,
        config.timeout
    )
}

fn tdx_api_configured(app_config: &AppConfig) -> bool {
    app_config
        .data_sources
        .tdx_api
        .as_ref()
        .map(|config| !config.base_url.trim().is_empty())
        .unwrap_or(false)
        || tdx_api_env_url().is_some()
}

fn tdx_api_env_url() -> Option<String> {
    std::env::var("TDX_API_URL")
        .ok()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}

fn format_tdx_api_source_summary(app_config: &AppConfig) -> String {
    if let Some(config) = app_config.data_sources.tdx_api.as_ref() {
        format_tdx_api_summary(config)
    } else if let Some(base_url) = tdx_api_env_url() {
        format!(
            "base_url={} timeout={}s max_retries={}",
            base_url,
            tdx_api_env_timeout_secs(),
            default_tdx_api_max_retries()
        )
    } else {
        "-".to_string()
    }
}

fn format_tdx_api_summary(config: &TdxApiConfig) -> String {
    format!(
        "base_url={} timeout={}s max_retries={}",
        config.base_url, config.timeout_secs, config.max_retries
    )
}

fn format_akshare_summary(config: &AkShareConfig) -> String {
    format!(
        "base_url={} rate_limit={}",
        config.base_url, config.rate_limit
    )
}

fn default_tdx_api_base_url() -> String {
    "http://tdx-api:8080".to_string()
}

fn default_tdx_api_timeout_secs() -> u64 {
    30
}

fn default_tdx_api_max_retries() -> u32 {
    3
}

fn tdx_api_env_timeout_secs() -> u64 {
    std::env::var("TDX_API_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(default_tdx_api_timeout_secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{AdjustType, Kline};
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn remove(name: &'static str) -> Self {
            let previous = std::env::var_os(name);
            unsafe {
                std::env::remove_var(name);
            }
            Self { name, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => unsafe {
                    std::env::set_var(self.name, value);
                },
                None => unsafe {
                    std::env::remove_var(self.name);
                },
            }
        }
    }

    #[test]
    fn load_market_fundamental_records_parses_json_array() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "quantix-market-fundamentals-{}.json",
            std::process::id()
        ));

        fs::write(
            &path,
            r#"[{"code":"600519","snapshot_date":"2026-03-14","market_cap":23000.5,"latest_report_profit":862.1,"profit_source":"report","pe_dynamic":27.4}]"#,
        )
        .unwrap();

        let records = load_market_fundamental_records(path.to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "600519");
        assert_eq!(records[0].market_cap, Some(23000.5));
        assert_eq!(records[0].latest_report_profit, Some(862.1));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn set_default_tdx_api_requires_config_or_env() {
        let _guard = env_lock();
        let _tdx_api_url = EnvVarGuard::remove("TDX_API_URL");
        let dir = tempdir().unwrap();

        let err = set_default_data_source(dir.path().to_str().unwrap(), DataSourceKind::TdxApi)
            .unwrap_err();

        assert!(
            err.to_string().contains("tdx_api 尚未配置"),
            "unexpected error: {err}"
        );
        assert!(!overlay_path(dir.path().to_str().unwrap()).exists());
    }

    #[test]
    fn set_default_tdx_api_preserves_file_config() {
        let _guard = env_lock();
        let _tdx_api_url = EnvVarGuard::remove("TDX_API_URL");
        let dir = tempdir().unwrap();
        let config_path = overlay_path(dir.path().to_str().unwrap());
        fs::write(
            &config_path,
            r#"[data_sources.tdx_api]
base_url = "http://127.0.0.1:8080"
timeout_secs = 7
max_retries = 2
"#,
        )
        .unwrap();

        set_default_data_source(dir.path().to_str().unwrap(), DataSourceKind::TdxApi).unwrap();

        let written = fs::read_to_string(config_path).unwrap();
        assert!(written.contains("[data_sources.tdx_api]"));
        assert!(written.contains("base_url = \"http://127.0.0.1:8080\""));
        assert!(written.contains("timeout_secs = 7"));
        assert!(written.contains("max_retries = 2"));
    }

    #[tokio::test]
    async fn export_klines_writes_parquet_file() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("000001.parquet");
        let klines = vec![Kline {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            open: dec!(10.1),
            high: dec!(10.8),
            low: dec!(9.9),
            close: dec!(10.5),
            volume: 120000,
            amount: Some(dec!(1260000.0)),
            adjust_type: AdjustType::None,
        }];

        export_klines_to_file(&klines, "parquet", &output_path)
            .await
            .unwrap();

        assert!(output_path.exists());
        assert!(fs::metadata(output_path).unwrap().len() > 0);
    }
}

async fn test_tdx_data_source(config: &TdxConfig) -> Result<()> {
    println!("🧪 测试 tdx 数据源");
    println!("  hosts: {}", config.hosts.join(","));
    println!("  port: {}", config.port);
    println!("  timeout: {}ms", config.timeout);

    let timeout = Duration::from_millis(config.timeout.max(1));
    let mut last_error = None;

    for host in &config.hosts {
        match tokio::time::timeout(
            timeout,
            tokio::net::TcpStream::connect((host.as_str(), config.port)),
        )
        .await
        {
            Ok(Ok(_)) => {
                println!("✅ TDX 连通性正常: {}:{}", host, config.port);
                return Ok(());
            }
            Ok(Err(err)) => {
                println!("⚠️  {}:{} 连接失败: {}", host, config.port, err);
                last_error = Some(err.to_string());
            }
            Err(_) => {
                println!("⚠️  {}:{} 连接超时", host, config.port);
                last_error = Some("连接超时".to_string());
            }
        }
    }

    Err(QuantixError::Other(format!(
        "tdx 数据源测试失败: {}",
        last_error.unwrap_or_else(|| "没有可用主机".to_string())
    )))
}

async fn test_akshare_data_source(config: &AkShareConfig) -> Result<()> {
    println!("🧪 测试 akshare 数据源");
    println!("  base_url: {}", config.base_url);
    println!("  rate_limit: {}", config.rate_limit);

    let response = reqwest::Client::new()
        .get(&config.base_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| QuantixError::Other(format!("akshare 请求失败: {}", e)))?;

    println!("✅ AkShare 已响应: HTTP {}", response.status());
    Ok(())
}
