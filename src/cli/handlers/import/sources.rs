use super::*;

pub(super) async fn run_import_from_image(file: &str, model: &str) -> Result<()> {
    let provider = crate::import::ImageVisionProvider::parse(model)?;
    let extractor = crate::import::ImageExtractor::with_provider(provider);
    let result = extractor.extract_from_file(file).await?;

    println!("📷 图片股票识别");
    println!("   文件: {}", file);
    println!("   模型: {}", model);
    println!();

    if result.items.is_empty() {
        if !result.errors.is_empty() {
            println!("❌ {}", result.errors[0]);
        } else {
            println!("❌ 未从图片中识别到股票信息");
        }
        return Ok(());
    }

    println!("✅ 识别到 {} 只股票:", result.items.len());
    println!();
    println!("{:<10} {:<12} {:<8} 来源", "代码", "名称", "置信度");
    println!("{}", "-".repeat(50));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!(
            "{:<10} {:<12} {:.0}%     {:?}",
            code,
            name,
            item.confidence * 100.0,
            item.source
        );
    }

    Ok(())
}

pub(super) async fn run_import_from_csv(file: &str) -> Result<()> {
    println!("📄 CSV 导入");
    println!("   文件: {}", file);
    println!();

    let parser = crate::import::CsvParser::with_defaults();
    let result = parser.parse_file(file)?;

    if result.items.is_empty() {
        println!("❌ 未从 CSV 中解析到股票信息");
        if !result.errors.is_empty() {
            println!();
            println!("解析错误:");
            for err in &result.errors {
                println!("   - {}", err);
            }
        }
        return Ok(());
    }

    println!(
        "✅ 解析完成: {} 只股票 (共 {} 行, 跳过 {} 行)",
        result.parsed_count, result.total_input_lines, result.skipped_count
    );
    println!();
    println!("{:<10} {:<12} 置信度", "代码", "名称");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

pub(super) async fn run_import_from_excel(file: &str, sheet: Option<&str>) -> Result<()> {
    println!("📊 Excel 导入");
    println!("   文件: {}", file);
    if let Some(sheet) = sheet {
        println!("   Sheet: {}", sheet);
    }
    println!();

    let parser = crate::import::ExcelParser::with_defaults();
    let result = parser.parse_file(file, sheet)?;

    if result.items.is_empty() {
        println!("❌ 未从 Excel 中解析到股票信息");
        if !result.errors.is_empty() {
            println!();
            println!("解析错误:");
            for err in &result.errors {
                println!("   - {}", err);
            }
        }
        return Ok(());
    }

    println!(
        "✅ 解析完成: {} 只股票 (共 {} 行, 跳过 {} 行)",
        result.parsed_count, result.total_input_lines, result.skipped_count
    );
    println!();
    println!("{:<10} {:<12} 置信度", "代码", "名称");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

pub(super) async fn run_import_from_clipboard() -> Result<()> {
    println!("📋 剪贴板导入");
    println!();

    let clipboard_content = get_clipboard_content()?;

    if clipboard_content.is_empty() {
        println!("❌ 剪贴板为空");
        return Ok(());
    }

    println!("📝 剪贴板内容 (前 200 字符):");
    let preview: String = clipboard_content.chars().take(200).collect();
    println!("   {}", preview);
    println!();

    let parser = crate::import::TextParser::with_defaults();
    let result = parser.parse(&clipboard_content, crate::import::ImportSource::Clipboard);

    if result.items.is_empty() {
        println!("❌ 未从剪贴板内容中解析到股票信息");
        return Ok(());
    }

    println!("✅ 解析到 {} 只股票:", result.items.len());
    println!();
    println!("{:<10} {:<12} 置信度", "代码", "名称");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

pub(super) async fn run_import_from_text(text: &str) -> Result<()> {
    println!("📝 文本导入");
    println!("   输入: {}", text);
    println!();

    let parser = crate::import::TextParser::with_defaults();
    let result = parser.parse(text, crate::import::ImportSource::Text);

    if result.items.is_empty() {
        println!("❌ 未从文本中解析到股票信息");
        if !result.errors.is_empty() {
            println!();
            println!("解析错误:");
            for err in &result.errors {
                println!("   - {}", err);
            }
        }
        return Ok(());
    }

    println!("✅ 解析到 {} 只股票:", result.items.len());
    println!();
    println!("{:<10} {:<12} 置信度", "代码", "名称");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

pub(super) async fn run_import_resolve(input: &str) -> Result<()> {
    println!("🔍 股票代码/名称解析");
    println!("   输入: {}", input);
    println!();

    let resolver = crate::import::CodeResolver::new();

    match resolver.resolve(input) {
        Some(result) => {
            println!("✅ 解析成功:");
            println!("   代码: {}", result.code);
            if let Some(name) = &result.name {
                println!("   名称: {}", name);
            }
            println!("   匹配方式: {:?}", result.match_method);
            println!("   置信度: {:.0}%", result.confidence * 100.0);
        }
        None => {
            println!("❌ 无法解析: {}", input);
            println!();
            println!("💡 提示:");
            println!("   - 输入6位数字代码 (如 000001)");
            println!("   - 输入股票名称 (如 平安银行)");
            println!("   - 输入拼音首字母 (如 PAYH)");
        }
    }

    Ok(())
}
