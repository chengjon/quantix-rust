use std::collections::HashMap;

use serde_json::Value;

use crate::core::{QuantixError, Result};
use crate::strategy::ConfiguredStrategyInstance;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndicatorInstanceId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorSpec {
    name: String,
    params: HashMap<String, Value>,
    instance_id: IndicatorInstanceId,
}

impl IndicatorSpec {
    /// 用 name + params 构造 IndicatorSpec；instance_id 由 from_parts 自动推导（参数规范化排序）。
    pub fn new(name: impl Into<String>, params: HashMap<String, Value>) -> Self {
        let name = name.into();
        let instance_id = IndicatorInstanceId::from_parts(&name, &params);

        Self {
            name,
            params,
            instance_id,
        }
    }

    /// 返回指标名（只读）。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回参数 map（只读）。
    pub fn params(&self) -> &HashMap<String, Value> {
        &self.params
    }

    /// 返回由 name + params 推导出的稳定 instance_id，用作 cache key 一部分。
    pub fn instance_id(&self) -> &IndicatorInstanceId {
        &self.instance_id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorPipelineConfig {
    pub indicators: Vec<IndicatorSpec>,
}

impl IndicatorInstanceId {
    /// 用 name + params 生成规范化 instance_id：params 为空时退化为 `name`；否则追加按键排序后的 JSON 字符串 `name:{...}`。
    pub fn from_parts(name: &str, params: &HashMap<String, Value>) -> Self {
        if params.is_empty() {
            return Self(name.to_string());
        }

        let mut entries: Vec<(&String, &Value)> = params.iter().collect();
        entries.sort_by_key(|(left, _)| *left);

        let mut canonical = serde_json::Map::new();
        for (key, value) in entries {
            canonical.insert(key.clone(), value.clone());
        }

        let suffix = Value::Object(canonical).to_string();
        Self(format!("{name}:{suffix}"))
    }
}

impl TryFrom<&ConfiguredStrategyInstance> for IndicatorPipelineConfig {
    type Error = QuantixError;

    fn try_from(value: &ConfiguredStrategyInstance) -> Result<Self> {
        if value.name != "ma_cross" {
            return Err(QuantixError::Unsupported(format!(
                "indicator pipeline first slice only supports ma_cross, got {}",
                value.name
            )));
        }

        let params = value.params.as_object().ok_or_else(|| {
            QuantixError::Config("ma_cross params must be a JSON object".to_string())
        })?;

        let fast = read_usize_param(params, "fast")?;
        let slow = read_usize_param(params, "slow")?;

        Ok(Self {
            indicators: vec![sma_spec(fast), sma_spec(slow)],
        })
    }
}

fn sma_spec(period: usize) -> IndicatorSpec {
    let mut params = HashMap::new();
    params.insert("period".to_string(), Value::from(period));
    IndicatorSpec::new("sma", params)
}

fn read_usize_param(
    params: &serde_json::Map<String, Value>,
    key: &str,
) -> std::result::Result<usize, QuantixError> {
    let raw = params
        .get(key)
        .ok_or_else(|| QuantixError::Config(format!("missing ma_cross param `{key}`")))?;

    match raw {
        Value::Number(n) => n
            .as_u64()
            .ok_or_else(|| {
                QuantixError::Config(format!(
                    "ma_cross param `{key}` must be a non-negative integer"
                ))
            })
            .and_then(|value| {
                usize::try_from(value).map_err(|_| {
                    QuantixError::Config(format!(
                        "ma_cross param `{key}` is too large for this platform"
                    ))
                })
            }),
        _ => Err(QuantixError::Config(format!(
            "ma_cross param `{key}` must be an integer"
        ))),
    }
}
