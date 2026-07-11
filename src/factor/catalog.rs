use crate::core::{QuantixError, Result};
use crate::factor::alpha101::{
    alpha101_002, alpha101_003, alpha101_005, alpha101_006, alpha101_012,
};
use crate::factor::alpha191::{
    alpha191_101, alpha191_102, alpha191_103, alpha191_104, alpha191_105, alpha191_106,
    alpha191_107, alpha191_108, alpha191_109, alpha191_110, alpha191_111, alpha191_112,
    alpha191_113, alpha191_114, alpha191_115, alpha191_116, alpha191_117, alpha191_118,
    alpha191_119, alpha191_120,
};
use crate::factor::dataset::FactorDataset;
use crate::factor::operators::{cs_rank, ts_delay, ts_delta, ts_rank};
use crate::factor::types::{FactorCategory, FactorComputeResult, FactorMeta, MissingPolicy};

/// 因子目录：持有 `Vec<FactorMeta>`，按 id 派发 compute；通常由 `builtin_factor_catalog()` 构造内置集合。
pub struct FactorCatalog {
    metas: Vec<FactorMeta>,
}

/// 构造内置因子目录：包含 rank_close / delay_close_1 / delta_close_1 / ts_rank_close_5、Alpha101（#002/#003/#005/#006/#012）与 Alpha191（#101~#120）共 30 项因子元数据，供 FactorCatalog::compute 按因子 id 派发。
pub fn builtin_factor_catalog() -> FactorCatalog {
    FactorCatalog {
        metas: vec![
            FactorMeta {
                id: "rank_close".to_string(),
                category: FactorCategory::Technical,
                description: "Cross-sectional rank of close within each date".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "delay_close_1".to_string(),
                category: FactorCategory::Technical,
                description: "One-bar per-symbol delayed close".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "delta_close_1".to_string(),
                category: FactorCategory::Technical,
                description: "One-bar per-symbol close delta".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            FactorMeta {
                id: "ts_rank_close_5".to_string(),
                category: FactorCategory::Technical,
                description: "Five-bar per-symbol time-series rank of close".to_string(),
                author: Some("quantix".to_string()),
                source: Some("p1".to_string()),
                refresh_frequency: Some("daily".to_string()),
                required_fields: vec!["close".to_string()],
                missing_policy: MissingPolicy::KeepNull,
            },
            alpha101_meta(
                "alpha101_002",
                "Alpha101 #2: negative rolling correlation of ranked volume delta and ranked intraday return",
                vec!["open", "close", "volume"],
            ),
            alpha101_meta(
                "alpha101_003",
                "Alpha101 #3: negative rolling correlation of ranked open and ranked volume",
                vec!["open", "volume"],
            ),
            alpha101_meta(
                "alpha101_005",
                "Alpha101 #5: ranked open-vwap mean spread times negative absolute ranked close-vwap spread",
                vec!["open", "close", "volume", "amount"],
            ),
            alpha101_meta(
                "alpha101_006",
                "Alpha101 #6: negative rolling correlation of open and volume",
                vec!["open", "volume"],
            ),
            alpha101_meta(
                "alpha101_012",
                "Alpha101 #12: signed volume delta times negative close delta",
                vec!["close", "volume"],
            ),
            alpha191_meta(
                "alpha191_101",
                "Alpha191 #101: intraday close-open position within high-low range",
                vec!["open", "high", "low", "close"],
            ),
            alpha191_meta(
                "alpha191_102",
                "Alpha191 #102: negative product of cross-sectional price-change rank and volume rank",
                vec!["open", "close", "volume"],
            ),
            alpha191_meta(
                "alpha191_103",
                "Alpha191 #103: intraday close-open position times volume",
                vec!["open", "high", "low", "close", "volume"],
            ),
            alpha191_meta(
                "alpha191_104",
                "Alpha191 #104: cross-sectional rank of 10-day close-volume correlation",
                vec!["close", "volume"],
            ),
            alpha191_meta(
                "alpha191_105",
                "Alpha191 #105: negative 5-day correlation of ranked high and ranked volume",
                vec!["high", "volume"],
            ),
            alpha191_meta(
                "alpha191_106",
                "Alpha191 #106: negative 10-day time-series rank of absolute close-open change",
                vec!["open", "close"],
            ),
            alpha191_meta(
                "alpha191_107",
                "Alpha191 #107: cross-sectional rank of intraday move over delayed-close gap",
                vec!["open", "close"],
            ),
            alpha191_meta(
                "alpha191_108",
                "Alpha191 #108: intraday close-open position times volume",
                vec!["open", "high", "low", "close", "volume"],
            ),
            alpha191_meta(
                "alpha191_109",
                "Alpha191 #109: negative 5-day close delta",
                vec!["close"],
            ),
            alpha191_meta(
                "alpha191_110",
                "Alpha191 #110: cross-sectional rank of low minus delayed close",
                vec!["low", "close"],
            ),
            alpha191_meta(
                "alpha191_111",
                "Alpha191 #111: cross-sectional rank of return over delayed-close gap ratio",
                vec!["open", "close"],
            ),
            alpha191_meta(
                "alpha191_112",
                "Alpha191 #112: negative product of close delta rank and volume rank",
                vec!["close", "volume"],
            ),
            alpha191_meta(
                "alpha191_113",
                "Alpha191 #113: negative 10-day correlation of ranked open and ranked volume",
                vec!["open", "volume"],
            ),
            alpha191_meta(
                "alpha191_114",
                "Alpha191 #114: cross-sectional rank of intraday close-open position",
                vec!["open", "high", "low", "close"],
            ),
            alpha191_meta(
                "alpha191_115",
                "Alpha191 #115: negative 7-day close delta",
                vec!["close"],
            ),
            alpha191_meta(
                "alpha191_116",
                "Alpha191 #116: negative 20-day time-series rank of absolute close change",
                vec!["close"],
            ),
            alpha191_meta(
                "alpha191_117",
                "Alpha191 #117: intraday close-open position times volume",
                vec!["open", "high", "low", "close", "volume"],
            ),
            alpha191_meta(
                "alpha191_118",
                "Alpha191 #118: cross-sectional rank of 5-day close-volume correlation",
                vec!["close", "volume"],
            ),
            alpha191_meta(
                "alpha191_119",
                "Alpha191 #119: negative 3-day close delta",
                vec!["close"],
            ),
            alpha191_meta(
                "alpha191_120",
                "Alpha191 #120: negative cross-sectional rank of 10-day close stddev",
                vec!["close"],
            ),
        ],
    }
}

fn alpha101_meta(id: &str, description: &str, required_fields: Vec<&str>) -> FactorMeta {
    FactorMeta {
        id: id.to_string(),
        category: FactorCategory::Composite,
        description: description.to_string(),
        author: Some("quantix".to_string()),
        source: Some("alpha101".to_string()),
        refresh_frequency: Some("daily".to_string()),
        required_fields: required_fields
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        missing_policy: MissingPolicy::KeepNull,
    }
}

fn alpha191_meta(id: &str, description: &str, required_fields: Vec<&str>) -> FactorMeta {
    FactorMeta {
        id: id.to_string(),
        category: FactorCategory::Composite,
        description: description.to_string(),
        author: Some("quantix".to_string()),
        source: Some("alpha191".to_string()),
        refresh_frequency: Some("daily".to_string()),
        required_fields: required_fields
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        missing_policy: MissingPolicy::KeepNull,
    }
}

impl FactorCatalog {
    /// 返回目录中所有因子元数据的切片（按注册顺序）；调用方只读访问，不可修改 metas。
    pub fn list(&self) -> &[FactorMeta] {
        &self.metas
    }

    /// 按 factor_id 在 dataset.frame() 上派发计算：内置因子（rank_*/delay_*/delta_*/ts_rank_*/alpha101_*/alpha191_*）映射到对应算子或 alpha 函数，返回包含 date/symbol/value 列的 FactorComputeResult。未知 factor_id 返回 QuantixError::Config；底层 polars 错误透传为 DataParse。
    pub fn compute(&self, factor_id: &str, dataset: &FactorDataset) -> Result<FactorComputeResult> {
        let values = match factor_id {
            "rank_close" => cs_rank(dataset.frame(), "close"),
            "delay_close_1" => ts_delay(dataset.frame(), "close", 1),
            "delta_close_1" => ts_delta(dataset.frame(), "close", 1),
            "ts_rank_close_5" => ts_rank(dataset.frame(), "close", 5),
            "alpha101_002" => alpha101_002(dataset.frame()),
            "alpha101_003" => alpha101_003(dataset.frame()),
            "alpha101_005" => alpha101_005(dataset.frame()),
            "alpha101_006" => alpha101_006(dataset.frame()),
            "alpha101_012" => alpha101_012(dataset.frame()),
            "alpha191_101" => alpha191_101(dataset.frame()),
            "alpha191_102" => alpha191_102(dataset.frame()),
            "alpha191_103" => alpha191_103(dataset.frame()),
            "alpha191_104" => alpha191_104(dataset.frame()),
            "alpha191_105" => alpha191_105(dataset.frame()),
            "alpha191_106" => alpha191_106(dataset.frame()),
            "alpha191_107" => alpha191_107(dataset.frame()),
            "alpha191_108" => alpha191_108(dataset.frame()),
            "alpha191_109" => alpha191_109(dataset.frame()),
            "alpha191_110" => alpha191_110(dataset.frame()),
            "alpha191_111" => alpha191_111(dataset.frame()),
            "alpha191_112" => alpha191_112(dataset.frame()),
            "alpha191_113" => alpha191_113(dataset.frame()),
            "alpha191_114" => alpha191_114(dataset.frame()),
            "alpha191_115" => alpha191_115(dataset.frame()),
            "alpha191_116" => alpha191_116(dataset.frame()),
            "alpha191_117" => alpha191_117(dataset.frame()),
            "alpha191_118" => alpha191_118(dataset.frame()),
            "alpha191_119" => alpha191_119(dataset.frame()),
            "alpha191_120" => alpha191_120(dataset.frame()),
            other => {
                return Err(QuantixError::Unsupported(format!(
                    "unknown factor `{}`",
                    other
                )));
            }
        }
        .map_err(|e| {
            QuantixError::DataParse(format!("factor `{}` compute failed: {}", factor_id, e))
        })?;

        let mut frame = dataset
            .frame()
            .select(["date", "symbol"])
            .map_err(|e| QuantixError::DataParse(format!("factor output select failed: {}", e)))?;
        let mut values = values.clone();
        values.rename("value".into());
        frame.with_column(values.into()).map_err(|e| {
            QuantixError::DataParse(format!("factor output value attach failed: {}", e))
        })?;

        Ok(FactorComputeResult {
            factor_id: factor_id.to_string(),
            frame,
        })
    }
}
