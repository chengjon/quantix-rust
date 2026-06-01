#![allow(clippy::unnecessary_map_or)]

//! 智能导入核心类型

use serde::{Deserialize, Serialize};

/// 导入来源
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportSource {
    Image,
    Csv,
    Excel,
    Text,
    Clipboard,
}

/// 导入项 - 从各种来源提取的股票信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    /// 股票代码 (如 "000001")
    pub code: Option<String>,
    /// 股票名称 (如 "平安银行")
    pub name: Option<String>,
    /// 置信度 0.0-1.0
    pub confidence: f64,
    /// 来源
    pub source: ImportSource,
    /// 原始文本
    pub raw_text: Option<String>,
}

impl ImportItem {
    pub fn new(
        code: Option<String>,
        name: Option<String>,
        confidence: f64,
        source: ImportSource,
    ) -> Self {
        Self {
            code,
            name,
            confidence,
            source,
            raw_text: None,
        }
    }

    /// 是否有有效代码
    pub fn has_code(&self) -> bool {
        self.code.as_ref().map_or(false, |c| !c.is_empty())
    }

    /// 是否有有效名称
    pub fn has_name(&self) -> bool {
        self.name.as_ref().map_or(false, |n| !n.is_empty())
    }
}

/// 代码解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeResolveResult {
    /// 输入文本
    pub input: String,
    /// 解析出的代码
    pub code: String,
    /// 匹配的名称
    pub name: Option<String>,
    /// 匹配方式
    pub match_method: MatchMethod,
    /// 置信度
    pub confidence: f64,
}

/// 匹配方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMethod {
    /// 精确匹配代码
    ExactCode,
    /// 精确匹配名称
    ExactName,
    /// 拼音首字母匹配
    Pinyin,
    /// 模糊匹配
    Fuzzy,
    /// 在线查询
    Online,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// 导入项列表
    pub items: Vec<ImportItem>,
    /// 原始输入行数
    pub total_input_lines: usize,
    /// 成功解析数
    pub parsed_count: usize,
    /// 跳过数
    pub skipped_count: usize,
    /// 错误信息
    pub errors: Vec<String>,
}

/// 判断文本是否像股票代码
pub fn is_code_like(text: &str) -> bool {
    let trimmed = text.trim();
    // A股: 6位数字
    if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let first = trimmed.chars().next().unwrap();
        return matches!(first, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '8');
    }
    false
}

/// 标准化股票代码
pub fn normalize_code(code: &str) -> String {
    let code = code.trim();
    // 去掉市场前缀
    let code = code
        .trim_start_matches("SH")
        .trim_start_matches("SZ")
        .trim_start_matches("BJ")
        .trim_start_matches("sh")
        .trim_start_matches("sz")
        .trim_start_matches("bj");
    // 去掉 .SH / .SZ 后缀
    let code = code
        .trim_end_matches(".SH")
        .trim_end_matches(".SZ")
        .trim_end_matches(".BJ")
        .trim_end_matches(".sh")
        .trim_end_matches(".sz")
        .trim_end_matches(".bj");
    code.trim().to_string()
}
