use super::*;
use crate::strategy::{ConfiguredStock, ConfiguredStrategyInstance};
use serde_json::{Map, Value};

pub(crate) async fn execute_strategy_create(
    id: String,
    name: String,
    code: String,
    params: Vec<String>,
    disabled: bool,
) -> Result<()> {
    let store = create_strategy_config_store();
    let config = execute_strategy_create_with_store(&store, &id, &name, &code, &params, !disabled)?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_strategy_update(
    id: String,
    name: Option<String>,
    code: Option<String>,
    params: Vec<String>,
    enable: bool,
    disable: bool,
) -> Result<()> {
    let store = create_strategy_config_store();
    let enabled = if enable {
        Some(true)
    } else if disable {
        Some(false)
    } else {
        None
    };
    let config = execute_strategy_update_with_store(
        &store,
        &id,
        name.as_deref(),
        code.as_deref(),
        &params,
        enabled,
    )?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_strategy_delete(id: String) -> Result<()> {
    let store = create_strategy_config_store();
    let config = execute_strategy_delete_with_store(&store, &id)?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) fn execute_strategy_create_with_store(
    store: &JsonStrategyConfigStore,
    id: &str,
    name: &str,
    code: &str,
    params: &[String],
    enabled: bool,
) -> Result<StrategyDaemonConfig> {
    validate_strategy_name(name)?;
    let mut config = store.load_or_create()?;
    if find_instance(&config, id).is_some() {
        return Err(QuantixError::Other(format!("策略实例已存在: {id}")));
    }

    let stock = find_or_create_stock_mut(&mut config, code);
    stock.strategies.push(ConfiguredStrategyInstance {
        id: id.to_string(),
        name: name.to_string(),
        enabled,
        params: parse_strategy_params(params)?,
    });

    store.save(&config)?;
    Ok(config)
}

pub(crate) fn execute_strategy_update_with_store(
    store: &JsonStrategyConfigStore,
    id: &str,
    name: Option<&str>,
    code: Option<&str>,
    params: &[String],
    enabled: Option<bool>,
) -> Result<StrategyDaemonConfig> {
    let mut config = store.load_or_create()?;
    let (current_code, mut instance) = remove_instance(&mut config, id)
        .ok_or_else(|| QuantixError::Other(format!("未找到策略实例: {id}")))?;

    if let Some(name) = name {
        validate_strategy_name(name)?;
        instance.name = name.to_string();
    }
    if !params.is_empty() {
        instance.params = parse_strategy_params(params)?;
    }
    if let Some(enabled) = enabled {
        instance.enabled = enabled;
    }

    let target_code = code.unwrap_or(&current_code);
    find_or_create_stock_mut(&mut config, target_code)
        .strategies
        .push(instance);
    drop_empty_stocks(&mut config);
    store.save(&config)?;
    Ok(config)
}

pub(crate) fn execute_strategy_delete_with_store(
    store: &JsonStrategyConfigStore,
    id: &str,
) -> Result<StrategyDaemonConfig> {
    let mut config = store.load_or_create()?;
    remove_instance(&mut config, id)
        .ok_or_else(|| QuantixError::Other(format!("未找到策略实例: {id}")))?;
    drop_empty_stocks(&mut config);
    store.save(&config)?;
    Ok(config)
}

pub(crate) fn print_strategy_catalog_and_instances(config: &StrategyDaemonConfig) {
    println!("📋 内置策略:");
    println!();
    println!("  1. ma_cross - 均线交叉策略");
    println!("     描述: MA5 上穿 MA20 买入，下穿卖出");
    println!("     运行: quantix strategy run --name ma_cross --mode backtest --code 000001");
    println!();
    println!("🧩 已配置策略实例:");

    let rows = list_instance_rows(config);
    if rows.is_empty() {
        println!("  暂无策略实例，可使用 `quantix strategy create` 创建");
        return;
    }

    println!(
        "  {:<24} {:<12} {:<10} {:<8} 参数",
        "实例 ID", "策略", "代码", "启用"
    );
    println!("  {}", "-".repeat(76));
    for row in rows {
        println!(
            "  {:<24} {:<12} {:<10} {:<8} {}",
            row.id,
            row.name,
            row.code,
            if row.enabled { "yes" } else { "no" },
            row.params
        );
    }
}

pub(crate) async fn show_strategy_or_instance(
    target: Option<String>,
    id: Option<String>,
) -> Result<()> {
    let target = id
        .or(target)
        .ok_or_else(|| QuantixError::Other("strategy show 需要提供 --name 或 --id".to_string()))?;

    let store = create_strategy_config_store();
    let config = store.load_or_create()?;
    if let Some((code, instance)) = find_instance(&config, &target) {
        println!("📖 策略实例详情: {}", instance.id);
        println!("  策略: {}", instance.name);
        println!("  代码: {}", code);
        println!("  启用: {}", if instance.enabled { "yes" } else { "no" });
        println!("  参数: {}", format_params(&instance.params));
        return Ok(());
    }

    show_strategy(target).await
}

fn validate_strategy_name(name: &str) -> Result<()> {
    match name {
        "ma_cross" => Ok(()),
        other => Err(QuantixError::Other(format!(
            "当前仅支持内置策略: ma_cross，收到 {other}"
        ))),
    }
}

fn parse_strategy_params(params: &[String]) -> Result<Value> {
    let mut object = Map::new();
    for item in params {
        let (key, value) = item
            .split_once('=')
            .ok_or_else(|| QuantixError::Other(format!("参数格式非法，期望 key=value: {item}")))?;
        object.insert(key.trim().to_string(), parse_param_value(value.trim()));
    }
    Ok(Value::Object(object))
}

fn parse_param_value(raw: &str) -> Value {
    if let Ok(value) = raw.parse::<i64>() {
        return Value::from(value);
    }
    if let Ok(value) = raw.parse::<f64>() {
        return Value::from(value);
    }
    if let Ok(value) = raw.parse::<bool>() {
        return Value::from(value);
    }
    Value::from(raw)
}

fn find_or_create_stock_mut<'a>(
    config: &'a mut StrategyDaemonConfig,
    code: &str,
) -> &'a mut ConfiguredStock {
    if let Some(index) = config.stocks.iter().position(|stock| stock.code == code) {
        return &mut config.stocks[index];
    }

    config.stocks.push(ConfiguredStock {
        code: code.to_string(),
        enabled: true,
        strategies: Vec::new(),
    });
    // push 之后 stocks 非空，按索引取最后一个 mut 引用，避免 expect/unwrap。
    let last_index = config.stocks.len() - 1;
    &mut config.stocks[last_index]
}

fn remove_instance(
    config: &mut StrategyDaemonConfig,
    id: &str,
) -> Option<(String, ConfiguredStrategyInstance)> {
    for stock in &mut config.stocks {
        if let Some(index) = stock.strategies.iter().position(|item| item.id == id) {
            return Some((stock.code.clone(), stock.strategies.remove(index)));
        }
    }
    None
}

fn find_instance<'a>(
    config: &'a StrategyDaemonConfig,
    id: &str,
) -> Option<(&'a str, &'a ConfiguredStrategyInstance)> {
    config.stocks.iter().find_map(|stock| {
        stock
            .strategies
            .iter()
            .find(|instance| instance.id == id)
            .map(|instance| (stock.code.as_str(), instance))
    })
}

fn drop_empty_stocks(config: &mut StrategyDaemonConfig) {
    config.stocks.retain(|stock| !stock.strategies.is_empty());
}

struct StrategyInstanceRow {
    id: String,
    name: String,
    code: String,
    enabled: bool,
    params: String,
}

fn list_instance_rows(config: &StrategyDaemonConfig) -> Vec<StrategyInstanceRow> {
    let mut rows: Vec<StrategyInstanceRow> = config
        .stocks
        .iter()
        .flat_map(|stock| {
            stock.strategies.iter().map(|instance| StrategyInstanceRow {
                id: instance.id.clone(),
                name: instance.name.clone(),
                code: stock.code.clone(),
                enabled: instance.enabled,
                params: format_params(&instance.params),
            })
        })
        .collect();
    rows.sort_by(|left, right| left.id.cmp(&right.id));
    rows
}

fn format_params(value: &Value) -> String {
    match value {
        Value::Object(map) if map.is_empty() => "{}".to_string(),
        Value::Object(map) => {
            let mut parts: Vec<String> = map
                .iter()
                .map(|(key, value)| format!("{key}={}", json_value_to_string(value)))
                .collect();
            parts.sort();
            parts.join(",")
        }
        other => json_value_to_string(other),
    }
}

fn json_value_to_string(value: &Value) -> String {
    match value {
        Value::String(inner) => inner.clone(),
        _ => value.to_string(),
    }
}
