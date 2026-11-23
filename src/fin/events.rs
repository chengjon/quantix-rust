//! 事件研究门面（库层，无 CLI，spec §2 非目标）。
//! 数据源约定（一期）：CSV/数据库事件表（code+date+类型）；news/ 接入二期。

use std::collections::HashMap;

use chrono::NaiveDate;
use fin_primitives::events::{EventStudy, EventWindow, MarketEvent};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};

/// 研究侧事件描述（quantix 语义，内部转 fin `MarketEvent`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSpec {
    pub code: String,
    pub date: NaiveDate,
    pub event_type: String,
    pub description: String,
}

/// 单事件研究输出（镜像 fin `EventResult` 摘要字段，不含逐日 AR 序列；
/// fin EventResult 无 serde，门面自行 derive）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStudyOutput {
    pub car_pre: f64,
    pub car_post: f64,
    pub peak_day: i32,
    pub trough_day: i32,
}

/// 批量事件研究输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatchOutput {
    pub studies: Vec<EventStudyOutput>,
    /// fin EventStudy::significance 的 t 类统计量
    /// （NaN = 0 个有效事件；n=1 时上游短路返回 0.0，events/mod.rs:172）
    pub significance: f64,
    /// 缺少序列而被跳过的 spec 数
    pub skipped: usize,
}

/// Kline 收盘序列 → fin 研究口径 f64 序列（date → UTC unix 秒）。
pub fn close_series(klines: &[crate::data::models::Kline]) -> Result<Vec<(u64, f64)>> {
    klines
        .iter()
        .map(|k| {
            // `NaiveDate::and_hms_opt` 返回 Option<NaiveDateTime>，先 ok_or 再 .and_utc()
            let ts = k
                .date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {}", k.date)))?
                .and_utc()
                .timestamp();
            let close = k.close.to_f64().ok_or_else(|| {
                QuantixError::DataParse(format!("close 转 f64 失败: {}", k.close))
            })?;
            // 1970-01-01 前日期 timestamp 为负，`as u64` 会回绕——A股数据域（1990+）不可达
            Ok((ts as u64, close))
        })
        .collect()
}

/// 批量事件研究。`series_by_code`: code → 收盘序列（调用方可经 `close_series` 构造）。
/// 窗口符号按上游约定（events/mod.rs:29-33）：`pre_days` 为负（如 -2 = 事件前 2 日）、
/// `post_days` 为正（如 +2），闭区间 `pre..=post` 逐日偏移。
pub fn run_event_studies(
    specs: &[EventSpec],
    series_by_code: &HashMap<String, Vec<(u64, f64)>>,
    benchmark: &[(u64, f64)],
    pre_days: i32,
    post_days: i32,
) -> Result<EventBatchOutput> {
    let mut studies = Vec::new();
    let mut results: Vec<fin_primitives::events::EventResult> = Vec::new();
    let mut skipped = 0usize;
    for spec in specs {
        let Some(series) = series_by_code.get(&spec.code) else {
            skipped += 1;
            continue;
        };
        let event_date = spec
            .date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {}", spec.date)))?
            // 同上：`as u64` 回绕仅在 1970 前日期出现，A股数据域不可达
            .and_utc()
            .timestamp() as u64;
        let event = MarketEvent {
            event_id: format!("{}:{}", spec.code, spec.date.format("%Y-%m-%d")),
            event_date,
            event_type: spec.event_type.clone(),
            description: spec.description.clone(),
        };
        let result = EventStudy::compute(
            &event,
            series,
            benchmark,
            EventWindow {
                pre_days,
                post_days,
            },
        );
        // 保留 fin EventResult 供真实 significance 统计（类型不出门面）
        studies.push(EventStudyOutput {
            car_pre: result.car_pre,
            car_post: result.car_post,
            peak_day: result.peak_day,
            trough_day: result.trough_day,
        });
        results.push(result);
    }
    // 真实调用 fin EventStudy::significance（t 类统计量，上游语义）；事件集为空 → NaN
    let significance = if results.is_empty() {
        f64::NAN
    } else {
        EventStudy::significance(&results)
    };
    Ok(EventBatchOutput {
        studies,
        significance,
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use crate::data::models::{AdjustType, Kline};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    use super::*;

    fn kline(day: u32, close: i64) -> Kline {
        let c = Decimal::from(close);
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: c,
            low: c,
            close: c,
            volume: 100,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    #[test]
    fn batch_skips_missing_codes_and_returns_studies() {
        let klines: Vec<Kline> = (1..=10u32).map(|d| kline(d, 10 + d as i64)).collect();
        let mut series = HashMap::new();
        series.insert("600000".to_string(), close_series(&klines).unwrap());
        let specs = vec![
            EventSpec {
                code: "600000".into(),
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                event_type: "announcement".into(),
                description: "test".into(),
            },
            EventSpec {
                code: "000001".into(),
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                event_type: "announcement".into(),
                description: "missing".into(),
            },
        ];
        let benchmark: Vec<(u64, f64)> = series["600000"].clone();
        let out = run_event_studies(&specs, &series, &benchmark, -2, 2).unwrap();
        assert_eq!(out.studies.len(), 1);
        assert_eq!(out.skipped, 1);
        assert!(out.significance.is_finite() || out.significance.is_nan());
    }

    /// CAR 精确哨兵（benchmark≠security，修复「全 AR≡0 路径零捕获力」）：
    /// security 事件前平坦 10.0、事件后次日跳 11.0；benchmark 恒 100.0（log return≡0）。
    /// 上游 daily_log_returns 为对数收益（(p_curr/p_prev).ln()，events/mod.rs:215），
    /// post 窗口恰含跳变次日 → car_post = ln(11/10)；平坦段 AR≡0 → car_pre = 0；
    /// CAR 序列 [0,0,0,ln1.1,ln1.1] → peak_day=+1（首个严格更大者）、
    /// trough_day=0（fold 初值日 car=0，无严格更低者，平局保先）。
    #[test]
    fn car_exact_sentinel_with_benchmark_spread() {
        let mut closes = vec![10i64; 5];
        closes.extend(vec![11i64; 5]); // 事件日 d5，d6 起跳变
        let klines: Vec<Kline> = closes
            .iter()
            .enumerate()
            .map(|(i, c)| kline((i + 1) as u32, *c))
            .collect();
        let security = close_series(&klines).unwrap();
        let benchmark: Vec<(u64, f64)> = security.iter().map(|&(ts, _)| (ts, 100.0)).collect();
        let mut series = HashMap::new();
        series.insert("600000".to_string(), security);
        let specs = vec![EventSpec {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            event_type: "announcement".into(),
            description: "sentinel".into(),
        }];
        let out = run_event_studies(&specs, &series, &benchmark, -2, 2).unwrap();
        assert_eq!(out.studies.len(), 1);
        let s = &out.studies[0];
        assert!(
            (s.car_pre - 0.0).abs() < 1e-12,
            "事件前平坦→car_pre 应为 0: {}",
            s.car_pre
        );
        assert!(
            (s.car_post - 0.0953101798043249).abs() < 1e-9,
            "car_post 应为 ln(11/10)（上游对数收益口径）: {}",
            s.car_post
        );
        assert_eq!(s.peak_day, 1, "CAR 峰值日 = 跳变次日（首个严格更大者）");
        assert_eq!(s.trough_day, 0, "谷日 = 初值日 0（无严格更低 CAR）");
    }

    /// 边界：空 specs → studies 空、significance = NaN（0 个有效事件）。
    #[test]
    fn empty_specs_yield_nan_significance() {
        let out = run_event_studies(&[], &HashMap::new(), &[], -2, 2).unwrap();
        assert!(out.studies.is_empty());
        assert_eq!(out.skipped, 0);
        assert!(out.significance.is_nan());
    }

    /// 边界：单事件 → significance = 0.0（上游 n<2 短路返回 0.0，events/mod.rs:172）。
    #[test]
    fn single_event_significance_is_zero() {
        let klines: Vec<Kline> = (1..=10u32).map(|d| kline(d, 10)).collect();
        let security = close_series(&klines).unwrap();
        let benchmark: Vec<(u64, f64)> = security.iter().map(|&(ts, _)| (ts, 100.0)).collect();
        let mut series = HashMap::new();
        series.insert("600000".to_string(), security);
        let specs = vec![EventSpec {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            event_type: "announcement".into(),
            description: "single".into(),
        }];
        let out = run_event_studies(&specs, &series, &benchmark, -2, 2).unwrap();
        assert_eq!(out.studies.len(), 1);
        assert_eq!(out.significance, 0.0);
    }
}
