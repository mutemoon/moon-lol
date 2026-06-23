//! Config 子系统的领域层（对应 ai_config 表，BYO 模型配置）。
//!
//! 这是黄金示例：展示 domain → repository → cache → service 的完整分层，
//! 后续子系统照此模板实现。

use serde::{Deserialize, Serialize};

/// BYO 模型配置：用户自带的 LLM API Key / Base URL / Preamble。
/// 存储于 ai_config 表，按 user_id 隔离（每个用户一份）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: String,
    pub preamble: String,
}

impl AiConfig {
    /// 新建一份空配置（用户首次访问时作为默认值）。
    pub fn empty() -> Self {
        Self::default()
    }

    /// 判断是否为空配置（三个字段全为空字符串）。
    pub fn is_empty(&self) -> bool {
        self.api_key.trim().is_empty()
            && self.base_url.trim().is_empty()
            && self.preamble.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_empty() {
        let cfg = AiConfig::empty();
        assert!(cfg.is_empty());
    }

    #[test]
    fn config_with_api_key_is_not_empty() {
        let cfg = AiConfig {
            api_key: "sk-xxx".into(),
            ..AiConfig::empty()
        };
        assert!(!cfg.is_empty());
    }

    #[test]
    fn config_with_whitespace_only_is_empty() {
        let cfg = AiConfig {
            api_key: "   ".into(),
            base_url: "\t\n".into(),
            preamble: "".into(),
        };
        assert!(cfg.is_empty(), "纯空白应视为空");
    }

    #[test]
    fn config_with_base_url_is_not_empty() {
        let cfg = AiConfig {
            base_url: "https://api.example.com".into(),
            ..AiConfig::empty()
        };
        assert!(!cfg.is_empty());
    }

    #[test]
    fn config_equality() {
        let a = AiConfig {
            api_key: "k".into(),
            base_url: "u".into(),
            preamble: "p".into(),
        };
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, AiConfig::empty());
    }
}
