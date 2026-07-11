use std::collections::HashMap;

use rust_decimal::Decimal;
use serde_json::Value;

use crate::analysis::indicator_config::IndicatorSpec;
use crate::analysis::indicators::{Kdj as KdjPoint, Macd as MacdPoint, ema, rsi, sma};
use crate::core::{QuantixError, Result};

/// 指标计算输出：可能为标量序列、MACD/KDJ 复合点序列，或 ATR 序列。
#[derive(Debug, Clone)]
pub enum IndicatorSeries {
    ScalarSeries(Vec<Option<Decimal>>),
    MacdSeries(Vec<Option<MacdPoint>>),
    KdjSeries(Vec<Option<KdjPoint>>),
    AtrSeries(Vec<Option<Decimal>>),
}

/// `IndicatorSeries` 的类型标签，供下游在不知道具体值时分支处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorSeriesKind {
    Scalar,
    Macd,
    Kdj,
    Atr,
}

/// 指标的静态元信息：规范名、回看周期、预热长度（warm-up）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndicatorMeta {
    pub canonical_name: &'static str,
    pub lookback: usize,
    pub warmup_len: usize,
}

/// 描述某个指标在某 period 下的可识别身份：元信息 + 序列类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndicatorDescriptor {
    pub meta: IndicatorMeta,
    pub series_kind: IndicatorSeriesKind,
}

/// First slice input is close-only. Later slices may extend this shape for ATR/KDJ-style indicators.
#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorInput {
    dataset_fingerprint: String,
    range: (usize, usize),
    close: Vec<Decimal>,
}

impl IndicatorInput {
    /// 用 close 序列构造，fingerprint 由 close 自动派生，range 覆盖整个序列。
    pub fn new(close: Vec<Decimal>) -> Self {
        let len = close.len();
        Self {
            dataset_fingerprint: derive_dataset_fingerprint(&close),
            range: (0, len),
            close,
        }
    }

    /// 与 `new` 类似，但使用显式 dataset_fingerprint（避免重复派生），range 仍覆盖整个序列。
    pub fn with_dataset_fingerprint(
        dataset_fingerprint: impl Into<String>,
        close: Vec<Decimal>,
    ) -> Self {
        let len = close.len();
        Self {
            dataset_fingerprint: dataset_fingerprint.into(),
            range: (0, len),
            close,
        }
    }

    /// 完整构造：自定义 fingerprint、序列范围 `[start, end)`、close 数据。
    pub fn with_context(
        dataset_fingerprint: impl Into<String>,
        range: (usize, usize),
        close: Vec<Decimal>,
    ) -> Self {
        Self {
            dataset_fingerprint: dataset_fingerprint.into(),
            range,
            close,
        }
    }

    /// 返回 close 序列的引用。
    pub fn close(&self) -> &[Decimal] {
        &self.close
    }

    /// 返回用于缓存命中的 dataset 指纹字符串。
    pub fn dataset_fingerprint(&self) -> &str {
        &self.dataset_fingerprint
    }

    /// 返回序列范围 `[start, end)`，下游用来定位结果在原始窗口中的位置。
    pub fn range(&self) -> (usize, usize) {
        self.range
    }
}

fn derive_dataset_fingerprint(close: &[Decimal]) -> String {
    let mut fingerprint = String::from("close:");
    for (idx, value) in close.iter().enumerate() {
        if idx > 0 {
            fingerprint.push(',');
        }
        fingerprint.push_str(&value.normalize().to_string());
    }
    fingerprint
}

type ComputeFn = fn(period: usize, input: &IndicatorInput) -> IndicatorSeries;
type MetaFn = fn(period: usize) -> IndicatorMeta;

#[derive(Clone, Copy)]
struct BuiltinIndicator {
    meta_fn: MetaFn,
    compute_fn: ComputeFn,
    series_kind: IndicatorSeriesKind,
}

/// 内置指标的注册表，目前固定包含 sma/ema/rsi。
pub struct IndicatorRegistry {
    builtins: HashMap<&'static str, BuiltinIndicator>,
}

impl Default for IndicatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl IndicatorRegistry {
    /// 创建内置指标注册表（sma/ema/rsi）。
    pub fn new() -> Self {
        let mut builtins = HashMap::new();
        builtins.insert(
            "sma",
            BuiltinIndicator {
                meta_fn: |period| IndicatorMeta {
                    canonical_name: "sma",
                    lookback: period,
                    warmup_len: period.saturating_sub(1),
                },
                compute_fn: |period, input| {
                    IndicatorSeries::ScalarSeries(sma(input.close(), period))
                },
                series_kind: IndicatorSeriesKind::Scalar,
            },
        );
        builtins.insert(
            "ema",
            BuiltinIndicator {
                meta_fn: |period| IndicatorMeta {
                    canonical_name: "ema",
                    lookback: period,
                    warmup_len: period.saturating_sub(1),
                },
                compute_fn: |period, input| {
                    IndicatorSeries::ScalarSeries(ema(input.close(), period))
                },
                series_kind: IndicatorSeriesKind::Scalar,
            },
        );
        builtins.insert(
            "rsi",
            BuiltinIndicator {
                meta_fn: |period| IndicatorMeta {
                    canonical_name: "rsi",
                    lookback: period.saturating_add(1),
                    warmup_len: period,
                },
                compute_fn: |period, input| {
                    IndicatorSeries::ScalarSeries(rsi(input.close(), period))
                },
                series_kind: IndicatorSeriesKind::Scalar,
            },
        );

        Self { builtins }
    }

    /// `new` 的别名（命名沿用旧 API），返回包含内置指标的注册表。
    pub fn register_builtin() -> Self {
        Self::new()
    }

    /// 返回 spec 对应的 descriptor（元信息 + 序列类型）；不支持的指标返回 `Unsupported` 错误。
    pub fn descriptor(&self, spec: &IndicatorSpec) -> Result<IndicatorDescriptor> {
        let (builtin, period) = self.resolve_builtin_and_period(spec)?;
        Ok(IndicatorDescriptor {
            meta: (builtin.meta_fn)(period),
            series_kind: builtin.series_kind,
        })
    }

    /// 在给定输入上执行 spec 描述的指标计算并返回结果序列；参数缺失/非法或指标不支持时返回错误。
    pub fn compute(&self, spec: &IndicatorSpec, input: &IndicatorInput) -> Result<IndicatorSeries> {
        let (builtin, period) = self.resolve_builtin_and_period(spec)?;
        Ok((builtin.compute_fn)(period, input))
    }

    fn resolve_builtin_and_period(
        &self,
        spec: &IndicatorSpec,
    ) -> Result<(BuiltinIndicator, usize)> {
        let normalized_name = spec.name().to_ascii_lowercase();
        let builtin = self
            .builtins
            .get(normalized_name.as_str())
            .copied()
            .ok_or_else(|| {
                QuantixError::Unsupported(format!(
                    "indicator registry first slice only supports sma/ema/rsi, got {}",
                    spec.name()
                ))
            })?;

        let min_period = if normalized_name == "rsi" { 2 } else { 1 };
        let period = parse_period(spec.params().get("period"), min_period, spec.name())?;
        Ok((builtin, period))
    }
}

fn parse_period(raw: Option<&Value>, min_period: usize, indicator_name: &str) -> Result<usize> {
    let raw =
        raw.ok_or_else(|| QuantixError::Config("indicator param `period` is required".into()))?;
    let period = match raw {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| {
                QuantixError::Config("indicator param `period` must be a positive integer".into())
            })
            .and_then(|value| {
                usize::try_from(value).map_err(|_| {
                    QuantixError::Config(
                        "indicator param `period` is too large for this platform".into(),
                    )
                })
            })?,
        _ => Err(QuantixError::Config(
            "indicator param `period` must be a positive integer".into(),
        ))?,
    };

    if period < min_period {
        return Err(QuantixError::Config(if min_period == 1 {
            "indicator param `period` must be greater than zero".into()
        } else {
            format!("indicator `{indicator_name}` requires `period` >= {min_period}")
        }));
    }

    Ok(period)
}
