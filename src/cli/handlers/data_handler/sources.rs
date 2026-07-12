//! Data-source configuration overlay (list / add / set-default / test),
//! the persisted TDX/AkShare models, and the on-disk load/save helpers.

use crate::cli::command_types::DataSourceKind;
use crate::core::config::{AkShareConfig, AppConfig, TdxConfig};
use crate::core::{QuantixError, Result};
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
    akshare: Option<PersistedAkShareConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedTdxConfig {
    hosts: Vec<String>,
    port: u16,
    timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedAkShareConfig {
    base_url: String,
    rate_limit: u32,
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
    }
}

fn load_app_config(config_dir: &str) -> Result<AppConfig> {
    AppConfig::load(config_dir).map_err(|e| QuantixError::Config(format!("读取配置失败: {e}")))
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

fn format_akshare_summary(config: &AkShareConfig) -> String {
    format!(
        "base_url={} rate_limit={}",
        config.base_url, config.rate_limit
    )
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
