//! ModelProvider 子系统的领域层（对应 model_providers 表）。
//!
//! 模型供应商：预设/自定义/平台三类，按 user_id 隔离。
//! 运行时 LLM 编排器按 provider_id 解析 api_key/base_url/model/api_format。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 合法的 API 格式。运行时据此选择 rig 客户端。
pub const API_FORMATS: &[&str] = &[
    "anthropic",
    "openai_chat",
    "openai_responses",
    "gemini_native",
];

/// 供应商分类。
pub const CATEGORIES: &[&str] = &["preset", "custom", "platform"];

pub fn validate_api_format(s: &str) -> bool {
    API_FORMATS.contains(&s)
}

pub fn validate_category(s: &str) -> bool {
    CATEGORIES.contains(&s)
}

pub fn validate_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 64
}

/// 模型供应商完整记录（含明文 api_key，仅内部与运行时使用）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelProvider {
    pub id: Uuid,
    pub owner_id: i32,
    pub name: String,
    pub category: String,
    pub preset_type: String,
    pub base_url: String,
    pub api_key: String,
    pub api_format: String,
    pub models: Vec<String>,
    pub enabled: bool,
    pub website_url: String,
    pub api_key_url: String,
    pub icon: String,
    pub icon_color: String,
    pub sort_order: i32,
}

/// 创建/更新供应商的输入。api_key 为空串时表示更新时保留旧值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderInput {
    pub name: String,
    pub category: String,
    pub preset_type: String,
    pub base_url: String,
    pub api_key: String,
    pub api_format: String,
    pub models: Vec<String>,
    pub enabled: bool,
    pub website_url: String,
    pub api_key_url: String,
    pub icon: String,
    pub icon_color: String,
    pub sort_order: i32,
}

impl Default for ModelProviderInput {
    fn default() -> Self {
        Self {
            name: String::new(),
            category: "custom".into(),
            preset_type: String::new(),
            base_url: String::new(),
            api_key: String::new(),
            api_format: "anthropic".into(),
            models: Vec::new(),
            enabled: true,
            website_url: String::new(),
            api_key_url: String::new(),
            icon: String::new(),
            icon_color: String::new(),
            sort_order: 0,
        }
    }
}

/// 返回给前端的脱敏 DTO：不含 api_key，附 has_api_key。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProviderDto {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub preset_type: String,
    pub base_url: String,
    pub has_api_key: bool,
    pub api_format: String,
    pub models: Vec<String>,
    pub enabled: bool,
    pub website_url: String,
    pub api_key_url: String,
    pub icon: String,
    pub icon_color: String,
    pub sort_order: i32,
}

impl ModelProvider {
    pub fn to_dto(&self) -> ModelProviderDto {
        ModelProviderDto {
            id: self.id,
            name: self.name.clone(),
            category: self.category.clone(),
            preset_type: self.preset_type.clone(),
            base_url: self.base_url.clone(),
            has_api_key: !self.api_key.trim().is_empty(),
            api_format: self.api_format.clone(),
            models: self.models.clone(),
            enabled: self.enabled,
            website_url: self.website_url.clone(),
            api_key_url: self.api_key_url.clone(),
            icon: self.icon.clone(),
            icon_color: self.icon_color.clone(),
            sort_order: self.sort_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_formats() {
        assert!(validate_api_format("anthropic"));
        assert!(validate_api_format("openai_chat"));
        assert!(!validate_api_format("unknown"));
        assert!(validate_category("preset"));
        assert!(!validate_category("other"));
    }

    #[test]
    fn dto_masks_api_key() {
        let p = ModelProvider {
            id: Uuid::new_v4(),
            owner_id: 1,
            name: "智谱".into(),
            category: "preset".into(),
            preset_type: "zhipu".into(),
            base_url: "https://open.bigmodel.cn/api/anthropic".into(),
            api_key: "sk-secret".into(),
            api_format: "anthropic".into(),
            models: vec!["glm-5.1".into()],
            enabled: true,
            website_url: String::new(),
            api_key_url: String::new(),
            icon: "zhipu".into(),
            icon_color: "#0F62FE".into(),
            sort_order: 0,
        };
        let dto = p.to_dto();
        assert!(dto.has_api_key);
        assert_eq!(dto.models, vec!["glm-5.1".to_string()]);
        // 序列化不含明文密钥
        let json = serde_json::to_string(&dto).unwrap();
        assert!(!json.contains("sk-secret"));
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("智谱 GLM"));
        assert!(!validate_name(""));
        assert!(!validate_name(&"x".repeat(65)));
    }
}
