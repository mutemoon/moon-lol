//! 凭证类型与纯解析函数。
//!
//! 桌面端与云端各自的 [`CredentialResolver`](crate::resolver::CredentialResolver)
//! 负责把各自存储里的供应商解析成 [`ProviderCredentials`]，再交给纯函数
//! [`resolve_credentials`] 统一判定走供应商还是平台网关回退。

use serde::{Deserialize, Serialize};

/// 场景中的单个 agent 定义（前端契约结构，桌面 / 云端共用）。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub id: String,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    /// 选手指定的模型名；选平台模型时缺省回退 env 默认模型。
    #[serde(default)]
    pub model: Option<String>,
    /// 选手绑定的模型供应商 id；为空表示选「平台模型」，走平台网关 env。
    /// 统一用字符串承载（桌面端 providers.json 的 id 是字符串，云端是 Uuid 序列化成字符串）。
    #[serde(default)]
    pub provider_id: Option<String>,
}

/// 模型具体配置（模型 ID/名称与最大上下文 token 数）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub name: String,
    pub max_tokens: u32,
}

/// 平台网关 env 凭证（无供应商时的回退）。
#[derive(Debug, Clone)]
pub struct PlatformEnv {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl PlatformEnv {
    /// 从 `ANTHROPIC_API_KEY` / `ANTHROPIC_BASE_URL` / `ANTHROPIC_MODEL` 读取。
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            base_url: std::env::var("ANTHROPIC_BASE_URL").unwrap_or_default(),
            model: std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "deepseek-v4-flash".to_string()),
        }
    }
}

/// 供应商凭证视图（由两端 resolver 从各自存储产出）。
#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    pub api_key: String,
    pub base_url: String,
    pub api_format: String,
    pub max_tokens: Option<u32>,
}

/// 解析后的单 agent LLM 凭证。
#[derive(Debug, Clone)]
pub struct ResolvedCredentials {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: Option<u32>,
}

/// 纯解析：有供应商且 api_key 非空走供应商，否则走平台 env；env api_key 空返回 None。
///
/// `agent.model` 缺省时回退 `env.model`。
pub fn resolve_credentials(
    agent: &AgentConfig,
    provider: Option<ProviderCredentials>,
    env: &PlatformEnv,
) -> Option<ResolvedCredentials> {
    let model = agent.model.clone().unwrap_or_else(|| env.model.clone());

    match provider {
        Some(p) if !p.api_key.trim().is_empty() => Some(ResolvedCredentials {
            api_key: p.api_key,
            base_url: p.base_url,
            model,
            max_tokens: p.max_tokens,
        }),
        _ => {
            if env.api_key.is_empty() {
                return None;
            }
            Some(ResolvedCredentials {
                api_key: env.api_key.clone(),
                base_url: env.base_url.clone(),
                model,
                max_tokens: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> PlatformEnv {
        PlatformEnv {
            api_key: "env-key".into(),
            base_url: "https://env".into(),
            model: "env-model".into(),
        }
    }

    fn agent(model: Option<&str>, provider_id: Option<&str>) -> AgentConfig {
        AgentConfig {
            id: "a1".into(),
            champion: "Riven".into(),
            team: "Order".into(),
            prompt: "".into(),
            model: model.map(String::from),
            provider_id: provider_id.map(String::from),
        }
    }

    #[test]
    fn provider_takes_priority_when_api_key_present() {
        let p = ProviderCredentials {
            api_key: "sk-prov".into(),
            base_url: "https://prov".into(),
            api_format: "anthropic".into(),
            max_tokens: Some(4096),
        };
        let r = resolve_credentials(&agent(Some("m"), Some("pid")), Some(p), &env()).unwrap();
        assert_eq!(r.api_key, "sk-prov");
        assert_eq!(r.base_url, "https://prov");
        assert_eq!(r.model, "m");
        assert_eq!(r.max_tokens, Some(4096));
    }

    #[test]
    fn empty_provider_api_key_falls_back_to_env() {
        let p = ProviderCredentials {
            api_key: "  ".into(),
            base_url: "https://prov".into(),
            api_format: "anthropic".into(),
            max_tokens: None,
        };
        let r = resolve_credentials(&agent(None, Some("pid")), Some(p), &env()).unwrap();
        assert_eq!(r.api_key, "env-key");
        assert_eq!(r.model, "env-model"); // agent.model 缺省回退 env
    }

    #[test]
    fn no_provider_uses_env() {
        let r = resolve_credentials(&agent(None, None), None, &env()).unwrap();
        assert_eq!(r.api_key, "env-key");
    }

    #[test]
    fn empty_env_api_key_returns_none() {
        let empty = PlatformEnv {
            api_key: "".into(),
            base_url: "".into(),
            model: "m".into(),
        };
        assert!(resolve_credentials(&agent(None, None), None, &empty).is_none());
    }
}
