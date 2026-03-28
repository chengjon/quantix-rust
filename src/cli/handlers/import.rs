use super::*;

// ============================================================
// 导入命令
// ============================================================

/// 处理导入命令
pub async fn run_import_command(cmd: ImportCommands) -> Result<()> {
    match cmd {
        ImportCommands::FromImage { file, model } => {
            run_import_from_image(&file, &model).await
        }
        ImportCommands::FromCsv { file } => {
            run_import_from_csv(&file).await
        }
        ImportCommands::FromClipboard => {
            run_import_from_clipboard().await
        }
        ImportCommands::FromText { text } => {
            run_import_from_text(&text).await
        }
        ImportCommands::Resolve { input } => {
            run_import_resolve(&input).await
        }
    }
}

async fn run_import_from_image(file: &str, model: &str) -> Result<()> {
    println!("📷 图片股票识别");
    println!("   文件: {}", file);
    println!("   模型: {}", model);
    println!();

    let extractor = crate::import::ImageExtractor::new();
    let result = extractor.extract_from_file(file).await?;

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
    println!("{:<10} {:<12} {:<8} {}", "代码", "名称", "置信度", "来源");
    println!("{}", "-".repeat(50));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%     {:?}", code, name, item.confidence * 100.0, item.source);
    }

    Ok(())
}

async fn run_import_from_csv(file: &str) -> Result<()> {
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

    println!("✅ 解析完成: {} 只股票 (共 {} 行, 跳过 {} 行)",
        result.parsed_count, result.total_input_lines, result.skipped_count);
    println!();
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_from_clipboard() -> Result<()> {
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
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_from_text(text: &str) -> Result<()> {
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
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_resolve(input: &str) -> Result<()> {
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

/// 获取剪贴板内容
fn get_clipboard_content() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        if let Ok(output) = std::process::Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("pbpaste").output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-command", "Get-Clipboard"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    Err(QuantixError::Other("无法读取剪贴板内容，请确保已安装 xclip/xsel (Linux)、pbpaste (macOS) 或 PowerShell (Windows)".to_string()))
}
