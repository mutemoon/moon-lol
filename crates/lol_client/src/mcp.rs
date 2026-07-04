use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::service::ServerSink;
use rmcp::{ClientHandler, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::io::duplex;

use crate::action::Action;
use crate::game_client::GameClient;
use crate::protocol::WsResponse;

/// `observe` 工具入参。
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ObserveArgs {
    /// 观测的英雄实体 ID
    pub entity_id: u64,
}

/// `move_to` 工具入参。
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MoveToArgs {
    /// 操作的英雄实体 ID
    pub entity_id: u64,
    /// 目标位置 X 坐标
    pub x: f32,
    /// 目标位置 Y 坐标
    pub y: f32,
}

/// `attack` 工具入参。
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AttackArgs {
    /// 操作的英雄实体 ID
    pub entity_id: u64,
    /// 目标实体 ID
    pub target_id: u64,
}

/// `stop` 工具入参。
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StopArgs {
    /// 操作的英雄实体 ID
    pub entity_id: u64,
}

/// `cast_skill` 工具入参。
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CastSkillArgs {
    /// 操作的英雄实体 ID
    pub entity_id: u64,
    /// 技能索引 (0: Q, 1: W, 2: E, 3: R)
    pub index: usize,
    /// 目标位置 X 坐标
    pub x: f32,
    /// 目标位置 Y 坐标
    pub y: f32,
}

/// `level_up_skill` 工具入参。
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LevelUpSkillArgs {
    /// 操作的英雄实体 ID
    pub entity_id: u64,
    /// 技能索引 (0: Q, 1: W, 2: E, 3: R)
    pub index: usize,
}

/// MCP 工具层：仅暴露 observe / action，委托 [`GameClient`]。
///
/// 调试 / 作弊类指令（上帝模式、冷却开关等）不进入此层，避免 agent 越权。
#[derive(Clone)]
#[allow(dead_code)]
pub struct GameToolServer {
    client: GameClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GameToolServer {
    pub fn new(client: GameClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    /// 获取指定英雄的局势观测数据
    #[tool(
        name = "observe",
        description = "获取指定英雄实体的局势观测数据，返回高密度自然语言中文描述"
    )]
    async fn observe(&self, Parameters(args): Parameters<ObserveArgs>) -> String {
        match self.client.observe(args.entity_id, false).await {
            Ok(resp) => {
                if resp.ok {
                    match resp.data {
                        Some(serde_json::Value::String(s)) => s,
                        Some(data) => serde_json::to_string_pretty(&data)
                            .unwrap_or_else(|_| "数据格式化失败".to_string()),
                        None => "没有获取到观测数据".to_string(),
                    }
                } else {
                    format!(
                        "错误: {}",
                        resp.error.unwrap_or_else(|| "未知错误".to_string())
                    )
                }
            }
            Err(e) => format!("错误: {}", e),
        }
    }

    /// 移动指定英雄实体到目标坐标 [x, y]
    #[tool(name = "move_to", description = "移动指定英雄实体到目标坐标 [x, y]")]
    async fn move_to(&self, Parameters(args): Parameters<MoveToArgs>) -> String {
        let action = Action::Move([args.x, args.y]);
        match self.client.action(args.entity_id, action).await {
            Ok(resp) => format_response(resp),
            Err(e) => format!("错误: {}", e),
        }
    }

    /// 令指定英雄实体攻击目标实体
    #[tool(name = "attack", description = "令指定英雄实体攻击目标实体")]
    async fn attack(&self, Parameters(args): Parameters<AttackArgs>) -> String {
        let action = Action::Attack(args.target_id);
        match self.client.action(args.entity_id, action).await {
            Ok(resp) => format_response(resp),
            Err(e) => format!("错误: {}", e),
        }
    }

    /// 令指定英雄实体停止当前动作
    #[tool(name = "stop", description = "令指定英雄实体停止当前动作")]
    async fn stop(&self, Parameters(args): Parameters<StopArgs>) -> String {
        let action = Action::Stop;
        match self.client.action(args.entity_id, action).await {
            Ok(resp) => format_response(resp),
            Err(e) => format!("错误: {}", e),
        }
    }

    /// 令指定英雄实体对目标坐标 [x, y] 释放指定索引的技能
    #[tool(
        name = "cast_skill",
        description = "令指定英雄实体对目标坐标 [x, y] 释放指定索引的技能 (0: Q, 1: W, 2: E, 3: R)"
    )]
    async fn cast_skill(&self, Parameters(args): Parameters<CastSkillArgs>) -> String {
        let action = Action::Skill {
            index: args.index,
            point: [args.x, args.y],
        };
        match self.client.action(args.entity_id, action).await {
            Ok(resp) => format_response(resp),
            Err(e) => format!("错误: {}", e),
        }
    }

    /// 升级指定英雄实体的指定索引技能
    #[tool(
        name = "level_up_skill",
        description = "升级指定英雄实体的指定索引技能 (0: Q, 1: W, 2: E, 3: R)"
    )]
    async fn level_up_skill(&self, Parameters(args): Parameters<LevelUpSkillArgs>) -> String {
        let action = Action::SkillLevelUp(args.index);
        match self.client.action(args.entity_id, action).await {
            Ok(resp) => format_response(resp),
            Err(e) => format!("错误: {}", e),
        }
    }
}

#[tool_handler]
impl ServerHandler for GameToolServer {}

fn format_response(resp: WsResponse) -> String {
    if resp.ok {
        match resp.data {
            Some(data) => {
                serde_json::to_string_pretty(&data).unwrap_or_else(|_| "指令执行成功".to_string())
            }
            None => "指令执行成功".to_string(),
        }
    } else {
        format!(
            "错误: {}",
            resp.error.unwrap_or_else(|| "未知错误".to_string())
        )
    }
}

/// 进程内 rmcp 客户端，无端口无 stdio。
#[derive(Debug, Clone, Default)]
struct GameClientHandler;

impl ClientHandler for GameClientHandler {}

/// 在进程内用 `tokio::io::duplex` 建立 rmcp client/server 对，
/// 返回 rig agent 注入所需的 `(tools, peer)`。
///
/// server 与 client 的运行时服务均 spawn 为 tokio task 常驻，
/// 只要返回的 `peer` 被使用即保持连接活跃。
pub async fn serve_inprocess(
    client: GameClient,
) -> Result<(Vec<rmcp::model::Tool>, ServerSink), String> {
    let (server_transport, client_transport) = dupex_pair();

    let server = GameToolServer::new(client);
    tokio::spawn(async move {
        if let Ok(svc) = server.serve(server_transport).await {
            let _ = svc.waiting().await;
        }
    });

    let client_service = GameClientHandler
        .serve(client_transport)
        .await
        .map_err(|e| e.to_string())?;
    let tools = client_service
        .list_tools(None)
        .await
        .map_err(|e| e.to_string())?
        .tools;
    let peer = client_service.peer().clone();

    // 保持客户端运行时服务常驻，供 peer 后续 call_tool 使用
    tokio::spawn(async move {
        let _ = client_service.waiting().await;
    });

    Ok((tools, peer))
}

fn dupex_pair() -> (tokio::io::DuplexStream, tokio::io::DuplexStream) {
    duplex(4096)
}
