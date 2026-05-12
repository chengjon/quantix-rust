use crate::core::{QuantixError, Result};
use crate::factor::alpha101::{
    alpha101_002, alpha101_003, alpha101_005, alpha101_006, alpha101_012,
};
use crate::factor::alpha191::{alpha191_101, alpha191_102, alpha191_103};
use crate::factor::dataset::FactorDataset;
use crate::factor::operators::{cs_rank, ts_delay, ts_delta};
use crate::factor::types::{FactorCategory, FactorComputeResult, FactorMeta, MissingPolicy};

pub struct FactorCatalog {
    metas: Vec<FactorMeta>,
}

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
    pub fn list(&self) -> &[FactorMeta] {
        &self.metas
    }

    pub fn compute(&self, factor_id: &str, dataset: &FactorDataset) -> Result<FactorComputeResult> {
        let values = match factor_id {
            "rank_close" => cs_rank(dataset.frame(), "close"),
            "delay_close_1" => ts_delay(dataset.frame(), "close", 1),
            "delta_close_1" => ts_delta(dataset.frame(), "close", 1),
            "alpha101_002" => alpha101_002(dataset.frame()),
            "alpha101_003" => alpha101_003(dataset.frame()),
            "alpha101_005" => alpha101_005(dataset.frame()),
            "alpha101_006" => alpha101_006(dataset.frame()),
            "alpha101_012" => alpha101_012(dataset.frame()),
            "alpha191_101" => alpha191_101(dataset.frame()),
            "alpha191_102" => alpha191_102(dataset.frame()),
            "alpha191_103" => alpha191_103(dataset.frame()),
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
        frame.with_column(values).map_err(|e| {
            QuantixError::DataParse(format!("factor output value attach failed: {}", e))
        })?;

        Ok(FactorComputeResult {
            factor_id: factor_id.to_string(),
            frame,
        })
    }
}
