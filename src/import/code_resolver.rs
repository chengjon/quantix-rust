//! 股票名称/代码解析器
//!
//! 支持精确匹配、拼音首字母匹配、模糊匹配

use std::collections::HashMap;

use super::types::{CodeResolveResult, MatchMethod};

/// 内置股票名称映射 (常用 A 股)
const STOCK_NAME_MAP: &[(&str, &str)] = &[
    ("平安银行", "000001"),
    ("万科A", "000002"),
    ("国信证券", "002736"),
    ("紫金矿业", "601899"),
    ("招商银行", "600036"),
    ("贵州茅台", "600519"),
    ("中国平安", "601318"),
    ("宁德时代", "300750"),
    ("比亚迪", "002594"),
    ("五粮液", "000858"),
    ("隆基绿能", "601012"),
    ("长江电力", "600900"),
    ("中国中免", "601888"),
    ("美的集团", "000333"),
    ("格力电器", "000651"),
    ("恒瑞医药", "600276"),
    ("药明康德", "603259"),
    ("海康威视", "002415"),
    ("迈瑞医疗", "300760"),
    ("中国中车", "601766"),
    ("中信证券", "600030"),
    ("海天味业", "603288"),
    ("爱尔眼科", "300015"),
    ("万科", "000002"),
    ("茅台", "600519"),
    ("平安", "601318"),
    ("招商", "600036"),
    ("比亚迪", "002594"),
    ("宁德", "300750"),
    ("五粮液", "000858"),
    ("美的", "000333"),
    ("格力", "000651"),
    ("隆基", "601012"),
    ("恒瑞", "600276"),
    ("海康", "002415"),
    ("迈瑞", "300760"),
    ("中信", "600030"),
    ("海天", "603288"),
    ("爱尔", "300015"),
    ("中车", "601766"),
    ("紫金", "601899"),
    ("长江电力", "600900"),
    ("中免", "601888"),
    ("立讯精密", "002475"),
    ("立讯", "002475"),
    ("东方财富", "300059"),
    ("东财", "300059"),
    ("三一重工", "600031"),
    ("三一", "600031"),
    ("中芯国际", "688981"),
    ("中芯", "688981"),
    ("韦尔股份", "603501"),
    ("韦尔", "603501"),
    ("汇川技术", "300124"),
    ("汇川", "300124"),
    ("万华化学", "600309"),
    ("万华", "600309"),
    ("智飞生物", "300122"),
    ("智飞", "300122"),
    ("泸州老窖", "000568"),
    ("老窖", "000568"),
    ("山西汾酒", "600809"),
    ("汾酒", "600809"),
    ("阳光电源", "300274"),
    ("阳光", "300274"),
    ("通威股份", "600438"),
    ("通威", "600438"),
    ("泰格医药", "300347"),
    ("泰格", "300347"),
    ("金山办公", "688111"),
    ("金山", "688111"),
    ("中微公司", "688012"),
    ("中微", "688012"),
    ("北方华创", "002371"),
    ("华创", "002371"),
];

/// 拼音首字母映射 (常用简称)
const PINYIN_MAP: &[(&str, &str)] = &[
    ("PAYH", "000001"), // 平安银行
    ("ZSYH", "600036"), // 招商银行
    ("GZMT", "600519"), // 贵州茅台
    ("ZGPA", "601318"), // 中国平安
    ("NDSD", "300750"), // 宁德时代
    ("BYD", "002594"),  // 比亚迪
    ("WLY", "000858"),  // 五粮液
    ("MDJT", "000333"), // 美的集团
    ("GLDQ", "000651"), // 格力电器
    ("HKWS", "002415"), // 海康威视
    ("HRYY", "600276"), // 恒瑞医药
    ("ZXZQ", "600030"), // 中信证券
    ("HTWW", "603288"), // 海天味业
    ("AEYK", "300015"), // 爱尔眼科
    ("LJLN", "601012"), // 隆基绿能
    ("ZJKY", "601899"), // 紫金矿业
];

/// 股票代码解析器
pub struct CodeResolver {
    /// 名称 -> 代码
    name_to_code: HashMap<String, String>,
    /// 代码 -> 名称
    code_to_name: HashMap<String, String>,
    /// 拼音 -> 代码
    pinyin_to_code: HashMap<String, String>,
}

impl CodeResolver {
    pub fn new() -> Self {
        let mut name_to_code = HashMap::new();
        let mut code_to_name = HashMap::new();
        let mut pinyin_to_code = HashMap::new();

        for (name, code) in STOCK_NAME_MAP {
            name_to_code.insert(name.to_string(), code.to_string());
            code_to_name.insert(code.to_string(), name.to_string());
        }

        for (py, code) in PINYIN_MAP {
            pinyin_to_code.insert(py.to_string(), code.to_string());
        }

        Self {
            name_to_code,
            code_to_name,
            pinyin_to_code,
        }
    }

    /// 解析输入文本为股票代码
    pub fn resolve(&self, input: &str) -> Option<CodeResolveResult> {
        let input = input.trim();

        if input.is_empty() {
            return None;
        }

        // 1. 精确代码匹配
        if super::types::is_code_like(input) {
            let code = super::types::normalize_code(input);
            let name = self.code_to_name.get(&code).cloned();
            return Some(CodeResolveResult {
                input: input.to_string(),
                code,
                name,
                match_method: MatchMethod::ExactCode,
                confidence: 1.0,
            });
        }

        // 2. 精确名称匹配
        if let Some(code) = self.name_to_code.get(input) {
            return Some(CodeResolveResult {
                input: input.to_string(),
                code: code.clone(),
                name: Some(input.to_string()),
                match_method: MatchMethod::ExactName,
                confidence: 1.0,
            });
        }

        // 3. 拼音首字母匹配
        let upper = input.to_uppercase();
        if let Some(code) = self.pinyin_to_code.get(&upper) {
            let name = self.code_to_name.get(code).cloned();
            return Some(CodeResolveResult {
                input: input.to_string(),
                code: code.clone(),
                name,
                match_method: MatchMethod::Pinyin,
                confidence: 0.9,
            });
        }

        // 4. 模糊名称匹配 (部分包含)
        for (name, code) in &self.name_to_code {
            if name.contains(input) || input.contains(name) {
                return Some(CodeResolveResult {
                    input: input.to_string(),
                    code: code.clone(),
                    name: Some(name.clone()),
                    match_method: MatchMethod::Fuzzy,
                    confidence: 0.7,
                });
            }
        }

        // 5. 标准化后重试代码匹配
        let normalized = super::types::normalize_code(input);
        if super::types::is_code_like(&normalized) {
            let name = self.code_to_name.get(&normalized).cloned();
            return Some(CodeResolveResult {
                input: input.to_string(),
                code: normalized,
                name,
                match_method: MatchMethod::ExactCode,
                confidence: 0.95,
            });
        }

        None
    }

    /// 批量解析
    pub fn resolve_batch(&self, inputs: &[&str]) -> Vec<Option<CodeResolveResult>> {
        inputs.iter().map(|s| self.resolve(s)).collect()
    }

    /// 获取已知股票数量
    pub fn known_count(&self) -> usize {
        self.name_to_code.len()
    }
}

impl Default for CodeResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_code() {
        let resolver = CodeResolver::new();
        let result = resolver.resolve("000001").unwrap();
        assert_eq!(result.code, "000001");
        assert_eq!(result.name.as_deref(), Some("平安银行"));
        assert_eq!(result.match_method, MatchMethod::ExactCode);
    }

    #[test]
    fn test_exact_name() {
        let resolver = CodeResolver::new();
        let result = resolver.resolve("贵州茅台").unwrap();
        assert_eq!(result.code, "600519");
        assert_eq!(result.match_method, MatchMethod::ExactName);
    }

    #[test]
    fn test_pinyin() {
        let resolver = CodeResolver::new();
        let result = resolver.resolve("GZMT").unwrap();
        assert_eq!(result.code, "600519");
        assert_eq!(result.match_method, MatchMethod::Pinyin);
    }

    #[test]
    fn test_fuzzy() {
        let resolver = CodeResolver::new();
        let result = resolver.resolve("茅台").unwrap();
        assert_eq!(result.code, "600519");
        assert_eq!(result.match_method, MatchMethod::ExactName);
    }

    #[test]
    fn test_normalize() {
        let resolver = CodeResolver::new();
        let result = resolver.resolve("SH600519").unwrap();
        assert_eq!(result.code, "600519");
    }

    #[test]
    fn test_unknown() {
        let resolver = CodeResolver::new();
        assert!(resolver.resolve("未知股票XYZ").is_none());
    }
}
