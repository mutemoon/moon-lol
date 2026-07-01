//! 游戏进程管理器：端口池 + 进程表 + spawn/kill。
//!
//! 逻辑取自 `lol_web_server::LocalGameServiceImpl` 的进程托管部分（原步骤 1/3/5/7），
//! 去掉对局体系（match 记录、状态机、supervisor、胜负判定）。对局体系由云端
//! `LocalGameService` 在本层之上叠加；桌面端直接用本层做本地多游戏调试托管。

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use lol_agent_runtime::AgentConfig;
use lol_client::launch::BevySpawnRequest;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{ManagerError, ManagerResult};
use crate::port_pool::{ProcessStatus, allocate_port, is_valid_port};

/// 进程启动抽象（可 mock）。spawn 实现由各端提供：
/// 桌面端 std/tokio 双路（dev `cargo run --` / release 打包二进制、非 headless），
/// 云端 tokio headless。
#[async_trait]
pub trait ProcessLauncher: Send + Sync {
    /// 启动监听 `port` 的 Bevy 子进程。`req.spawn` 已含 program/prefix/game_config/cwd/rust_log，
    /// 但 `port` 以参数为准（由 manager 从端口池分配后填入）。
    async fn launch(&self, port: i32, req: &BevySpawnRequest) -> ManagerResult<()>;

    /// 停止监听 `port` 的子进程。
    async fn kill(&self, port: i32) -> ManagerResult<()>;
}

/// 托管的游戏进程记录（运行时状态，不持久化——进程退出即失效）。
#[derive(Debug, Clone)]
pub struct ManagedProcess {
    /// 进程实例 id。桌面端用作停止/选择标识；云端对齐 match_id。
    pub id: Uuid,
    pub port: i32,
    pub status: ProcessStatus,
}

/// 内存状态：已用端口 + 托管进程列表。
#[derive(Default)]
pub struct ProcessManagerState {
    pub used_ports: HashSet<i32>,
    pub processes: Vec<ManagedProcess>,
}

impl ProcessManagerState {
    /// 标记端口为已用。
    pub fn acquire(&mut self, port: i32) {
        self.used_ports.insert(port);
    }

    /// 释放端口。
    pub fn release(&mut self, port: i32) {
        self.used_ports.remove(&port);
    }

    /// 添加托管进程。
    pub fn add_process(&mut self, proc_: ManagedProcess) {
        self.processes.push(proc_);
    }

    /// 按 id 移除进程，返回是否找到。
    pub fn remove_process_by_id(&mut self, id: Uuid) -> bool {
        let before = self.processes.len();
        self.processes.retain(|p| p.id != id);
        self.processes.len() < before
    }

    /// 按 port 移除进程，返回是否找到。
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

/// 启动一个游戏的输入。
#[derive(Debug, Clone)]
pub struct StartGameInput {
    /// 进程实例 id（调用方提供）：桌面端用作日志 db 命名与停止标识，云端对齐 match_id。
    pub id: Uuid,
    /// spawn 配置（program/prefix_args/game_config/cwd/rust_log）。port 字段由 manager 覆盖。
    pub spawn: BevySpawnRequest,
    /// 场景 agent 阵容：非空且 manager 注入了 [`AgentRunner`] 时启动 AI 决策环。
    pub scenario_agents: Vec<AgentConfig>,
}

/// AI 决策环启动器（解耦凭证解析）：桌面注入桌面 runner，云端注入云端 runner。
/// `None` 表示不启动 AI（纯观战/回放）。
pub type AgentRunner =
    Arc<dyn Fn(i32, Vec<AgentConfig>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// 游戏进程管理器。
pub struct GameProcessManager {
    pub launcher: Arc<dyn ProcessLauncher>,
    pub agent_runner: Option<AgentRunner>,
    pub state: Arc<Mutex<ProcessManagerState>>,
}

impl GameProcessManager {
    pub fn new(launcher: Arc<dyn ProcessLauncher>, agent_runner: Option<AgentRunner>) -> Self {
        Self {
            launcher,
            agent_runner,
            state: Arc::new(Mutex::new(ProcessManagerState::default())),
        }
    }

    /// 分配端口 → spawn → 进程表登记 → (可选) spawn AI 环。返回 (id, port)。
    ///
    /// spawn 失败时回滚端口。不涉及 match 记录 / 胜负判定。
    pub async fn start(&self, mut input: StartGameInput) -> ManagerResult<(Uuid, i32)> {
        // 1. 分配端口
        let port = {
            let state = self.state.lock().await;
            allocate_port(&state.used_ports)
                .ok_or(ManagerError::Conflict("端口池已满，无法启动新游戏".into()))?
        };
        input.spawn.port = port as u16;

        let id = input.id;

        // 2. 启动 Bevy 进程（失败回滚端口）
        if let Err(e) = self.launcher.launch(port, &input.spawn).await {
            let mut state = self.state.lock().await;
            state.release(port);
            return Err(e);
        }

        // 3. 登记托管进程
        {
            let mut state = self.state.lock().await;
            state.acquire(port);
            state.add_process(ManagedProcess {
                id,
                port,
                status: ProcessStatus::Running,
            });
        }

        // 4. (可选) 启动 AI 决策环
        if !input.scenario_agents.is_empty() {
            if let Some(runner) = &self.agent_runner {
                let runner = runner.clone();
                let agents = input.scenario_agents.clone();
                tokio::spawn(async move {
                    runner(port, agents).await;
                });
            }
        }

        Ok((id, port))
    }

    /// 按 id 停止进程 + 释放端口。
    pub async fn stop(&self, id: Uuid) -> ManagerResult<()> {
        let port = {
            let state = self.state.lock().await;
            let proc_ = state
                .processes
                .iter()
                .find(|p| p.id == id)
                .ok_or(ManagerError::NotFound)?;
            proc_.port
        };

        if !is_valid_port(port) {
            return Err(ManagerError::Internal(format!("无效端口: {port}")));
        }

        self.launcher.kill(port).await?;

        {
            let mut state = self.state.lock().await;
            state.release(port);
            state.remove_process_by_id(id);
        }

        Ok(())
    }

    /// 按 port 查找托管进程（桌面端按端口选局用）。
    pub async fn find_by_port(&self, port: i32) -> ManagerResult<ManagedProcess> {
        let state = self.state.lock().await;
        state
            .processes
            .iter()
            .find(|p| p.port == port)
            .cloned()
            .ok_or(ManagerError::NotFound)
    }

    /// 按 port 停止进程 + 释放端口（云端已从 DB 拿到 match 的 bevy_port 时用）。
    pub async fn stop_by_port(&self, port: i32) -> ManagerResult<()> {
        if !is_valid_port(port) {
            return Err(ManagerError::Internal(format!("无效端口: {port}")));
        }

        self.launcher.kill(port).await?;

        let mut state = self.state.lock().await;
        state.release(port);
        state.remove_process_by_port(port);
        Ok(())
    }

    pub async fn list_processes(&self) -> ManagerResult<Vec<ManagedProcess>> {
        let state = self.state.lock().await;
        Ok(state.processes.clone())
    }

    /// 清理已退出的进程（Stopped/Crashed 释放端口）。
    pub async fn cleanup(&self) -> ManagerResult<usize> {
        let mut state = self.state.lock().await;
        Ok(state.cleanup_terminated())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StdMutex;

    use super::*;
    use crate::port_pool::{PORT_POOL_END, PORT_POOL_START};
    struct FakeLauncher {
        launch_fail: StdMutex<bool>,
        killed: StdMutex<Vec<i32>>,
    }

    #[async_trait]
    impl ProcessLauncher for FakeLauncher {
        async fn launch(&self, _port: i32, _req: &BevySpawnRequest) -> ManagerResult<()> {
            if *self.launch_fail.lock().unwrap() {
                return Err(ManagerError::Internal("launch failed".into()));
            }
            Ok(())
        }
        async fn kill(&self, port: i32) -> ManagerResult<()> {
            self.killed.lock().unwrap().push(port);
            Ok(())
        }
    }

    fn spawn_req() -> BevySpawnRequest {
        BevySpawnRequest {
            program: "cargo".into(),
            prefix_args: vec![],
            port: 0,
            game_config: lol_client::launch::BevyGameConfig::default(),
            cwd: None,
            rust_log: None,
            log_db: None,
        }
    }

    fn manager(launch_fail: bool) -> (GameProcessManager, Arc<FakeLauncher>) {
        let launcher = Arc::new(FakeLauncher {
            launch_fail: StdMutex::new(launch_fail),
            killed: StdMutex::new(vec![]),
        });
        let mgr = GameProcessManager::new(launcher.clone(), None);
        (mgr, launcher)
    }

    #[tokio::test]
    async fn start_allocates_port_and_registers() {
        let (mgr, _) = manager(false);
        let (id, port) = mgr
            .start(StartGameInput {
                id: Uuid::new_v4(),
                spawn: spawn_req(),
                scenario_agents: vec![],
            })
            .await
            .unwrap();
        assert_eq!(port, 9100);
        let state = mgr.state.lock().await;
        assert!(state.used_ports.contains(&9100));
        assert_eq!(state.processes.len(), 1);
        assert_eq!(state.processes[0].id, id);
    }

    #[tokio::test]
    async fn start_launch_failure_releases_port() {
        let (mgr, _) = manager(true);
        let err = mgr
            .start(StartGameInput {
                id: Uuid::new_v4(),
                spawn: spawn_req(),
                scenario_agents: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, ManagerError::Internal(_)));
        let state = mgr.state.lock().await;
        assert!(state.used_ports.is_empty());
        assert!(state.processes.is_empty());
    }

    #[tokio::test]
    async fn stop_kills_and_releases() {
        let (mgr, launcher) = manager(false);
        let (id, port) = mgr
            .start(StartGameInput {
                id: Uuid::new_v4(),
                spawn: spawn_req(),
                scenario_agents: vec![],
            })
            .await
            .unwrap();
        mgr.stop(id).await.unwrap();
        assert_eq!(*launcher.killed.lock().unwrap(), vec![port]);
        let state = mgr.state.lock().await;
        assert!(!state.used_ports.contains(&port));
        assert!(state.processes.is_empty());
    }

    #[tokio::test]
    async fn stop_missing_not_found() {
        let (mgr, _) = manager(false);
        let err = mgr.stop(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ManagerError::NotFound));
    }

    #[tokio::test]
    async fn find_by_port() {
        let (mgr, _) = manager(false);
        let (id, port) = mgr
            .start(StartGameInput {
                id: Uuid::new_v4(),
                spawn: spawn_req(),
                scenario_agents: vec![],
            })
            .await
            .unwrap();
        let found = mgr.find_by_port(port).await.unwrap();
        assert_eq!(found.id, id);
        assert!(mgr.find_by_port(9999).await.is_err());
    }

    #[tokio::test]
    async fn port_pool_full_rejected() {
        let (mgr, _) = manager(false);
        {
            let mut state = mgr.state.lock().await;
            for p in PORT_POOL_START..PORT_POOL_END {
                state.used_ports.insert(p);
            }
        }
        let err = mgr
            .start(StartGameInput {
                id: Uuid::new_v4(),
                spawn: spawn_req(),
                scenario_agents: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, ManagerError::Conflict(_)));
    }

    #[test]
    fn state_cleanup_terminated() {
        let mut s = ProcessManagerState::default();
        s.acquire(9100);
        s.acquire(9101);
        s.add_process(ManagedProcess {
            id: Uuid::new_v4(),
            port: 9100,
            status: ProcessStatus::Crashed,
        });
        s.add_process(ManagedProcess {
            id: Uuid::new_v4(),
            port: 9101,
            status: ProcessStatus::Running,
        });
        assert_eq!(s.cleanup_terminated(), 1);
        assert!(!s.used_ports.contains(&9100));
        assert!(s.used_ports.contains(&9101));
        assert_eq!(s.processes.len(), 1);
    }

    #[test]
    fn state_remove_by_id_and_port() {
        let mut s = ProcessManagerState::default();
        let id = Uuid::new_v4();
        s.add_process(ManagedProcess {
            id,
            port: 9100,
            status: ProcessStatus::Running,
        });
        assert!(s.remove_process_by_id(id));
        assert!(!s.remove_process_by_id(id));
        s.add_process(ManagedProcess {
            id: Uuid::new_v4(),
            port: 9100,
            status: ProcessStatus::Running,
        });
        assert!(s.remove_process_by_port(9100));
    }
}
