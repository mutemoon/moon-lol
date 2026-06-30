//! Rig Agent 决策环：web server 侧对运行中本地对局的 AI 编排。
//!
//! 纯编排逻辑（状态机、rmcp 工具层注入、暂停→观测→思考→恢复循环）由
//! `lol_agent_runtime::run_orchestrator` 承载，与本 crate 的存储解耦。
//! 本文件仅提供云端凭证解析（`ModelProviderService` + DB）与空副作用出口。
//!
//! LLM 凭据优先按每个选手的 `provider_id` 从 `ModelProviderService` 解析
//! （api_key / base_url）；选手选「平台模型」时（无 provider_id）走管理员在
//! 服务端 env 配置的平台网关（ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL /
//! ANTHROPIC_MODEL），按 Token 消耗以精粹结算。

use std::sync::Arc;

use async_trait::async_trait;
use lol_agent_runtime::{
    AgentConfig, CredentialResolver, NoopSink, PlatformEnv, ProviderCredentials,
    ResolvedCredentials, resolve_credentials, run_orchestrator,
};
use lol_client::{GameClient, start_ws_client};
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain::model_provider::ModelProvider;
use crate::service::model_provider_service::ModelProviderService;

/// 云端凭证解析：按 `provider_id`（Uuid 字符串）从 `ModelProviderService` 取供应商。
pub struct WebCredentialResolver {
    owner_id: i32,
    providers: Arc<dyn ModelProviderService>,
}

#[async_trait]
impl CredentialResolver for WebCredentialResolver {
    async fn resolve(&self, agent: &AgentConfig, env: &PlatformEnv) -> Option<ResolvedCredentials> {
        let provider: Option<ModelProvider> = match agent.provider_id.as_deref() {
            Some(pid) => match pid.parse::<Uuid>() {
                Ok(uuid) => match self
                    .providers
                    .resolve_for_runtime(uuid, self.owner_id)
                    .await
                {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("[Web Orchestrator] 解析供应商 {} 失败: {}", pid, e);
                        None
                    }
                },
                Err(e) => {
                    warn!("[Web Orchestrator] provider_id {} 非 Uuid: {}", pid, e);
                    None
                }
            },
            None => None,
        };

        let creds = provider.map(|p| {
            if p.api_format != "anthropic" {
                warn!(
                    "[Web Orchestrator] 供应商 {} 的 api_format={} 暂仅支持 anthropic 兼容端点",
                    p.name, p.api_format
                );
            }
            ProviderCredentials {
                api_key: p.api_key,
                base_url: p.base_url,
                api_format: p.api_format,
            }
        });

        resolve_credentials(agent, creds, env)
    }
}

/// 启动 AI Agent 决策环：连接 ws_port 的 Bevy 进程，注入 rmcp tools 并循环决策。
///
/// 无 LLM 凭据或无 agent 配置时静默返回（不报错），与 Tauri 行为一致。
/// `agents` 通常来自场景定义（`Scenario::agents` JSON）。
/// `owner_id` 用于按 provider_id 解析该用户的供应商凭证。
pub async fn run_agent_orchestrator(
    ws_port: i32,
    agents: Vec<AgentConfig>,
    owner_id: i32,
    providers: Arc<dyn ModelProviderService>,
) {
    info!(
        "[Web Orchestrator] 启动 AI Agent 后台生命周期循环 (ws={})",
        ws_port
    );

    let session = match start_ws_client(ws_port as u16, None).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[Web Orchestrator] ws={} 连接 Bevy WS 失败: {}", ws_port, e);
            return;
        }
    };
    let client = GameClient::new(session);

    let env = PlatformEnv::from_env();
    let resolver = Arc::new(WebCredentialResolver {
        owner_id,
        providers,
    });
    let sink = Arc::new(NoopSink);

    run_orchestrator(client, agents, resolver, env, sink).await;
    info!("[Web Orchestrator] ws={} 决策环退出", ws_port);
}
