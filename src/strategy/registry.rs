use serde_json::Value;

use crate::analysis::{
    IndicatorInput, IndicatorInstanceId, IndicatorPipeline, IndicatorPipelineConfig,
    IndicatorSeries,
};
use crate::core::signal::Signal;
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::execution::models::SignalEnvelope;
use crate::strategy::ConfiguredStrategyInstance;

/// 已配置策略评估器 trait：lookback_required 返回所需 K 线回看长度、evaluate 在给定 K 线上求值并返回 SignalEnvelope。Send + Sync 以适配 daemon。
pub trait ConfiguredStrategyEvaluator: Send + Sync {
    fn lookback_required(&self) -> usize;
    fn evaluate(&self, klines: &[Kline]) -> Result<SignalEnvelope>;
}

/// 策略注册中心：根据 ConfiguredStrategyInstance.name 路由到对应 Evaluator 实现（当前仅 ma_cross）。
#[derive(Debug, Default, Clone)]
pub struct StrategyRegistry;

impl StrategyRegistry {
    /// 创建空注册中心（无状态，仅做路由表入口）。
    pub fn new() -> Self {
        Self
    }

    /// 按 config.name 构造对应策略的 Evaluator 装箱返回；未知策略名返回错误。当前支持 "ma_cross"。
    pub fn build(
        &self,
        config: &ConfiguredStrategyInstance,
    ) -> Result<Box<dyn ConfiguredStrategyEvaluator>> {
        match config.name.as_str() {
            "ma_cross" => Ok(Box::new(MaCrossEvaluator::from_config(config)?)),
            other => Err(QuantixError::Other(format!("未知策略: {other}"))),
        }
    }
}

struct MaCrossEvaluator {
    fast_id: IndicatorInstanceId,
    slow_id: IndicatorInstanceId,
    lookback: usize,
    pipeline_config: IndicatorPipelineConfig,
}

impl MaCrossEvaluator {
    fn from_config(config: &ConfiguredStrategyInstance) -> Result<Self> {
        let pipeline_config = IndicatorPipelineConfig::try_from(config)?;
        let [fast_spec, slow_spec] = pipeline_config.indicators.as_slice() else {
            return Err(QuantixError::Config(format!(
                "strategy {} indicator pipeline expected two sma indicators",
                config.id
            )));
        };

        if fast_spec.name() != "sma" || slow_spec.name() != "sma" {
            return Err(QuantixError::Config(format!(
                "strategy {} indicator pipeline expected two sma indicators",
                config.id
            )));
        }

        let fast = read_period(fast_spec.params().get("period")).map_err(|err| {
            QuantixError::Other(format!(
                "strategy {} 缺少或无效的 fast 参数: {err}",
                config.id
            ))
        })?;
        let slow = read_period(slow_spec.params().get("period")).map_err(|err| {
            QuantixError::Other(format!(
                "strategy {} 缺少或无效的 slow 参数: {err}",
                config.id
            ))
        })?;

        if fast == 0 || slow == 0 || fast >= slow {
            return Err(QuantixError::Other(format!(
                "strategy {} 的 ma_cross 参数非法: fast={fast}, slow={slow}",
                config.id
            )));
        }

        Ok(Self {
            fast_id: fast_spec.instance_id().clone(),
            slow_id: slow_spec.instance_id().clone(),
            lookback: slow,
            pipeline_config,
        })
    }
}

impl ConfiguredStrategyEvaluator for MaCrossEvaluator {
    fn lookback_required(&self) -> usize {
        self.lookback
    }

    fn evaluate(&self, klines: &[Kline]) -> Result<SignalEnvelope> {
        if klines.len() < self.lookback {
            return Ok(SignalEnvelope::new(Signal::Hold));
        }

        let closes: Vec<_> = klines.iter().map(|bar| bar.close).collect();
        let input = IndicatorInput::new(closes);
        let mut pipeline = IndicatorPipeline::with_builtin_registry();
        let output = pipeline.run(&self.pipeline_config, &input)?;
        let short_mas = scalar_series(&output, &self.fast_id)?;
        let long_mas = scalar_series(&output, &self.slow_id)?;

        if short_mas.len() != long_mas.len() {
            return Err(QuantixError::Other(format!(
                "ma_cross indicator series length mismatch: short={}, long={}",
                short_mas.len(),
                long_mas.len()
            )));
        }

        let mut last_short = None;
        let mut last_long = None;
        let mut latest_signal = Signal::Hold;

        for idx in 0..short_mas.len() {
            let Some(curr_short) = short_mas[idx] else {
                continue;
            };
            let Some(curr_long) = long_mas[idx] else {
                continue;
            };

            if let (Some(prev_short), Some(prev_long)) = (last_short, last_long) {
                if prev_short <= prev_long && curr_short > curr_long {
                    latest_signal = Signal::Buy;
                } else if prev_short >= prev_long && curr_short < curr_long {
                    latest_signal = Signal::Sell;
                } else {
                    latest_signal = Signal::Hold;
                }
            }

            last_short = Some(curr_short);
            last_long = Some(curr_long);
        }

        Ok(SignalEnvelope::new(latest_signal))
    }
}

fn read_period(raw: Option<&Value>) -> std::result::Result<usize, &'static str> {
    match raw {
        Some(Value::Number(number)) if number.is_u64() => number
            .as_u64()
            .ok_or("period must be a non-negative integer")
            .and_then(|value| {
                usize::try_from(value).map_err(|_| "period is too large for this platform")
            }),
        Some(Value::Number(_)) => Err("period must be a non-negative integer"),
        _ => Err("period must be an integer"),
    }
}

fn scalar_series<'a>(
    output: &'a std::collections::HashMap<IndicatorInstanceId, IndicatorSeries>,
    id: &IndicatorInstanceId,
) -> Result<&'a [Option<rust_decimal::Decimal>]> {
    let series = output.get(id).ok_or_else(|| {
        QuantixError::Other(format!("ma_cross indicator output missing `{}`", id.0))
    })?;

    match series {
        IndicatorSeries::ScalarSeries(values) => Ok(values.as_slice()),
        _ => Err(QuantixError::Other(format!(
            "ma_cross indicator `{}` must produce a scalar series",
            id.0
        ))),
    }
}
