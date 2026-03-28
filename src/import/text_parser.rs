//! 文本/剪贴板解析器
//!
//! 解析纯文本为股票列表，支持多种格式:
//! - 每行一个代码或名称
//! - 逗号/空格/制表符分隔
//! - "代码 名称" 对格式

use super::code_resolver::CodeResolver;
use super::types::{ImportItem, ImportSource, ImportResult};

/// 文本解析器
pub struct TextParser {
    resolver: CodeResolver,
}

impl TextParser {
    pub fn new(resolver: CodeResolver) -> Self {
        Self { resolver }
    }

    /// 使用默认解析器
    pub fn with_defaults() -> Self {
        Self::new(CodeResolver::new())
    }

    /// 解析文本为导入结果
    pub fn parse(&self, text: &str, source: ImportSource) -> ImportResult {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        // 按行分割
        let lines: Vec<&str> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let total_input_lines = lines.len();

        for line in &lines {
            // 尝试多种分割方式
            let tokens = self.tokenize_line(line);

            for token in tokens {
                if token.is_empty() {
                    continue;
                }

                if let Some(result) = self.resolver.resolve(&token) {
                    items.push(ImportItem {
                        code: Some(result.code),
                        name: result.name,
                        confidence: result.confidence,
                        source: source.clone(),
                        raw_text: Some(token.clone()),
                    });
                } else {
                    // 无法解析，保留原始文本
                    errors.push(format!("无法解析: {}", token));
                }
            }
        }

        // 去重 (按代码)
        items.dedup_by(|a, b| {
            a.code.as_ref() == b.code.as_ref() && a.code.is_some()
        });

        let parsed_count = items.len();
        let skipped_count = total_input_lines.saturating_sub(parsed_count);

        ImportResult {
            items,
            total_input_lines,
            parsed_count,
            skipped_count,
            errors,
        }
    }

    /// 分割一行为多个 token
    fn tokenize_line(&self, line: &str) -> Vec<String> {
        // 先尝试制表符
        if line.contains('\t') {
            return line.split('\t')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 尝试逗号
        if line.contains(',') || line.contains('，') {
            return line.split(&[',', '，'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 尝试分号
        if line.contains(';') || line.contains('；') {
            return line.split(&[';', '；'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // "代码 名称" 对格式 (如 "000001 平安银行")
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let first = parts[0];
            let second = parts[1];
            // 如果第一个是代码，第二个是名称
            if super::types::is_code_like(first) {
                return vec![first.to_string()];
            }
            // 如果两个都不是代码，可能是两个名称
            if !super::types::is_code_like(second) {
                return vec![first.to_string(), second.to_string()];
            }
        }

        // 单个 token
        vec![line.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codes() {
        let parser = TextParser::with_defaults();
        let result = parser.parse("000001\n600036\n600519", ImportSource::Clipboard);
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].code.as_deref(), Some("000001"));
    }

    #[test]
    fn test_parse_names() {
        let parser = TextParser::with_defaults();
        let result = parser.parse("平安银行\n招商银行", ImportSource::Clipboard);
        assert!(result.items.len() >= 2);
    }

    #[test]
    fn test_parse_comma_separated() {
        let parser = TextParser::with_defaults();
        let result = parser.parse("000001,600036,600519", ImportSource::Text);
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_parse_code_name_pair() {
        let parser = TextParser::with_defaults();
        let result = parser.parse("000001 平安银行", ImportSource::Text);
        assert!(result.items.len() >= 1);
        assert_eq!(result.items[0].code.as_deref(), Some("000001"));
    }
}
