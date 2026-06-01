//! Excel 文件解析器
//!
//! 解析 Excel worksheet 为股票列表，支持代码/名称 watchlist 表格。

use std::path::Path;

use calamine::{Data, Reader, open_workbook_auto};

use super::code_resolver::CodeResolver;
use super::types::{ImportItem, ImportResult, ImportSource};
use crate::core::{QuantixError, Result};

/// Excel 解析器
pub struct ExcelParser {
    resolver: CodeResolver,
}

impl ExcelParser {
    pub fn new(resolver: CodeResolver) -> Self {
        Self { resolver }
    }

    pub fn with_defaults() -> Self {
        Self::new(CodeResolver::new())
    }

    /// 从 Excel 文件解析指定 sheet；未指定时读取第一个 worksheet。
    pub fn parse_file<P: AsRef<Path>>(&self, path: P, sheet: Option<&str>) -> Result<ImportResult> {
        let path = path.as_ref();
        let mut workbook = open_workbook_auto(path).map_err(|e| {
            QuantixError::Other(format!("读取 Excel 文件失败 {}: {}", path.display(), e))
        })?;

        let sheet_names = workbook.sheet_names();
        let sheet_name = select_sheet_name(&sheet_names, sheet)?;
        let range = workbook.worksheet_range(&sheet_name).map_err(|e| {
            QuantixError::Other(format!(
                "读取 Excel sheet 失败 {}!{}: {}",
                path.display(),
                sheet_name,
                e
            ))
        })?;

        Ok(self.parse_rows(range.rows()))
    }

    fn parse_rows<'a, I>(&self, rows: I) -> ImportResult
    where
        I: IntoIterator<Item = &'a [Data]>,
    {
        let mut logical_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| row.iter().map(cell_to_string).collect::<Vec<_>>())
            .filter(|row| row.iter().any(|field| !field.trim().is_empty()))
            .collect();

        let has_header = logical_rows
            .first()
            .map(|row| detect_header(row))
            .unwrap_or(false);
        if has_header {
            logical_rows.remove(0);
        }

        let total_input_lines = logical_rows.len();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for row in logical_rows {
            let fields: Vec<&str> = row.iter().map(String::as_str).collect();
            if let Some(item) = self.extract_from_fields(&fields) {
                items.push(item);
            } else {
                errors.push(format!("无法解析行: {}", row.join("\t")));
            }
        }

        items.dedup_by(|a, b| a.code.as_ref() == b.code.as_ref() && a.code.is_some());
        let parsed_count = items.len();

        ImportResult {
            items,
            total_input_lines,
            parsed_count,
            skipped_count: total_input_lines.saturating_sub(parsed_count),
            errors,
        }
    }

    fn extract_from_fields(&self, fields: &[&str]) -> Option<ImportItem> {
        if fields.len() >= 2 {
            let code_field = fields[0].trim().trim_matches('"');
            let name = fields[1].trim().trim_matches('"');
            if super::types::is_code_like(code_field) && !name.is_empty() {
                return Some(ImportItem {
                    code: Some(super::types::normalize_code(code_field)),
                    name: Some(name.to_string()),
                    confidence: 0.95,
                    source: ImportSource::Excel,
                    raw_text: Some(fields.join("\t")),
                });
            }
        }

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
                    source: ImportSource::Excel,
                    raw_text: Some(fields.join("\t")),
                });
            }
        }

        if fields.len() >= 2 {
            let code_field = fields[0].trim().trim_matches('"');
            if super::types::is_code_like(code_field) {
                return Some(ImportItem {
                    code: Some(super::types::normalize_code(code_field)),
                    name: None,
                    confidence: 0.95,
                    source: ImportSource::Excel,
                    raw_text: Some(fields.join("\t")),
                });
            }
        }

        None
    }
}

impl Default for ExcelParser {
    fn default() -> Self {
        Self::with_defaults()
    }
}

fn select_sheet_name(sheet_names: &[String], requested: Option<&str>) -> Result<String> {
    if sheet_names.is_empty() {
        return Err(QuantixError::Other(
            "Excel 文件不包含可读取的 worksheet".to_string(),
        ));
    }

    match requested {
        Some(name) if sheet_names.iter().any(|sheet| sheet == name) => Ok(name.to_string()),
        Some(name) => Err(QuantixError::Other(format!(
            "Excel sheet 不存在: {}；可用 sheet: {}",
            name,
            sheet_names.join(", ")
        ))),
        None => Ok(sheet_names[0].clone()),
    }
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.trim().to_string(),
        Data::Float(value) if value.fract() == 0.0 => format!("{value:.0}"),
        Data::Float(value) => value.to_string(),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) => value.trim().to_string(),
        Data::DurationIso(value) => value.trim().to_string(),
        Data::Error(value) => value.to_string(),
    }
}

fn detect_header(row: &[String]) -> bool {
    let first = row.first().map(|f| f.trim()).unwrap_or("");
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
        .any(|keyword| first.eq_ignore_ascii_case(keyword))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::import::ImportSource;
    use std::fs::File;
    use std::io::{self, Write};
    use std::path::Path;

    #[test]
    fn parses_named_sheet_as_watchlist_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("watchlist.xlsx");
        write_minimal_xlsx(
            &path,
            "positions",
            &[
                &["代码", "名称"],
                &["000001", "平安银行"],
                &["600000", "浦发银行"],
            ],
        )
        .unwrap();

        let parser = ExcelParser::with_defaults();
        let result = parser.parse_file(&path, Some("positions")).unwrap();

        assert_eq!(result.total_input_lines, 2);
        assert_eq!(result.parsed_count, 2);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.items[0].code.as_deref(), Some("000001"));
        assert_eq!(result.items[0].name.as_deref(), Some("平安银行"));
        assert_eq!(result.items[0].source, ImportSource::Excel);
        assert_eq!(result.items[1].code.as_deref(), Some("600000"));
        assert_eq!(result.items[1].name.as_deref(), Some("浦发银行"));
    }

    pub(crate) fn write_minimal_xlsx(
        path: &Path,
        sheet_name: &str,
        rows: &[&[&str]],
    ) -> io::Result<()> {
        let worksheet = build_worksheet_xml(rows);
        let workbook = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="{}" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#,
            escape_xml(sheet_name)
        );

        write_zip_stored(
            path,
            &[
                (
                    "[Content_Types].xml",
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#
                        .as_bytes()
                        .to_vec(),
                ),
                (
                    "_rels/.rels",
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#
                        .as_bytes()
                        .to_vec(),
                ),
                ("xl/workbook.xml", workbook.into_bytes()),
                (
                    "xl/_rels/workbook.xml.rels",
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#
                        .as_bytes()
                        .to_vec(),
                ),
                ("xl/worksheets/sheet1.xml", worksheet.into_bytes()),
            ],
        )
    }

    fn build_worksheet_xml(rows: &[&[&str]]) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>"#,
        );
        for (row_index, row) in rows.iter().enumerate() {
            let row_number = row_index + 1;
            xml.push_str(&format!(r#"<row r="{row_number}">"#));
            for (column_index, value) in row.iter().enumerate() {
                let cell_ref = format!("{}{}", column_name(column_index), row_number);
                xml.push_str(&format!(
                    r#"<c r="{cell_ref}" t="inlineStr"><is><t>{}</t></is></c>"#,
                    escape_xml(value)
                ));
            }
            xml.push_str("</row>");
        }
        xml.push_str("</sheetData></worksheet>");
        xml
    }

    fn column_name(mut index: usize) -> String {
        let mut name = String::new();
        loop {
            let rem = index % 26;
            name.insert(0, (b'A' + rem as u8) as char);
            if index < 26 {
                return name;
            }
            index = index / 26 - 1;
        }
    }

    fn escape_xml(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn write_zip_stored(path: &Path, files: &[(&str, Vec<u8>)]) -> io::Result<()> {
        let mut out = File::create(path)?;
        let mut central_directory = Vec::new();
        let mut offset = 0u32;

        for (name, content) in files {
            let name_bytes = name.as_bytes();
            let crc = crc32(content);
            write_u32(&mut out, 0x0403_4b50)?;
            write_u16(&mut out, 20)?;
            write_u16(&mut out, 0)?;
            write_u16(&mut out, 0)?;
            write_u16(&mut out, 0)?;
            write_u16(&mut out, 0)?;
            write_u32(&mut out, crc)?;
            write_u32(&mut out, content.len() as u32)?;
            write_u32(&mut out, content.len() as u32)?;
            write_u16(&mut out, name_bytes.len() as u16)?;
            write_u16(&mut out, 0)?;
            out.write_all(name_bytes)?;
            out.write_all(content)?;

            write_u32(&mut central_directory, 0x0201_4b50)?;
            write_u16(&mut central_directory, 20)?;
            write_u16(&mut central_directory, 20)?;
            write_u16(&mut central_directory, 0)?;
            write_u16(&mut central_directory, 0)?;
            write_u16(&mut central_directory, 0)?;
            write_u16(&mut central_directory, 0)?;
            write_u32(&mut central_directory, crc)?;
            write_u32(&mut central_directory, content.len() as u32)?;
            write_u32(&mut central_directory, content.len() as u32)?;
            write_u16(&mut central_directory, name_bytes.len() as u16)?;
            write_u16(&mut central_directory, 0)?;
            write_u16(&mut central_directory, 0)?;
            write_u16(&mut central_directory, 0)?;
            write_u16(&mut central_directory, 0)?;
            write_u32(&mut central_directory, 0)?;
            write_u32(&mut central_directory, offset)?;
            central_directory.write_all(name_bytes)?;

            offset += 30 + name_bytes.len() as u32 + content.len() as u32;
        }

        let central_offset = offset;
        out.write_all(&central_directory)?;
        write_u32(&mut out, 0x0605_4b50)?;
        write_u16(&mut out, 0)?;
        write_u16(&mut out, 0)?;
        write_u16(&mut out, files.len() as u16)?;
        write_u16(&mut out, files.len() as u16)?;
        write_u32(&mut out, central_directory.len() as u32)?;
        write_u32(&mut out, central_offset)?;
        write_u16(&mut out, 0)?;
        Ok(())
    }

    fn write_u16<W: Write>(writer: &mut W, value: u16) -> io::Result<()> {
        writer.write_all(&value.to_le_bytes())
    }

    fn write_u32<W: Write>(writer: &mut W, value: u32) -> io::Result<()> {
        writer.write_all(&value.to_le_bytes())
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0u32.wrapping_sub(crc & 1);
                crc = (crc >> 1) ^ (0xedb8_8320 & mask);
            }
        }
        !crc
    }
}
