//! CSV 文件解析器
//!
//! 解析 CSV 文件为股票列表，支持多种列格式

use std::path::Path;

use super::code_resolver::CodeResolver;
use super::types::{ImportItem, ImportResult, ImportSource};

/// CSV 解析器
pub struct CsvParser {
    resolver: CodeResolver,
}

impl CsvParser {
    pub fn new(resolver: CodeResolver) -> Self {
        Self { resolver }
    }

    pub fn with_defaults() -> Self {
        Self::new(CodeResolver::new())
    }

    /// 从文件解析
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> crate::core::Result<ImportResult> {
        let path = path.as_ref();

        // 读取文件
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::core::QuantixError::Other(format!("读取文件失败 {}: {}", path.display(), e))
        })?;

        self.parse_content(&content)
    }

    /// 从内容解析
    pub fn parse_content(&self, content: &str) -> crate::core::Result<ImportResult> {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        let mut lines = content.lines().peekable();

        // 尝试检测分隔符
        let first_line = lines.peek().copied().unwrap_or("");
        let delimiter = detect_delimiter(first_line);

        // 检测是否有 header
        let has_header = detect_header(first_line, delimiter);
        if has_header {
            lines.next(); // 跳过 header
        }

        let total_input_lines =
            content
                .lines()
                .count()
                .saturating_sub(if has_header { 1 } else { 0 });

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let fields: Vec<&str> = if delimiter == b'\t' {
                line.split('\t').collect()
            } else if delimiter == b',' {
                line.split(',').collect()
            } else {
                line.split_whitespace().collect()
            };

            if fields.is_empty() {
                continue;
            }

            // 尝试从每行提取股票信息
            if let Some(item) = self.extract_from_fields(&fields) {
                items.push(item);
            } else {
                errors.push(format!("无法解析行: {}", line));
            }
        }

        // 去重
        items.dedup_by(|a, b| a.code.as_ref() == b.code.as_ref() && a.code.is_some());

        let parsed_count = items.len();

        Ok(ImportResult {
            items,
            total_input_lines,
            parsed_count,
            skipped_count: total_input_lines.saturating_sub(parsed_count),
            errors,
        })
    }

    /// 从字段中提取股票信息
    fn extract_from_fields(&self, fields: &[&str]) -> Option<ImportItem> {
        // 策略: 遍历所有字段，找第一个能解析的
        for field in fields {
            let field = field.trim().trim_matches('"');
            if field.is_empty() {
                continue;
            }

            if let Some(result) = self.resolver.resolve(field) {
                return Some(ImportItem {
                    code: Some(result.code),
                    name: result.name,
                    confidence: result.confidence,
                    source: ImportSource::Csv,
                    raw_text: Some(fields.join(",")),
                });
            }
        }

        // 如果有多个字段，尝试组合 "代码 名称" 对
        if fields.len() >= 2 {
            let code_field = fields[0].trim().trim_matches('"');
            if super::types::is_code_like(code_field) {
                let name = fields[1].trim().trim_matches('"');
                return Some(ImportItem {
                    code: Some(super::types::normalize_code(code_field)),
                    name: if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    },
                    confidence: 0.95,
                    source: ImportSource::Csv,
                    raw_text: Some(fields.join(",")),
                });
            }
        }

        None
    }
}

/// 检测分隔符
fn detect_delimiter(line: &str) -> u8 {
    let tab_count = line.matches('\t').count();
    let comma_count = line.matches(',').count();

    if tab_count > comma_count && tab_count > 0 {
        b'\t'
    } else if comma_count > 0 {
        b','
    } else {
        b' '
    }
}

/// 检测是否有 header 行
fn detect_header(line: &str, delimiter: u8) -> bool {
    let fields: Vec<&str> = if delimiter == b'\t' {
        line.split('\t').collect()
    } else if delimiter == b',' {
        line.split(',').collect()
    } else {
        line.split_whitespace().collect()
    };

    // 如果第一个字段是常见 header 名称
    let first = fields
        .first()
        .map(|f| f.trim().trim_matches('"'))
        .unwrap_or("");
    let header_keywords = [
        "code",
        "代码",
        "股票代码",
        "股票",
        "stock",
        "symbol",
        "name",
        "名称",
        "股票名称",
        "no",
        "编号",
        "序号",
    ];

    header_keywords
        .iter()
        .any(|kw| first.eq_ignore_ascii_case(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_with_header() {
        let parser = CsvParser::with_defaults();
        let content = "code,name\n000001,平安银行\n600036,招商银行";
        let result = parser.parse_content(content).unwrap();
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_parse_csv_no_header() {
        let parser = CsvParser::with_defaults();
        let content = "000001\n600036\n600519";
        let result = parser.parse_content(content).unwrap();
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_parse_tab_separated() {
        let parser = CsvParser::with_defaults();
        let content = "000001\t平安银行\n600036\t招商银行";
        let result = parser.parse_content(content).unwrap();
        assert!(result.items.len() >= 2);
    }
}
