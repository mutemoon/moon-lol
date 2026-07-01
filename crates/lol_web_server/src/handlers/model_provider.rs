//! Model Provider 路由：列表 / 创建 / 更新 / 删除。

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

#[derive(serde::Deserialize)]
pub struct TestModelProviderInput {
    pub provider_id: Option<Uuid>,
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_format: String,
    pub model: String,
    pub max_tokens: Option<u32>,
}

#[derive(serde::Serialize)]
pub struct TestModelProviderResponse {
    pub success: bool,
    pub message: String,
}

pub async fn test_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<TestModelProviderInput>,
) -> ApiResponse<TestModelProviderResponse> {
    // 1. 获取并解密 API 密钥
    let api_key = if let Some(key) = input.api_key.filter(|k| !k.is_empty()) {
        key
    } else if let Some(provider_id) = input.provider_id {
        match s.model_provider_service.resolve_for_runtime(provider_id, auth.user_id).await {
            Ok(Some(provider)) => provider.api_key,
            Ok(None) => {
                return ApiResponse::ok(TestModelProviderResponse {
                    success: false,
                    message: "未找到指定的模型供应商".into(),
                });
            }
            Err(e) => {
                return ApiResponse::ok(TestModelProviderResponse {
                    success: false,
                    message: format!("解析凭证失败: {}", e),
                });
            }
        }
    } else {
        "".to_string()
    };

    // 2. 调用 lol_agent_runtime 中的测试方法进行连接测试
    match lol_agent_runtime::test_model_connection(&api_key, &input.base_url, &input.model, input.max_tokens).await {
        Ok(reply) => ApiResponse::ok(TestModelProviderResponse {
            success: true,
            message: reply,
        }),
        Err(e) => ApiResponse::ok(TestModelProviderResponse {
            success: false,
            message: e,
        }),
    }
}

pub async fn list_model_providers(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::model_provider::ModelProviderDto>> {
    match s.model_provider_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn create_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<crate::domain::model_provider::ModelProviderInput>,
) -> ApiResponse<crate::domain::model_provider::ModelProviderDto> {
    match s.model_provider_service.create(auth.user_id, input).await {
        Ok(dto) => ApiResponse::ok(dto),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn update_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<crate::domain::model_provider::ModelProviderInput>,
) -> ApiResponse<()> {
    match s
        .model_provider_service
        .update(auth.user_id, id, input)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn delete_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.model_provider_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
