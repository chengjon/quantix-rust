#![allow(clippy::collapsible_if, clippy::unnecessary_map_or)]

//! 图片股票代码提取器
//!
//! 使用 LLM Vision API 从图片中提取股票代码和名称

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::core::Result;

use super::code_resolver::CodeResolver;
use super::types::{ImportItem, ImportResult, ImportSource};

const SUPPORTED_IMAGE_FORMATS: &str = "png, jpg, jpeg, gif, webp";

/// 图片格式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
}

impl ImageFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::WebP),
            _ => None,
        }
    }
}

/// 图片提取器
pub struct ImageExtractor {
    resolver: CodeResolver,
    client: reqwest::Client,
    provider: ImageVisionProvider,
}

/// Vision API provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageVisionProvider {
    Deepseek,
    Openai,
}

#[derive(Debug, Clone)]
struct ImageVisionRuntimeConfig {
    api_key: String,
    base_url: String,
    model: String,
}

impl ImageVisionProvider {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "deepseek" => Ok(Self::Deepseek),
            "openai" => Ok(Self::Openai),
            _ => Err(crate::core::QuantixError::Other(format!(
                "不支持的 Vision provider: {value}，支持: deepseek, openai"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Deepseek => "deepseek",
            Self::Openai => "openai",
        }
    }

    fn runtime_config_from_env(self) -> Result<ImageVisionRuntimeConfig> {
        match self {
            Self::Deepseek => {
                let api_key = std::env::var("DEEPSEEK_API_KEY").map_err(|_| {
                    crate::core::QuantixError::Unsupported(format!(
                        "Vision provider 尚未配置: {}；请配置 DEEPSEEK_API_KEY 后再执行 import from-image --model {}",
                        self.as_str(),
                        self.as_str()
                    ))
                })?;
                let base_url = std::env::var("DEEPSEEK_BASE_URL")
                    .unwrap_or_else(|_| "https://api.deepseek.com".to_string());
                let model = std::env::var("DEEPSEEK_VISION_MODEL")
                    .unwrap_or_else(|_| "deepseek-chat".to_string());

                Ok(ImageVisionRuntimeConfig {
                    api_key,
                    base_url,
                    model,
                })
            }
            Self::Openai => {
                let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                    crate::core::QuantixError::Unsupported(format!(
                        "Vision provider 尚未配置: {}；请配置 OPENAI_API_KEY 后再执行 import from-image --model {}",
                        self.as_str(),
                        self.as_str()
                    ))
                })?;
                let base_url = std::env::var("OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
                let model = std::env::var("OPENAI_VISION_MODEL")
                    .unwrap_or_else(|_| "gpt-4o-mini".to_string());

                Ok(ImageVisionRuntimeConfig {
                    api_key,
                    base_url,
                    model,
                })
            }
        }
    }
}

/// LLM 返回的股票项
#[derive(Debug, serde::Deserialize)]
struct LlmStockItem {
    code: Option<String>,
    name: Option<String>,
    #[serde(default)]
    confidence: Option<f64>,
}

/// LLM 返回结构
#[derive(Debug, serde::Deserialize)]
struct LlmExtractionResponse {
    #[serde(default)]
    items: Vec<LlmStockItem>,
}

const VISION_PROMPT: &str = r#"你是一个专业的股票代码识别助手。请从这张图片中识别所有出现的股票代码和股票名称。

请返回 JSON 格式:
```json
{
  "items": [
    {"code": "000001", "name": "平安银行", "confidence": 0.95},
    {"code": "600036", "name": "招商银行", "confidence": 0.9}
  ]
}
```

规则:
1. A股代码为6位数字，沪市以6开头，深市以0/3开头，北交所以8/4开头
2. 如果只能看到名称看不到代码，code 字段留空
3. 如果只能看到代码看不到名称，name 字段留空
4. confidence 为识别置信度，0.0-1.0
5. 只返回 JSON，不要其他文本"#;

impl ImageExtractor {
    pub fn new() -> Self {
        Self::with_provider(ImageVisionProvider::Deepseek)
    }

    pub fn with_provider(provider: ImageVisionProvider) -> Self {
        Self {
            resolver: CodeResolver::new(),
            client: reqwest::Client::new(),
            provider,
        }
    }

    /// 从图片文件提取股票代码
    pub async fn extract_from_file(&self, path: &str) -> Result<ImportResult> {
        // 读取文件
        let file_content = std::fs::read(path).map_err(|e| {
            crate::core::QuantixError::Other(format!("读取图片失败 {}: {}", path, e))
        })?;

        // 检测格式
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");

        let format = ImageFormat::from_extension(ext).ok_or_else(|| {
            crate::core::QuantixError::Unsupported(format!(
                "image format 不支持: {ext}；支持: {SUPPORTED_IMAGE_FORMATS}"
            ))
        })?;

        // 文件大小检查 (5MB)
        if file_content.len() > 5 * 1024 * 1024 {
            return Err(crate::core::QuantixError::Other(
                "图片文件超过 5MB 限制".to_string(),
            ));
        }

        let base64_data = BASE64.encode(&file_content);
        self.extract_from_base64(&base64_data, &format).await
    }

    /// 从 base64 数据提取
    pub async fn extract_from_base64(
        &self,
        base64_data: &str,
        format: &ImageFormat,
    ) -> Result<ImportResult> {
        // 尝试使用 LLM Vision API
        let items = self.call_vision_api(base64_data, format).await?;
        Ok(ImportResult {
            items,
            total_input_lines: 1,
            parsed_count: 0,
            skipped_count: 0,
            errors: vec![],
        })
    }

    /// 调用 Vision API
    async fn call_vision_api(
        &self,
        base64_data: &str,
        format: &ImageFormat,
    ) -> Result<Vec<ImportItem>> {
        let runtime_config = self.provider.runtime_config_from_env()?;

        let image_url = format!("data:{};base64,{}", format.mime_type(), base64_data);

        // 构建请求
        let payload = serde_json::json!({
            "model": runtime_config.model,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": VISION_PROMPT},
                    {"type": "image_url", "image_url": {"url": image_url}}
                ]
            }],
            "max_tokens": 2048,
            "temperature": 0.1
        });

        let url = format!(
            "{}/chat/completions",
            runtime_config.base_url.trim_end_matches('/')
        );

        let response = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", runtime_config.api_key),
            )
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::core::QuantixError::Other(format!("Vision API 请求失败: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(crate::core::QuantixError::Other(format!(
                "Vision API 错误 {}: {}",
                status, body
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::core::QuantixError::Other(format!("解析 API 响应失败: {}", e)))?;

        // 提取内容
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");

        self.parse_llm_response(content)
    }

    /// 解析 LLM 返回的 JSON
    fn parse_llm_response(&self, content: &str) -> Result<Vec<ImportItem>> {
        // 尝试提取 JSON
        let json_str = extract_json_from_text(content);

        let parsed: LlmExtractionResponse = serde_json::from_str(&json_str).map_err(|e| {
            crate::core::QuantixError::Other(format!("解析 LLM 返回的 JSON 失败: {}", e))
        })?;

        let mut items = Vec::new();

        for llm_item in parsed.items {
            let mut code = llm_item.code;
            let mut name = llm_item.name;
            let mut confidence = llm_item.confidence.unwrap_or(0.8);

            // 如果有名称但没有代码，尝试解析
            if code.is_none() || code.as_ref().map_or(true, |c| c.is_empty()) {
                if let Some(ref n) = name {
                    if let Some(result) = self.resolver.resolve(n) {
                        code = Some(result.code);
                        confidence = confidence.min(result.confidence);
                    }
                }
            }

            // 标准化代码
            if let Some(ref c) = code {
                let normalized = super::types::normalize_code(c);
                if !normalized.is_empty() {
                    code = Some(normalized);
                }
            }

            // 如果有代码但没有名称，尝试反向查找
            if name.is_none() || name.as_ref().map_or(true, |n| n.is_empty()) {
                if let Some(ref c) = code {
                    if let Some(result) = self.resolver.resolve(c) {
                        name = result.name;
                    }
                }
            }

            if code.is_some() || name.is_some() {
                items.push(ImportItem {
                    code,
                    name,
                    confidence,
                    source: ImportSource::Image,
                    raw_text: None,
                });
            }
        }

        Ok(items)
    }
}

impl Default for ImageExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// 从文本中提取 JSON 块
fn extract_json_from_text(text: &str) -> String {
    // 尝试提取 ```json ... ``` 块
    if let Some(start) = text.find("```json") {
        let start = start + 7;
        if let Some(end) = text[start..].find("```") {
            return text[start..start + end].trim().to_string();
        }
    }

    // 尝试提取 ``` ... ``` 块
    if let Some(start) = text.find("```") {
        let start = start + 3;
        if let Some(end) = text[start..].find("```") {
            return text[start..start + end].trim().to_string();
        }
    }

    // 尝试提取 { ... }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return text[start..=end].to_string();
        }
    }

    text.trim().to_string()
}
