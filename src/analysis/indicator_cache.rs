use std::collections::HashMap;

use crate::analysis::{IndicatorInstanceId, IndicatorSeries};
use crate::core::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndicatorCacheKey {
    pub dataset_fingerprint: String,
    pub instance_id: IndicatorInstanceId,
    pub range: (usize, usize),
}

impl IndicatorCacheKey {
    /// 用 dataset 指纹 + 指标实例 ID + 行范围构造 cache key；三者共同决定 cache 命中。
    pub fn new(
        dataset_fingerprint: impl Into<String>,
        instance_id: IndicatorInstanceId,
        range: (usize, usize),
    ) -> Self {
        Self {
            dataset_fingerprint: dataset_fingerprint.into(),
            instance_id,
            range,
        }
    }
}

#[derive(Debug, Default)]
pub struct IndicatorCache {
    entries: HashMap<IndicatorCacheKey, IndicatorSeries>,
}

impl IndicatorCache {
    /// 构造空 IndicatorCache（Default::default() 的别名）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 命中则返回 clone；未命中调用 compute 计算后写入并返回；compute 失败直接透传错误且不污染 cache。
    pub fn get_or_compute<F>(
        &mut self,
        key: IndicatorCacheKey,
        compute: F,
    ) -> Result<IndicatorSeries>
    where
        F: FnOnce() -> Result<IndicatorSeries>,
    {
        if let Some(cached) = self.entries.get(&key) {
            return Ok(cached.clone());
        }

        let computed = compute()?;
        self.entries.insert(key, computed.clone());
        Ok(computed)
    }

    /// 返回当前 cache 条目数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 判断 cache 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
