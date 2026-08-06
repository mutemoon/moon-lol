//! ModelProvider wire DTO（模型供应商 + 模型配置）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API 格式枚举 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiFormat {
    Anthropic,
    #[serde(rename = "openai_chat")]
    OpenaiChat,
    #[serde(rename = "openai_responses")]
    OpenaiResponses,
    #[serde(rename = "gemini_native")]
    GeminiNative,
}

// ── 供应商分类枚举 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderCategory {
    Preset,
    Custom,
    Platform,
}

// ── 模型配置 ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub name: String,
    pub max_tokens: u32,
}

// ── ModelProvider DTO ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub preset_type: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    pub has_api_key: bool,
    pub api_format: String,
    pub models: Vec<ModelConfig>,
    pub enabled: bool,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub api_key_url: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_color: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelProviderInput {
    pub name: String,
    pub category: String,
    pub preset_type: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    pub api_format: String,
    pub models: Vec<ModelConfig>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub website_url: String,
    #[serde(default)]
    pub api_key_url: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub icon_color: String,
    pub sort_order: i32,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestModelProviderInput {
    pub provider_id: Option<Uuid>,
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_format: String,
    pub model: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestModelProviderResponse {
    pub success: bool,
    pub message: String,
}

// ── roundtrip 单测 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_format_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&ApiFormat::Anthropic).unwrap(),
            r#""anthropic""#
        );
        assert_eq!(
            serde_json::to_string(&ApiFormat::OpenaiChat).unwrap(),
            r#""openai_chat""#
        );
        assert_eq!(
            serde_json::to_string(&ApiFormat::OpenaiResponses).unwrap(),
            r#""openai_responses""#
        );
        assert_eq!(
            serde_json::to_string(&ApiFormat::GeminiNative).unwrap(),
            r#""gemini_native""#
        );
    }

    #[test]
    fn api_format_roundtrip() {
        let cases = [
            ("anthropic", ApiFormat::Anthropic),
            ("openai_chat", ApiFormat::OpenaiChat),
            ("openai_responses", ApiFormat::OpenaiResponses),
            ("gemini_native", ApiFormat::GeminiNative),
        ];
        for (s, expected) in cases {
            let f: ApiFormat = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(f, expected);
            assert_eq!(serde_json::to_string(&f).unwrap(), format!(r#""{s}""#));
        }
    }

    #[test]
    fn provider_category_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&ProviderCategory::Preset).unwrap(),
            r#""preset""#
        );
        assert_eq!(
            serde_json::to_string(&ProviderCategory::Custom).unwrap(),
            r#""custom""#
        );
        assert_eq!(
            serde_json::to_string(&ProviderCategory::Platform).unwrap(),
            r#""platform""#
        );
    }

    #[test]
    fn provider_category_roundtrip() {
        let cases = ["preset", "custom", "platform"];
        for s in cases {
            let c: ProviderCategory = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(serde_json::to_string(&c).unwrap(), format!(r#""{s}""#));
        }
    }
}
