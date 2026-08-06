//! Model Provider 路由：列表 / 创建 / 更新 / 删除。

use axum::Json;
use axum::extract::{Path, State};
use lol_web_protocol::model_provider::{
    ModelProvider, ModelProviderInput, TestModelProviderInput, TestModelProviderResponse,
};
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn test_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<TestModelProviderInput>,
) -> ApiResponse<TestModelProviderResponse> {
    let api_key = if let Some(key) = input.api_key.filter(|k| !k.is_empty()) {
        key
    } else if let Some(provider_id) = input.provider_id {
        match s
            .model_provider_service
            .resolve_for_runtime(provider_id, auth.user_id)
            .await
        {
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

    match lol_agent_runtime::test_model_connection(
        &api_key,
        &input.base_url,
        &input.model,
        input.max_tokens,
    )
    .await
    {
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
) -> ApiResponse<Vec<ModelProvider>> {
    match s.model_provider_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn create_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<ModelProviderInput>,
) -> ApiResponse<ModelProvider> {
    let domain_input = input.into();
    match s
        .model_provider_service
        .create(auth.user_id, domain_input)
        .await
    {
        Ok(dto) => ApiResponse::ok(dto.into()),
        Err(e) => api_error(e),
    }
}

pub async fn update_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<ModelProviderInput>,
) -> ApiResponse<()> {
    let domain_input = input.into();
    match s
        .model_provider_service
        .update(auth.user_id, id, domain_input)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn delete_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.model_provider_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}
