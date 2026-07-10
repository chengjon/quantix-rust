use polars::prelude::*;

use crate::core::Result;
use crate::factor::loader::FactorDataLoader;
use crate::factor::types::FactorLoadRequest;

/// 因子计算数据集：持有规范化后的 DataFrame（symbol/date/字段列），是 FactorCatalog::compute 的输入。
#[derive(Debug, Clone)]
pub struct FactorDataset {
    frame: DataFrame,
}

impl FactorDataset {
    /// 通过 loader 加载 bars，构造 dataset 并校验 request.required_fields 都存在；任一校验失败返回错误。
    pub async fn from_loader<L>(loader: &L, request: &FactorLoadRequest) -> Result<Self>
    where
        L: FactorDataLoader + ?Sized,
    {
        let frame = loader.load_bars(request).await?;
        let dataset = Self::new(frame)?;
        dataset.ensure_required_columns(&request.required_fields)?;
        Ok(dataset)
    }

    /// 用已就绪的 DataFrame 构造 dataset：规范化列名、空字段校验通过、并强制 (symbol,date) 时间对齐。
    pub fn new(frame: DataFrame) -> Result<Self> {
        let frame = crate::factor::check::normalize_factor_frame(frame)?;
        let dataset = Self { frame };
        dataset.ensure_required_columns(&[])?;
        dataset.ensure_time_aligned()?;
        Ok(dataset)
    }

    /// 返回底层 DataFrame 的只读引用，供调用方做进一步计算。
    pub fn frame(&self) -> &DataFrame {
        &self.frame
    }

    /// 校验 frame 是否包含指定字段；缺失任一字段返回错误（空切片直接通过）。
    pub fn ensure_required_columns(&self, fields: &[String]) -> Result<()> {
        crate::factor::check::ensure_required_columns(&self.frame, fields)
    }

    /// 校验 frame 按 (symbol,date) 升序且无重复组合；不满足返回错误。
    pub fn ensure_time_aligned(&self) -> Result<()> {
        crate::factor::check::ensure_symbol_date_sorted(&self.frame)?;
        crate::factor::check::ensure_unique_symbol_date(&self.frame)
    }

    /// 基础未来函数检查（不替代因子内部严格校验）；调用方应在做横截面/时序计算前调用一次。
    pub fn validate_no_lookahead_basic(&self) -> Result<()> {
        crate::factor::check::validate_no_lookahead_basic(&self.frame)
    }
}
