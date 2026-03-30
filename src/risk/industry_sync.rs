use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::Row;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::core::{QuantixError, Result, UpstreamMySqlSettings};
use crate::risk::industry::{
    ClassificationStandard, IndustryResolver, ShenwanCurrentSeedRow, ShenwanHistoricalSeedRow,
    normalize_security_code,
};
use crate::risk::industry_store::SqliteIndustryStore;

const SHENWAN_CURRENT_SOURCE: &str = "mystocks.sw_industry_classification";
const SHENWAN_HISTORY_SOURCE: &str = "mystocks.sw_stock_update+sw_industry";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndustrySyncSummary {
    pub standard: ClassificationStandard,
    pub current_rows: usize,
    pub history_rows: usize,
    pub store_path: PathBuf,
    pub synced_at: DateTime<Utc>,
}

#[async_trait]
pub trait IndustrySyncSource: Send + Sync {
    async fn fetch_shenwan_current_rows(&self) -> Result<Vec<ShenwanCurrentSeedRow>>;

    async fn fetch_shenwan_history_rows(&self) -> Result<Vec<ShenwanHistoricalSeedRow>>;
}

#[derive(Debug, Clone)]
pub struct MySqlIndustrySyncSource {
    settings: UpstreamMySqlSettings,
}

impl MySqlIndustrySyncSource {
    pub fn new(settings: UpstreamMySqlSettings) -> Self {
        Self { settings }
    }

    async fn connect(&self) -> Result<sqlx::MySqlPool> {
        let dsn = format!(
            "{}/{}",
            self.settings.url.trim_end_matches('/'),
            self.settings.database
        );
        let options = MySqlConnectOptions::from_str(&dsn)
            .map_err(|err| QuantixError::Config(format!("upstream mysql URL 无效: {err}")))?
            .username(&self.settings.user)
            .password(&self.settings.password);

        MySqlPoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|err| QuantixError::DatabaseConnection(format!("upstream mysql 连接失败: {err}")))
    }
}

#[async_trait]
impl IndustrySyncSource for MySqlIndustrySyncSource {
    async fn fetch_shenwan_current_rows(&self) -> Result<Vec<ShenwanCurrentSeedRow>> {
        let pool = self.connect().await?;
        let rows = sqlx::query(
            r#"
SELECT `股票代码` AS security_code, `新版一级行业` AS industry_name
FROM `sw_industry_classification`
WHERE `股票代码` IS NOT NULL
  AND `新版一级行业` IS NOT NULL
  AND TRIM(`新版一级行业`) <> ''
ORDER BY `股票代码`, `id`
"#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|err| QuantixError::DatabaseQuery(format!("读取申万当前行业映射失败: {err}")))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let security_code: String = row.try_get("security_code")?;
            let industry_name: String = row.try_get("industry_name")?;
            let normalized_code = normalize_security_code(&security_code);
            let normalized_name = industry_name.trim().to_string();
            if normalized_code.is_empty() || normalized_name.is_empty() {
                continue;
            }

            result.push(ShenwanCurrentSeedRow {
                security_code: normalized_code,
                industry_name: normalized_name,
                source: SHENWAN_CURRENT_SOURCE.to_string(),
            });
        }

        Ok(result)
    }

    async fn fetch_shenwan_history_rows(&self) -> Result<Vec<ShenwanHistoricalSeedRow>> {
        let pool = self.connect().await?;
        let rows = sqlx::query(
            r#"
SELECT
    changes.`股票代码` AS security_code,
    changes.`计入日期` AS effective_from,
    dict.`一级行业名称` AS industry_name
FROM `sw_stock_update` AS changes
JOIN `sw_industry` AS dict
  ON dict.`行业代码` = changes.`行业代码`
WHERE changes.`股票代码` IS NOT NULL
  AND changes.`计入日期` IS NOT NULL
  AND dict.`一级行业名称` IS NOT NULL
  AND TRIM(dict.`一级行业名称`) <> ''
ORDER BY changes.`股票代码`, changes.`计入日期`, changes.`更新日期`, changes.`id`
"#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|err| QuantixError::DatabaseQuery(format!("读取申万历史行业映射失败: {err}")))?;

        let mut raw_rows = Vec::with_capacity(rows.len());
        for row in rows {
            let security_code: String = row.try_get("security_code")?;
            let effective_from: NaiveDate = row.try_get("effective_from")?;
            let industry_name: String = row.try_get("industry_name")?;
            let normalized_code = normalize_security_code(&security_code);
            let normalized_name = industry_name.trim().to_string();
            if normalized_code.is_empty() || normalized_name.is_empty() {
                continue;
            }

            raw_rows.push(RawHistoricalIndustryRow {
                security_code: normalized_code,
                industry_name: normalized_name,
                effective_from,
            });
        }

        Ok(build_history_rows(raw_rows))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawHistoricalIndustryRow {
    security_code: String,
    industry_name: String,
    effective_from: NaiveDate,
}

pub async fn sync_industry_reference_data_at<Source>(
    risk_state_path: impl AsRef<Path>,
    standard: ClassificationStandard,
    source: &Source,
    synced_at: DateTime<Utc>,
) -> Result<IndustrySyncSummary>
where
    Source: IndustrySyncSource,
{
    if standard != ClassificationStandard::Shenwan {
        return Err(QuantixError::Unsupported(
            "risk sync industry 目前仅支持 --standard shenwan".to_string(),
        ));
    }

    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path).await?;
    let resolver = IndustryResolver::new(store.clone());
    let current_rows = source.fetch_shenwan_current_rows().await?;
    let history_rows = source.fetch_shenwan_history_rows().await?;

    resolver
        .sync_shenwan_current_rows(&current_rows, synced_at)
        .await?;
    resolver
        .sync_shenwan_history_rows(&history_rows, synced_at)
        .await?;

    Ok(IndustrySyncSummary {
        standard,
        current_rows: current_rows.len(),
        history_rows: history_rows.len(),
        store_path: store.path().to_path_buf(),
        synced_at,
    })
}

fn build_history_rows(rows: Vec<RawHistoricalIndustryRow>) -> Vec<ShenwanHistoricalSeedRow> {
    let mut deduped: Vec<RawHistoricalIndustryRow> = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(last) = deduped.last_mut() {
            if last.security_code == row.security_code && last.effective_from == row.effective_from {
                last.industry_name = row.industry_name;
                continue;
            }
        }

        deduped.push(row);
    }

    let mut result = Vec::with_capacity(deduped.len());
    for (index, row) in deduped.iter().enumerate() {
        let effective_to = deduped
            .get(index + 1)
            .filter(|next| next.security_code == row.security_code)
            .and_then(|next| next.effective_from.pred_opt().or(Some(next.effective_from)));

        result.push(ShenwanHistoricalSeedRow {
            security_code: row.security_code.clone(),
            industry_name: row.industry_name.clone(),
            effective_from: row.effective_from,
            effective_to,
            source: SHENWAN_HISTORY_SOURCE.to_string(),
        });
    }

    result
}
