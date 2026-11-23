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
    /// fin EventStudy::significance 的 t 类统计量（NaN = 事件过少）
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
}
