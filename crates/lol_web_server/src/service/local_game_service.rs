//! LocalGame 子系统的 service 层（本地对局进程托管）。
//!
//! 进程托管（端口池 + spawn/kill + 进程表）委托共享 crate `lol_game_process_manager`；
//! 本层在其上叠加对局体系：match 记录、状态机、match_supervisor 胜负判定。
//! 进程状态不持久化——服务重启后所有运行中的本地对局视为 crashed。

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use lol_agent_runtime::AgentConfig;
use lol_game_process_manager::{
    GameProcessManager, ManagedProcess, ManagerError, ProcessLauncher, StartGameInput,
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::match_::{MatchForm, MatchStatus};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::match_repo::{MatchInput, MatchRepo};
use crate::service::match_service::MatchService;
use crate::service::model_provider_service::ModelProviderService;
use crate::service::{agent_orchestrator, match_supervisor};

/// 本地对局启动输入。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalStartInput {
    pub mode: String,
    pub scenario_id: Option<Uuid>,
    pub win_condition: Option<serde_json::Value>,
    /// 场景 agent 阵容：非空时启动 AI 决策环（接入进程内 rmcp 工具层）。
    #[serde(default)]
    pub scenario_agents: Vec<AgentConfig>,
}

#[async_trait]
pub trait LocalGameService: Send + Sync {
    /// 启动本地对局：创建 match 记录 + 启动 Bevy 进程（端口池分配）。
    /// 返回 (match_id, port)。
    async fn start(&self, owner_id: i32, input: LocalStartInput) -> ServiceResult<(Uuid, i32)>;

    /// 停止本地对局：杀进程 + 释放端口 + 更新 match 状态。
    async fn stop(&self, owner_id: i32, match_id: Uuid) -> ServiceResult<()>;

    /// 列出当前托管的进程。
    async fn list_processes(&self) -> ServiceResult<Vec<ManagedProcess>>;

    /// 清理已退出的进程（状态为 Stopped/Crashed 的释放端口）。
    async fn cleanup(&self) -> ServiceResult<usize>;
}

pub struct LocalGameServiceImpl {
    pub match_repo: Arc<dyn MatchRepo>,
    pub manager: Arc<GameProcessManager>,
    pub match_service: Arc<dyn MatchService>,
    pub model_provider_service: Arc<dyn ModelProviderService>,
}

impl LocalGameServiceImpl {
    pub fn new(
        match_repo: Arc<dyn MatchRepo>,
        launcher: Arc<dyn ProcessLauncher>,
        match_service: Arc<dyn MatchService>,
        model_provider_service: Arc<dyn ModelProviderService>,
    ) -> Self {
        // 云端 AI 决策环在 start 内按 owner_id 显式 spawn（需每请求的 owner_id 解析凭证），
        // 故 manager 不注入 agent_runner。
        Self {
            match_repo,
            manager: Arc::new(GameProcessManager::new(launcher, None)),
            match_service,
            model_provider_service,
        }
    }
}

#[async_trait]
impl LocalGameService for LocalGameServiceImpl {
    async fn start(&self, owner_id: i32, input: LocalStartInput) -> ServiceResult<(Uuid, i32)> {
        let mode = input.mode.trim();
        if mode.is_empty() || mode.len() > 64 {
            return Err(ServiceError::Validation(
                "mode 不能为空且不超过 64 字符".into(),
            ));
        }

        // 1. 创建 match 记录
        let match_record = self
            .match_repo
            .insert(
                owner_id,
                &MatchInput {
                    form: MatchForm::Local,
                    room_id: None,
                    mode: mode.to_string(),
                    scenario_id: input.scenario_id,
                    win_condition: input.win_condition,
                },
            )
            .await?;

        // 2. 启动 Bevy 进程（端口池分配 + spawn + 进程表登记），委托 GameProcessManager
        let start_input = StartGameInput {
            id: match_record.id,
            spawn: cloud_spawn_request(),
            scenario_agents: Vec::new(), // 云端 AI 环在下方按 owner_id 显式 spawn
        };
        let (_proc_id, port) = match self.manager.start(start_input).await {
            Ok(pair) => pair,
            Err(e) => {
                // 启动失败：回滚 match 状态为 aborted
                let _ = self
                    .match_repo
                    .update_abort(
                        match_record.id,
                        MatchStatus::Pending,
                        "process_launch_failed",
                    )
                    .await;
                return Err(map_manager_error(e));
            }
        };

        // 3. 更新 match 端口 + 状态为 running
        self.match_repo
            .update_ports(match_record.id, Some(port), Some(port))
            .await?;
        self.match_repo
            .update_status(match_record.id, MatchStatus::Pending, MatchStatus::Running)
            .await?;

        // 4. 启动 match supervisor：订阅 Bevy WS，套用 SOLO 胜负规则并落库。
        let match_id = match_record.id;
        let match_service = self.match_service.clone();
        tokio::spawn(async move {
            match_supervisor::run_supervisor(match_id, port, match_service).await;
        });

        // 5. 启动 AI Agent 决策环：仅当配置了场景 agent 时接入进程内 rmcp 工具层。
        //    无凭据时编排环内部静默跳过。
        let scenario_agents = input.scenario_agents.clone();
        if !scenario_agents.is_empty() {
            let providers = self.model_provider_service.clone();
            tokio::spawn(async move {
                agent_orchestrator::run_agent_orchestrator(
                    port,
                    scenario_agents,
                    owner_id,
                    providers,
                )
                .await;
            });
        }

        Ok((match_record.id, port))
    }

    async fn stop(&self, owner_id: i32, match_id: Uuid) -> ServiceResult<()> {
        // 校验 match 存在且属于 owner
        let m = self
            .match_repo
            .find_by_id(match_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if m.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }

        // 取端口
        let port = m
            .bevy_port
            .ok_or_else(|| ServiceError::Conflict("该对局未分配端口，可能已停止".into()))?;

        // 杀进程 + 释放端口 + 移除托管记录
        self.manager
            .stop_by_port(port)
            .await
            .map_err(map_manager_error)?;

        // 更新 match 状态
        let _ = self
            .match_repo
            .update_abort(match_id, m.status, "manual_stop")
            .await;

        Ok(())
    }

    async fn list_processes(&self) -> ServiceResult<Vec<ManagedProcess>> {
        self.manager
            .list_processes()
            .await
            .map_err(map_manager_error)
    }

    async fn cleanup(&self) -> ServiceResult<usize> {
        self.manager.cleanup().await.map_err(map_manager_error)
    }
}

/// 云端 Bevy 进程 spawn 配置：`cargo run --bin moon_lol --` + headless。
/// `CommandProcessLauncher` 据此（或 env 二进制路径）构建命令。
fn cloud_spawn_request() -> lol_client::launch::BevySpawnRequest {
    lol_client::launch::BevySpawnRequest {
        program: "cargo".into(),
        prefix_args: vec!["run".into(), "--bin".into(), "moon_lol".into(), "--".into()],
        port: 0, // 由 manager 覆盖
        game_config: lol_client::launch::BevyGameConfig {
            headless: true,
            ..Default::default()
        },
        cwd: None,
        rust_log: None,
        log_db: None,
    }
}

/// 把进程托管层错误映射为 service 层错误。
fn map_manager_error(e: ManagerError) -> ServiceError {
    use ManagerError as E;
    match e {
        E::NotFound => ServiceError::NotFound,
        E::Conflict(msg) => ServiceError::Conflict(msg),
        E::Validation(msg) => ServiceError::Validation(msg),
        E::Internal(msg) => ServiceError::Internal(msg),
    }
}

/// 命令行进程启动实现（生产用）。
///
/// spawn 命令构建复用 `lol_client::launch::build_command`。程序来源：
/// - 设了 `MOON_LOL_BINARY` env → 直接跑该二进制（release 部署），prefix_args 为空；
/// - 否则回退 `cargo run --bin moon_lol --`（dev / CI）。
pub struct CommandProcessLauncher {
    /// 端口对子进程的映射，用于手动 stop 时 kill 对应进程。
    pub processes: Mutex<HashMap<i32, tokio::process::Child>>,
}

impl Default for CommandProcessLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessLauncher {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }

    /// 据环境决定程序与前缀：env 二进制优先，否则 cargo run。
    fn program_and_prefix() -> (String, Vec<String>) {
        match std::env::var("MOON_LOL_BINARY") {
            Ok(path) if !path.trim().is_empty() => (path, Vec::new()),
            _ => (
                "cargo".into(),
                vec!["run".into(), "--bin".into(), "moon_lol".into(), "--".into()],
            ),
        }
    }
}

#[async_trait]
impl ProcessLauncher for CommandProcessLauncher {
    async fn launch(
        &self,
        port: i32,
        req: &lol_client::launch::BevySpawnRequest,
    ) -> Result<(), ManagerError> {
        let (program, prefix_args) = Self::program_and_prefix();
        let mut req = req.clone();
        req.program = program;
        req.prefix_args = prefix_args;
        req.port = port as u16;

        let child = tokio::process::Command::from(lol_client::launch::build_command(&req))
            .spawn()
            .map_err(|e| ManagerError::Internal(format!("启动对局进程失败: {e}")))?;

        let mut procs = self.processes.lock().await;
        procs.insert(port, child);
        Ok(())
    }

    async fn kill(&self, port: i32) -> Result<(), ManagerError> {
        let mut procs = self.processes.lock().await;
        if let Some(mut child) = procs.remove(&port) {
            let _ = child.kill().await;
        }
        Ok(())
    }
}
