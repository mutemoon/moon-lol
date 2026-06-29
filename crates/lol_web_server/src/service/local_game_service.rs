//! LocalGame 子系统的 service 层（本地对局进程托管）。
//!
//! 编排：端口池（内存 HashSet）+ 进程抽象（ProcessLauncher trait）+ MatchRepo（记录对局）。
//! 进程状态不持久化——服务重启后所有运行中的本地对局视为 crashed。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::local_game::{ManagedProcess, ProcessStatus, allocate_port, is_valid_port};
use crate::domain::match_::{MatchForm, MatchStatus};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::match_repo::{MatchInput, MatchRepo};
use crate::service::agent_orchestrator::SceneAgentConfig;
use crate::service::match_service::MatchService;
use crate::service::{agent_orchestrator, match_supervisor};

/// 进程启动抽象（可 mock，真实 impl 用 tokio::process::Command）。
#[async_trait]
pub trait ProcessLauncher: Send + Sync {
    /// 启动一个 Bevy 子进程，监听指定端口。返回 join handle 或占位。
    /// 失败返回 Err。
    async fn launch(&self, port: i32, match_id: Uuid) -> ServiceResult<()>;

    /// 停止指定端口的子进程。
    async fn kill(&self, port: i32) -> ServiceResult<()>;
}

/// 本地对局启动输入。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalStartInput {
    pub mode: String,
    pub scenario_id: Option<Uuid>,
    pub win_condition: Option<serde_json::Value>,
    /// 场景 agent 阵容：非空时启动 AI 决策环（接入进程内 rmcp 工具层）。
    #[serde(default)]
    pub scenario_agents: Vec<SceneAgentConfig>,
}

#[async_trait]
pub trait LocalGameService: Send + Sync {
    /// 启动本地对局：分配端口 + 创建 match 记录 + 启动 Bevy 进程。
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
    pub launcher: Arc<dyn ProcessLauncher>,
    pub match_service: Arc<dyn MatchService>,
    /// 端口池 + 进程表（内存状态）。
    pub state: Arc<Mutex<LocalGameState>>,
}

/// 内存状态：已用端口 + 托管进程列表。
#[derive(Default)]
pub struct LocalGameState {
    pub used_ports: HashSet<i32>,
    pub processes: Vec<ManagedProcess>,
}

impl LocalGameState {
    /// 标记端口为已用。
    pub fn acquire(&mut self, port: i32) {
        self.used_ports.insert(port);
    }

    /// 释放端口。
    pub fn release(&mut self, port: i32) {
        self.used_ports.remove(&port);
    }

    /// 添加托管进程。
    pub fn add_process(&mut self, proc: ManagedProcess) {
        self.processes.push(proc);
    }

    /// 按端口移除进程，返回是否找到。
    pub fn remove_process_by_port(&mut self, port: i32) -> bool {
        let before = self.processes.len();
        self.processes.retain(|p| p.port != port);
        self.processes.len() < before
    }

    /// 移除所有 Stopped/Crashed 进程并释放对应端口，返回清理数量。
    pub fn cleanup_terminated(&mut self) -> usize {
        let to_remove: Vec<i32> = self
            .processes
            .iter()
            .filter(|p| matches!(p.status, ProcessStatus::Stopped | ProcessStatus::Crashed))
            .map(|p| p.port)
            .collect();
        for port in &to_remove {
            self.release(*port);
        }
        self.processes.retain(|p| !to_remove.contains(&p.port));
        to_remove.len()
    }
}

impl LocalGameServiceImpl {
    pub fn new(
        match_repo: Arc<dyn MatchRepo>,
        launcher: Arc<dyn ProcessLauncher>,
        match_service: Arc<dyn MatchService>,
    ) -> Self {
        Self {
            match_repo,
            launcher,
            match_service,
            state: Arc::new(Mutex::new(LocalGameState::default())),
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

        // 1. 分配端口
        let port = {
            let state = self.state.lock().await;
            allocate_port(&state.used_ports)
                .ok_or(ServiceError::Conflict("端口池已满，无法启动新对局".into()))?
        };

        // 2. 创建 match 记录
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

        // 3. 启动 Bevy 进程
        if let Err(e) = self.launcher.launch(port, match_record.id).await {
            // 启动失败：释放端口，回滚 match 状态为 aborted
            let mut state = self.state.lock().await;
            state.release(port);
            // 尝试标记 match 为 aborted（忽略错误，因为进程都没起来）
            let _ = self
                .match_repo
                .update_abort(
                    match_record.id,
                    MatchStatus::Pending,
                    "process_launch_failed",
                )
                .await;
            return Err(e);
        }

        // 4. 更新 match 端口 + 状态为 running
        self.match_repo
            .update_ports(match_record.id, Some(port), Some(port))
            .await?;
        self.match_repo
            .update_status(match_record.id, MatchStatus::Pending, MatchStatus::Running)
            .await?;

        // 5. 记录托管进程
        {
            let mut state = self.state.lock().await;
            state.acquire(port);
            state.add_process(ManagedProcess {
                match_id: match_record.id,
                port,
                status: ProcessStatus::Running,
            });
        }

        // 6. 启动 match supervisor：订阅 Bevy WS，套用 SOLO 胜负规则并落库。
        let match_id = match_record.id;
        let match_service = self.match_service.clone();
        tokio::spawn(async move {
            match_supervisor::run_supervisor(match_id, port, match_service).await;
        });

        // 7. 启动 AI Agent 决策环：仅当配置了场景 agent 且存在 LLM 凭据时才接入
        //    进程内 rmcp 工具层（observe + action）。无凭据时编排环内部静默跳过。
        let scenario_agents = input.scenario_agents.clone();
        if !scenario_agents.is_empty() {
            tokio::spawn(async move {
                agent_orchestrator::run_agent_orchestrator(port, scenario_agents).await;
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
        if !is_valid_port(port) {
            return Err(ServiceError::Internal(format!("无效端口: {port}")));
        }

        // 杀进程
        self.launcher.kill(port).await?;

        // 释放端口 + 移除托管记录
        {
            let mut state = self.state.lock().await;
            state.release(port);
            state.remove_process_by_port(port);
        }

        // 更新 match 状态
        let _ = self
            .match_repo
            .update_abort(match_id, m.status, "manual_stop")
            .await;

        Ok(())
    }

    async fn list_processes(&self) -> ServiceResult<Vec<ManagedProcess>> {
        let state = self.state.lock().await;
        Ok(state.processes.clone())
    }

    async fn cleanup(&self) -> ServiceResult<usize> {
        let mut state = self.state.lock().await;
        Ok(state.cleanup_terminated())
    }
}

/// 命令行进程启动实现（生产用）。
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
}

#[async_trait]
impl ProcessLauncher for CommandProcessLauncher {
    async fn launch(&self, port: i32, _match_id: Uuid) -> ServiceResult<()> {
        let child = tokio::process::Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "moon_lol",
                "--",
                "--ws-port",
                &port.to_string(),
                "--headless",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| ServiceError::Internal(format!("启动对局进程失败: {e}")))?;

        let mut procs = self.processes.lock().await;
        procs.insert(port, child);
        Ok(())
    }

    async fn kill(&self, port: i32) -> ServiceResult<()> {
        let mut procs = self.processes.lock().await;
        if let Some(mut child) = procs.remove(&port) {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::local_game::ProcessStatus;
    use crate::domain::match_::{Match, MatchForm, MatchStatus};

    mock! {
        pub MatchRepo {}
        #[async_trait]
        impl MatchRepo for MatchRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Match>>;
            async fn list_by_owner(&self, owner_id: i32, limit: i64) -> RepoResult<Vec<Match>>;
            async fn list_by_status(&self, status: MatchStatus, limit: i64) -> RepoResult<Vec<Match>>;
            async fn insert(&self, owner_id: i32, input: &MatchInput) -> RepoResult<Match>;
            async fn update_status(&self, id: Uuid, from: MatchStatus, to: MatchStatus) -> RepoResult<()>;
            async fn update_result(&self, id: Uuid, winner: crate::domain::match_::Winner) -> RepoResult<()>;
            async fn update_abort(&self, id: Uuid, from: MatchStatus, reason: &str) -> RepoResult<()>;
            async fn update_ports(&self, id: Uuid, bevy_port: Option<i32>, ws_port: Option<i32>) -> RepoResult<()>;
        }
    }

    mock! {
        pub Launcher {}
        #[async_trait]
        impl ProcessLauncher for Launcher {
            async fn launch(&self, port: i32, match_id: Uuid) -> ServiceResult<()>;
            async fn kill(&self, port: i32) -> ServiceResult<()>;
        }
    }

    mock! {
        pub MatchSvc {}
        #[async_trait]
        impl MatchService for MatchSvc {
            async fn create(&self, owner_id: i32, input: MatchInput) -> ServiceResult<Match>;
            async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Match>;
            async fn list_mine(&self, owner_id: i32) -> ServiceResult<Vec<Match>>;
            async fn list_by_status(&self, status: MatchStatus) -> ServiceResult<Vec<Match>>;
            async fn start(&self, requester_id: i32, id: Uuid, bevy_port: i32, ws_port: i32) -> ServiceResult<Match>;
            async fn finish(&self, requester_id: i32, id: Uuid, winner: crate::domain::match_::Winner) -> ServiceResult<Match>;
            async fn finish_internal(&self, id: Uuid, winner: crate::domain::match_::Winner) -> ServiceResult<Match>;
            async fn abort(&self, requester_id: i32, id: Uuid, reason: String) -> ServiceResult<Match>;
            async fn append_event(&self, requester_id: i32, id: Uuid, event: crate::repository::match_repo::MatchEventInput) -> ServiceResult<crate::domain::match_::MatchEvent>;
            async fn append_event_internal(&self, id: Uuid, event: crate::repository::match_repo::MatchEventInput) -> ServiceResult<crate::domain::match_::MatchEvent>;
            async fn get_events(&self, requester_id: i32, id: Uuid, from_seq: i32, limit: i64) -> ServiceResult<Vec<crate::domain::match_::MatchEvent>>;
        }
    }

    fn sample_match(owner: i32, port: Option<i32>, status: MatchStatus) -> Match {
        Match {
            id: Uuid::new_v4(),
            form: MatchForm::Local,
            room_id: None,
            owner_id: owner,
            mode: "1v1".into(),
            status,
            bevy_port: port,
            winner_team: None,
            abort_reason: None,
        }
    }

    fn sample_input() -> LocalStartInput {
        LocalStartInput {
            mode: "1v1".into(),
            scenario_id: None,
            win_condition: None,
            scenario_agents: Vec::new(),
        }
    }

    fn build_service(repo: MockMatchRepo, launcher: MockLauncher) -> LocalGameServiceImpl {
        LocalGameServiceImpl {
            match_repo: Arc::new(repo),
            launcher: Arc::new(launcher),
            match_service: Arc::new(MockMatchSvc::new()),
            state: Arc::new(Mutex::new(LocalGameState::default())),
        }
    }

    #[tokio::test]
    async fn start_success_allocates_port_and_launches() {
        let fixed_match = sample_match(1, None, MatchStatus::Pending);
        let mid = fixed_match.id;
        let fixed_clone = fixed_match.clone();
        let mut repo = MockMatchRepo::new();
        repo.expect_insert()
            .returning(move |_, _| Ok(fixed_clone.clone()));
        repo.expect_update_ports()
            .with(eq(mid), always(), always())
            .returning(|_, _, _| Ok(()));
        repo.expect_update_status()
            .with(eq(mid), eq(MatchStatus::Pending), eq(MatchStatus::Running))
            .returning(|_, _, _| Ok(()));

        let mut launcher = MockLauncher::new();
        launcher
            .expect_launch()
            .with(eq(9100), eq(mid))
            .returning(|_, _| Ok(()));

        let svc = build_service(repo, launcher);
        let (match_id, port) = svc.start(1, sample_input()).await.unwrap();
        assert_eq!(port, 9100);
        assert_eq!(match_id, mid);

        // 端口已占用
        let state = svc.state.lock().await;
        assert!(state.used_ports.contains(&9100));
        assert_eq!(state.processes.len(), 1);
    }

    #[tokio::test]
    async fn start_validates_empty_mode() {
        let svc = build_service(MockMatchRepo::new(), MockLauncher::new());
        let mut input = sample_input();
        input.mode = "".into();
        let err = svc.start(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn start_launch_failure_releases_port_and_aborts() {
        let mut repo = MockMatchRepo::new();
        repo.expect_insert()
            .returning(|owner, _| Ok(sample_match(owner, None, MatchStatus::Pending)));
        repo.expect_update_abort().returning(|_, _, _| Ok(())); // 回滚
        repo.expect_update_ports().times(0); // 启动失败不应写端口
        repo.expect_update_status().times(0);

        let mut launcher = MockLauncher::new();
        launcher
            .expect_launch()
            .returning(|_, _| Err(ServiceError::Internal("launch failed".into())));

        let svc = build_service(repo, launcher);
        let err = svc.start(1, sample_input()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Internal(_)));

        // 端口应已释放
        let state = svc.state.lock().await;
        assert!(state.used_ports.is_empty());
    }

    #[tokio::test]
    async fn start_port_pool_full_rejected() {
        let svc = build_service(MockMatchRepo::new(), MockLauncher::new());
        // 填满端口池
        {
            let mut state = svc.state.lock().await;
            for p in
                crate::domain::local_game::PORT_POOL_START..crate::domain::local_game::PORT_POOL_END
            {
                state.used_ports.insert(p);
            }
        }
        let err = svc.start(1, sample_input()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn stop_success_kills_and_releases() {
        let m = sample_match(1, Some(9100), MatchStatus::Running);
        let mid = m.id;
        let port = 9100;
        let m_clone = m.clone();
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m_clone.clone())));
        repo.expect_update_abort().returning(|_, _, _| Ok(()));

        let mut launcher = MockLauncher::new();
        launcher.expect_kill().with(eq(port)).returning(|_| Ok(()));

        let svc = build_service(repo, launcher);
        // 预占端口
        {
            let mut state = svc.state.lock().await;
            state.acquire(port);
            state.add_process(ManagedProcess {
                match_id: mid,
                port,
                status: ProcessStatus::Running,
            });
        }

        svc.stop(1, mid).await.unwrap();

        let state = svc.state.lock().await;
        assert!(!state.used_ports.contains(&port));
        assert!(state.processes.is_empty());
    }

    #[tokio::test]
    async fn stop_non_owner_forbidden() {
        let m = sample_match(1, Some(9100), MatchStatus::Running);
        let m_clone = m.clone();
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m_clone.clone())));
        repo.expect_update_abort().times(0);

        let mut launcher = MockLauncher::new();
        launcher.expect_kill().times(0);

        let svc = build_service(repo, launcher);
        let err = svc.stop(2, m.id).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn stop_missing_match_not_found() {
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo, MockLauncher::new());
        let err = svc.stop(1, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn stop_match_without_port_conflict() {
        let m = sample_match(1, None, MatchStatus::Pending);
        let m_clone = m.clone();
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m_clone.clone())));
        let svc = build_service(repo, MockLauncher::new());
        let err = svc.stop(1, m.id).await.unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn list_processes_returns_current() {
        let svc = build_service(MockMatchRepo::new(), MockLauncher::new());
        {
            let mut state = svc.state.lock().await;
            state.add_process(ManagedProcess {
                match_id: Uuid::new_v4(),
                port: 9100,
                status: ProcessStatus::Running,
            });
        }
        let procs = svc.list_processes().await.unwrap();
        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0].port, 9100);
    }

    #[tokio::test]
    async fn cleanup_removes_terminated_and_releases_ports() {
        let svc = build_service(MockMatchRepo::new(), MockLauncher::new());
        {
            let mut state = svc.state.lock().await;
            state.acquire(9100);
            state.acquire(9101);
            state.add_process(ManagedProcess {
                match_id: Uuid::new_v4(),
                port: 9100,
                status: ProcessStatus::Stopped,
            });
            state.add_process(ManagedProcess {
                match_id: Uuid::new_v4(),
                port: 9101,
                status: ProcessStatus::Running,
            });
        }
        let cleaned = svc.cleanup().await.unwrap();
        assert_eq!(cleaned, 1);
        let state = svc.state.lock().await;
        assert!(!state.used_ports.contains(&9100)); // 已释放
        assert!(state.used_ports.contains(&9101)); // 运行中不释放
        assert_eq!(state.processes.len(), 1);
    }

    // ── LocalGameState 纯逻辑测试 ──

    #[test]
    fn state_acquire_release() {
        let mut s = LocalGameState::default();
        s.acquire(9100);
        assert!(s.used_ports.contains(&9100));
        s.release(9100);
        assert!(!s.used_ports.contains(&9100));
    }

    #[test]
    fn state_remove_process_by_port() {
        let mut s = LocalGameState::default();
        s.add_process(ManagedProcess {
            match_id: Uuid::new_v4(),
            port: 9100,
            status: ProcessStatus::Running,
        });
        assert!(s.remove_process_by_port(9100));
        assert!(!s.remove_process_by_port(9100)); // 第二次找不到
    }

    #[test]
    fn state_cleanup_terminated() {
        let mut s = LocalGameState::default();
        s.acquire(9100);
        s.acquire(9101);
        s.add_process(ManagedProcess {
            match_id: Uuid::new_v4(),
            port: 9100,
            status: ProcessStatus::Crashed,
        });
        s.add_process(ManagedProcess {
            match_id: Uuid::new_v4(),
            port: 9101,
            status: ProcessStatus::Running,
        });
        assert_eq!(s.cleanup_terminated(), 1);
        assert!(!s.used_ports.contains(&9100));
        assert!(s.used_ports.contains(&9101));
        assert_eq!(s.processes.len(), 1);
    }
}
