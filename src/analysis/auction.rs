/// 竞价分析模块
///
/// 实现抢筹强度评分、封单金额计算、板块统计等功能

use crate::sources::auction_collector::AuctionQuote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 强度评分等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrengthLevel {
    /// 极强 (90-100)
    Extreme,
    /// 强 (70-89)
    High,
    /// 中等 (50-69)
    Medium,
    /// 弱 (30-49)
    Low,
    /// 极弱 (0-29)
    VeryLow,
}

impl StrengthLevel {
    /// 从评分获取等级
    pub fn from_score(score: f32) -> Self {
        if score >= 90.0 {
            StrengthLevel::Extreme
        } else if score >= 70.0 {
            StrengthLevel::High
        } else if score >= 50.0 {
            StrengthLevel::Medium
        } else if score >= 30.0 {
            StrengthLevel::Low
        } else {
            StrengthLevel::VeryLow
        }
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            StrengthLevel::Extreme => "极强",
            StrengthLevel::High => "强",
            StrengthLevel::Medium => "中等",
            StrengthLevel::Low => "弱",
            StrengthLevel::VeryLow => "极弱",
        }
    }
}

/// 竞价分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionAnalysis {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 当前价
    pub price: f64,
    /// 涨跌幅(%)
    pub change_percent: f64,
    /// 强度评分
    pub strength_score: f32,
    /// 强度等级
    pub strength_level: String,
    /// 买封金额（元）
    pub sealed_amount_buy: f64,
    /// 卖封金额（元）
    pub sealed_amount_sell: f64,
    /// 买封占比
    pub buy_ratio: f32,
    /// 成交量（手）
    pub volume: u64,
    /// 是否推荐买入
    pub is_recommended: bool,
}

/// 板块统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorStats {
    /// 板块名称
    pub sector_name: String,
    /// 股票数量
    pub stock_count: usize,
    /// 平均强度评分
    pub avg_strength_score: f32,
    /// 平均涨跌幅
    pub avg_change_percent: f64,
    /// 总买封金额（元）
    pub total_sealed_buy: f64,
    /// 总卖封金额（元）
    pub total_sealed_sell: f64,
    /// 强势股票数量（评分>=70）
    pub strong_count: usize,
    /// 推荐买入股票数量
    pub recommended_count: usize,
}

/// 竞价分析器
pub struct AuctionAnalyzer {
    /// 最低推荐评分阈值
    min_recommend_score: f32,
    /// 最小买封金额阈值（元）
    min_sealed_amount: f64,
    /// 最大涨幅阈值（%）
    max_change_percent: f64,
}

impl AuctionAnalyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            min_recommend_score: 70.0,
            min_sealed_amount: 500_000.0,  // 50万元
            max_change_percent: 8.0,         // 最多涨8%
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(
        min_recommend_score: f32,
        min_sealed_amount: f64,
        max_change_percent: f64,
    ) -> Self {
        Self {
            min_recommend_score,
            min_sealed_amount,
            max_change_percent,
        }
    }

    /// 分析单条竞价数据
    pub fn analyze_quote(&self, quote: &AuctionQuote) -> AuctionAnalysis {
        let strength_level = StrengthLevel::from_score(quote.strength_score);

        // 计算买封占比
        let total_sealed = quote.sealed_amount_buy + quote.sealed_amount_sell;
        let buy_ratio = if total_sealed > 0.0 {
            (quote.sealed_amount_buy / total_sealed) as f32
        } else {
            0.5
        };

        // 判断是否推荐买入
        let is_recommended = quote.strength_score >= self.min_recommend_score
            && quote.sealed_amount_buy >= self.min_sealed_amount
            && quote.change_percent <= self.max_change_percent
            && buy_ratio > 0.6;

        AuctionAnalysis {
            code: quote.code.clone(),
            name: quote.name.clone(),
            price: quote.price,
            change_percent: quote.change_percent,
            strength_score: quote.strength_score,
            strength_level: strength_level.display_name().to_string(),
            sealed_amount_buy: quote.sealed_amount_buy,
            sealed_amount_sell: quote.sealed_amount_sell,
            buy_ratio,
            volume: quote.volume,
            is_recommended,
        }
    }

    /// 批量分析竞价数据
    pub fn analyze_quotes(&self, quotes: &[AuctionQuote]) -> Vec<AuctionAnalysis> {
        quotes
            .iter()
            .map(|q| self.analyze_quote(q))
            .collect()
    }

    /// 获取推荐买入列表
    pub fn get_recommendations(&self, quotes: &[AuctionQuote]) -> Vec<AuctionAnalysis> {
        self.analyze_quotes(quotes)
            .into_iter()
            .filter(|a| a.is_recommended)
            .collect()
    }

    /// 按板块统计
    ///
    /// 注意：当前版本使用简单的板块分组（基于股票代码前缀）
    /// 实际生产环境应该从数据库读取板块分类
    pub fn analyze_by_sector(&self, quotes: &[AuctionQuote]) -> Vec<SectorStats> {
        let mut sector_map: HashMap<String, Vec<&AuctionQuote>> = HashMap::new();

        for quote in quotes {
            // 简化的板块分组（实际应从数据库读取）
            let sector = Self::classify_sector(&quote.code);
            sector_map
                .entry(sector)
                .or_insert_with(Vec::new)
                .push(quote);
        }

        let mut stats = Vec::new();

        for (sector_name, sector_quotes) in sector_map {
            let stock_count = sector_quotes.len();
            let avg_strength_score = sector_quotes
                .iter()
                .map(|q| q.strength_score)
                .sum::<f32>()
                / stock_count.max(1) as f32;

            let avg_change_percent = sector_quotes
                .iter()
                .map(|q| q.change_percent)
                .sum::<f64>()
                / stock_count.max(1) as f64;

            let total_sealed_buy = sector_quotes.iter().map(|q| q.sealed_amount_buy).sum();
            let total_sealed_sell = sector_quotes.iter().map(|q| q.sealed_amount_sell).sum();

            let strong_count = sector_quotes
                .iter()
                .filter(|q| q.strength_score >= 70.0)
                .count();

            // 转换为拥有的值用于分析
            let owned_quotes: Vec<AuctionQuote> = sector_quotes
                .iter()
                .map(|q| (*q).clone())
                .collect();

            let recommended_count = self
                .analyze_quotes(&owned_quotes)
                .iter()
                .filter(|a| a.is_recommended)
                .count();

            stats.push(SectorStats {
                sector_name,
                stock_count,
                avg_strength_score,
                avg_change_percent,
                total_sealed_buy,
                total_sealed_sell,
                strong_count,
                recommended_count,
            });
        }

        // 按平均强度评分降序排序
        stats.sort_by(|a, b| {
            b.avg_strength_score
                .partial_cmp(&a.avg_strength_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        stats
    }

    /// 简化的板块分类（基于股票代码）
    fn classify_sector(code: &str) -> String {
        if code.starts_with("600") || code.starts_with("601") || code.starts_with("603") {
            "上海主板".to_string()
        } else if code.starts_with("688") {
            "科创板".to_string()
        } else if code.starts_with("000") || code.starts_with("001") {
            "深圳主板".to_string()
        } else if code.starts_with("300") {
            "创业板".to_string()
        } else {
            "其他".to_string()
        }
    }

    /// 获取抢筹强度排名
    pub fn rank_by_strength(&self, quotes: &[AuctionQuote]) -> Vec<AuctionAnalysis> {
        let mut analyses = self.analyze_quotes(quotes);
        analyses.sort_by(|a, b| {
            b.strength_score
                .partial_cmp(&a.strength_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        analyses
    }

    /// 获取买封金额排名
    pub fn rank_by_sealed_amount(&self, quotes: &[AuctionQuote]) -> Vec<AuctionAnalysis> {
        let mut analyses = self.analyze_quotes(quotes);
        analyses.sort_by(|a, b| {
            b.sealed_amount_buy
                .partial_cmp(&a.sealed_amount_buy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        analyses
    }
}

impl Default for AuctionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算封单匹配度
///
/// 返回值范围：0.0-1.0
/// - 接近 1.0：买卖均衡
/// - 接近 0.0：一边倒
pub fn calculate_matched_ratio(buy_sealed: f64, sell_sealed: f64) -> f32 {
    if buy_sealed == 0.0 && sell_sealed == 0.0 {
        return 1.0;
    }

    let max_sealed = buy_sealed.max(sell_sealed);
    let min_sealed = buy_sealed.min(sell_sealed);

    if max_sealed == 0.0 {
        1.0
    } else {
        (min_sealed / max_sealed) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::auction_collector::AuctionQuote;

    #[test]
    fn test_strength_level_from_score() {
        assert_eq!(StrengthLevel::from_score(95.0), StrengthLevel::Extreme);
        assert_eq!(StrengthLevel::from_score(75.0), StrengthLevel::High);
        assert_eq!(StrengthLevel::from_score(55.0), StrengthLevel::Medium);
        assert_eq!(StrengthLevel::from_score(35.0), StrengthLevel::Low);
        assert_eq!(StrengthLevel::from_score(15.0), StrengthLevel::VeryLow);
    }

    #[test]
    fn test_calculate_matched_ratio() {
        assert_eq!(calculate_matched_ratio(1_000_000.0, 1_000_000.0), 1.0);
        assert_eq!(calculate_matched_ratio(100_000.0, 1_000_000.0), 0.1);
        assert_eq!(calculate_matched_ratio(0.0, 0.0), 1.0);
    }

    #[test]
    fn test_auction_analyzer() {
        let analyzer = AuctionAnalyzer::new();

        let quote = AuctionQuote {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            time: "2026-01-01 09:20:00".to_string(),
            price: 11.50,
            pre_close: 10.50,
            volume: 5_000_000,
            amount: 57_500_000.0,
            buy1_price: 11.50,
            buy1_volume: 100_000,
            sell1_price: 11.60,
            sell1_volume: 10_000,
            change_percent: 9.52,
            sealed_amount_buy: 1_150_000.0,
            sealed_amount_sell: 116_000.0,
            strength_score: 85.0,
        };

        let analysis = analyzer.analyze_quote(&quote);

        assert_eq!(analysis.code, "000001");
        assert!(analysis.strength_score >= 80.0);
    }

    #[test]
    fn test_classify_sector() {
        assert_eq!(AuctionAnalyzer::classify_sector("600000"), "上海主板");
        assert_eq!(AuctionAnalyzer::classify_sector("688001"), "科创板");
        assert_eq!(AuctionAnalyzer::classify_sector("000001"), "深圳主板");
        assert_eq!(AuctionAnalyzer::classify_sector("300001"), "创业板");
    }
}
