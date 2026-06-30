//! 平台模型：管理员在服务端 env 配置的可选模型名。
//!
//! 平台模型走平台网关（ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL / ANTHROPIC_MODEL），
//! 凭证由管理员持有，按 Token 消耗以精粹结算。模型名由管理员通过 `PLATFORM_MODELS`
//! 环境变量（逗号分隔）提供，用户在选手编辑页只能从中选择，不能手填。

use super::AuthUser;
use super::response::ApiResponse;

/// 列出管理员配置的平台可选模型名（仅返回公开的模型名清单）。
pub async fn list_platform_models(_auth: AuthUser) -> ApiResponse<Vec<String>> {
    let models = std::env::var("PLATFORM_MODELS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    ApiResponse::ok(models)
}
