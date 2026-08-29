use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use lol_env::curriculum::{CurriculumConfig, CurriculumScheduler};
pub use lol_rl_protocol::{
    CheckpointItem, EngineMode, InFrame, ObsFeaturePayload, OutFrame, RewardItem,
    TaskConfigPayload, TaskOverviewItem,
};
use sqlx::PgPool;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::{error, info};
use uuid::Uuid;

use crate::algo::agent::RlAgent;
use crate::algo::grpo::{GRPOAgent, GRPOConfig};
use crate::algo::ppo::{PPOAgent, PPOConfig};
use crate::autotune::AutoTuner;
use crate::db::{self, CheckpointRow, PgRlRepo, RlRepo, TaskRow};
use crate::engine::pool::TrainingWorkerPool;
use crate::engine::r#async::AsyncTrainingSession;
use crate::engine::sync::SyncTrainingSession;
use crate::engine::traits::TrainingEngine;
use crate::model_store::{checkpoint_dir, new_checkpoint_path};

/// 训练循环内保存当前权重的请求。
#[derive(Debug, Clone)]
pub struct SaveRequest {
    pub ckpt_id: String,
    pub path: String,
    pub ep_return: f32,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub status: String,
    pub current_step: usize,
    pub current_iter: usize,
    pub ep_return: f32,
    pub config: TaskConfigPayload,
    pub checkpoints: Vec<CheckpointItem>,
    pub metrics_history: Vec<lol_rl_protocol::MetricsRow>,
    pub logs: Vec<String>,
    pub created_at: String,
    /// 训练循环存活时的保存请求通道；训练结束后置 None。
    pub save_tx: Option<mpsc::UnboundedSender<SaveRequest>>,
}

impl TaskState {
    fn from_row(r: &TaskRow) -> Self {
        let config: TaskConfigPayload =
            serde_json::from_value(r.config_json.clone()).unwrap_or_default();
        Self {
            id: r.id.to_string(),
            name: r.name.clone(),
            agent_type: r.agent_type.clone(),
            env_name: r.env_name.clone(),
            status: r.status.clone(),
            current_step: r.current_step as usize,
            current_iter: 0,
            ep_return: r.ep_return,
            config,
            checkpoints: Vec::new(),
            metrics_history: Vec::new(),
            logs: Vec::new(),
            created_at: r.created_at.to_rfc3339(),
            save_tx: None,
        }
    }
}

pub struct RLService {
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    event_tx: broadcast::Sender<OutFrame>,
    repo: Arc<dyn RlRepo>,
    worker_pool: Arc<TrainingWorkerPool>,
}

impl RLService {
    pub async fn new(
        pool: PgPool,
        event_capacity: usize,
    ) -> anyhow::Result<(Self, broadcast::Receiver<OutFrame>)> {
        db::apply_schema(&pool)
            .await
            .map_err(|e| anyhow::anyhow!("RL schema 初始化失败: {e}"))?;
        let repo = Arc::new(PgRlRepo { pool });
        Self::new_with_repo(repo, event_capacity).await
    }

    pub async fn new_with_repo(
        repo: Arc<dyn RlRepo>,
        event_capacity: usize,
    ) -> anyhow::Result<(Self, broadcast::Receiver<OutFrame>)> {
        let interrupted = repo.mark_all_running_interrupted().await.unwrap_or(0);
        if interrupted > 0 {
            info!("连接数据库…恢复 {} 个中断任务", interrupted);
        }

        let db_tasks = repo.list_tasks().await.unwrap_or_default();
        let mut initial_tasks = HashMap::new();
        for t in &db_tasks {
            let mut state = TaskState::from_row(t);
            if let Ok(cps) = repo.list_checkpoints(&state.id).await {
                for cp in &cps {
                    let id = Path::new(&cp.path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("iter-{}", cp.step));
                    state.checkpoints.push(CheckpointItem {
                        id,
                        step: cp.step as usize,
                        path: cp.path.clone(),
                        ep_return: cp.ep_return,
                        created_at: cp.created_at.to_rfc3339(),
                    });
                }
            }
            if let Ok(metrics) = repo.list_metrics(&state.id).await {
                state.metrics_history = metrics;
            }
            if let Ok(logs) = repo.list_logs(&state.id).await {
                state.logs = logs;
            }
            initial_tasks.insert(state.id.clone(), state);
        }
        if !initial_tasks.is_empty() {
            info!("从数据库载入 {} 个任务", initial_tasks.len());
        }

        let (event_tx, rx) = broadcast::channel(event_capacity);
        let worker_pool = Arc::new(TrainingWorkerPool::new(
            crate::device::device_kind_from_env(),
        ));
        Ok((
            Self {
                tasks: Arc::new(Mutex::new(initial_tasks)),
                event_tx,
                repo,
                worker_pool,
            },
            rx,
        ))
    }

    pub async fn new_in_memory(event_capacity: usize) -> (Self, broadcast::Receiver<OutFrame>) {
        Self::new_with_repo(Arc::new(crate::db::NoopRlRepo), event_capacity)
            .await
            .expect("In-memory RLService creation should never fail")
    }

    pub fn subscribe(&self) -> broadcast::Receiver<OutFrame> {
        self.event_tx.subscribe()
    }

    pub fn get_event_sender(&self) -> broadcast::Sender<OutFrame> {
        self.event_tx.clone()
    }

    pub async fn handle_frame(&self, frame: InFrame) {
        match frame {
            InFrame::GetTaskList => {
                self.broadcast_task_list().await;
            }
            InFrame::GetTaskDetail { task_id } => {
                self.handle_get_task_detail(&task_id).await;
            }
            InFrame::CreateTask { config } => {
                let task_id = Uuid::new_v4();
                let config_json = serde_json::to_value(&config).unwrap_or_default();
                let now = Utc::now();
                let agent_display = format!(
                    "{} ({})",
                    config.algorithm.display_name(),
                    config.backbone.display_name()
                );
                let row = TaskRow {
                    id: task_id,
                    name: config.name.clone(),
                    agent_type: agent_display.clone(),
                    env_name: config.env_name.clone(),
                    status: "queued".to_string(),
                    config_json,
                    current_step: 0,
                    ep_return: 0.0,
                    created_at: now,
                    updated_at: now,
                };
                if let Err(e) = self.repo.insert_task(&row).await {
                    error!("创建任务写入 DB 失败: {e}");
                    return;
                }

                let new_task = TaskState {
                    id: task_id.to_string(),
                    name: config.name.clone(),
                    agent_type: agent_display,
                    env_name: config.env_name.clone(),
                    status: "queued".to_string(),
                    current_step: 0,
                    current_iter: 0,
                    ep_return: 0.0,
                    config,
                    checkpoints: Vec::new(),
                    metrics_history: Vec::new(),
                    logs: Vec::new(),
                    created_at: now.to_rfc3339(),
                    save_tx: None,
                };
                {
                    let mut tasks = self.tasks.lock().await;
                    tasks.insert(task_id.to_string(), new_task);
                }
                info!("创建新的 RL 训练任务: {}", task_id);

                self.broadcast_task_list().await;

                // Mark as running and spawn training
                let tid = task_id.to_string();
                let _ = self.repo.update_status(&tid, "running").await;
                {
                    let mut tasks = self.tasks.lock().await;
                    if let Some(t) = tasks.get_mut(&tid) {
                        t.status = "running".to_string();
                    }
                }
                let _ = self.event_tx.send(OutFrame::Status {
                    task_id: tid.clone(),
                    status: "running".into(),
                });
                self.broadcast_task_list().await;

                let event_tx = self.event_tx.clone();
                let tasks_arc = self.tasks.clone();
                let repo = self.repo.clone();
                let tid_clone = tid.clone();
                let worker_pool = self.worker_pool.clone();
                tokio::spawn(async move {
                    // 训练循环存活期间持有 permit，限制并发训练任务数。
                    let permit = worker_pool.acquire().await;
                    let _ = tokio::task::spawn_blocking(move || {
                        let _permit = permit;
                        run_training_loop_for_task(event_tx, tasks_arc, repo, tid_clone);
                    })
                    .await;
                });
            }
            InFrame::Control {
                task_id,
                command,
                config_json: _,
            } => match command.as_str() {
                "start" => {
                    info!("启动任务 {}", task_id);
                    {
                        let mut tasks = self.tasks.lock().await;
                        if let Some(t) = tasks.get_mut(&task_id) {
                            t.status = "running".to_string();
                        }
                    }
                    let _ = self.repo.update_status(&task_id, "running").await;
                    let _ = self.event_tx.send(OutFrame::Status {
                        task_id: task_id.clone(),
                        status: "running".into(),
                    });
                    self.broadcast_task_list().await;

                    let event_tx = self.event_tx.clone();
                    let tasks_arc = self.tasks.clone();
                    let repo = self.repo.clone();
                    let tid = task_id.clone();
                    let worker_pool = self.worker_pool.clone();
                    tokio::spawn(async move {
                        let permit = worker_pool.acquire().await;
                        let _ = tokio::task::spawn_blocking(move || {
                            let _permit = permit;
                            run_training_loop_for_task(event_tx, tasks_arc, repo, tid);
                        })
                        .await;
                    });
                }
                "stop" => {
                    info!("停止任务 {}", task_id);
                    {
                        let mut tasks = self.tasks.lock().await;
                        if let Some(t) = tasks.get_mut(&task_id) {
                            t.status = "stopped".to_string();
                        }
                    }
                    let _ = self.repo.update_status(&task_id, "stopped").await;
                    let _ = self.event_tx.send(OutFrame::Status {
                        task_id: task_id.clone(),
                        status: "stopped".into(),
                    });
                    self.broadcast_task_list().await;
                }
                _ => {}
            },
            InFrame::SaveCheckpoint { task_id } => {
                self.handle_save_checkpoint(&task_id).await;
            }
            InFrame::ApplyCheckpoint { task_id, id } => {
                self.handle_apply_checkpoint(&task_id, &id).await;
            }
            InFrame::DeleteTask { task_id } => {
                self.handle_delete_task(&task_id).await;
            }
        }
    }

    async fn handle_get_task_detail(&self, task_id: &str) {
        let (checkpoints, metrics_history, logs) = {
            let tasks = self.tasks.lock().await;
            if let Some(t) = tasks.get(task_id) {
                (
                    t.checkpoints.clone(),
                    t.metrics_history.clone(),
                    t.logs.clone(),
                )
            } else {
                let cps = self
                    .repo
                    .list_checkpoints(task_id)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|cp| {
                        let id = Path::new(&cp.path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("iter-{}", cp.step));
                        CheckpointItem {
                            id,
                            step: cp.step as usize,
                            path: cp.path,
                            ep_return: cp.ep_return,
                            created_at: cp.created_at.to_rfc3339(),
                        }
                    })
                    .collect();
                let m = self.repo.list_metrics(task_id).await.unwrap_or_default();
                let l = self.repo.list_logs(task_id).await.unwrap_or_default();
                (cps, m, l)
            }
        };
        let _ = self.event_tx.send(OutFrame::TaskDetail {
            task_id: task_id.to_string(),
            checkpoints,
            metrics_history,
            logs,
        });
    }

    async fn handle_delete_task(&self, task_id: &str) {
        // 先从内存移除：训练循环在下一轮迭代感知并退出，persist_checkpoint 也会跳过已删除任务
        {
            let mut tasks = self.tasks.lock().await;
            tasks.remove(task_id);
        }
        // 删除磁盘上的模型权重目录
        let dir = checkpoint_dir(task_id);
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                error!("删除任务 {} 模型目录失败 {}: {e}", task_id, dir.display());
            }
        }
        // 删除 DB 记录（rl_checkpoints 经外键 ON DELETE CASCADE 一并删除）
        if let Err(e) = self.repo.delete_task(task_id).await {
            error!("删除任务 {} DB 记录失败: {e}", task_id);
        }
        info!("已删除 RL 训练任务 {}", task_id);
        self.broadcast_task_list().await;
    }

    async fn handle_save_checkpoint(&self, task_id: &str) {
        let (save_tx, iter, ep_return) = {
            let tasks = self.tasks.lock().await;
            match tasks.get(task_id) {
                Some(t) => (t.save_tx.clone(), t.current_iter, t.ep_return),
                None => return,
            }
        };
        let ckpt_id = format!("iter-{}", iter);
        let Some(tx) = save_tx else {
            // 训练已结束：最终模型已在收敛时自动保存，无需重复写入
            let _ = self.event_tx.send(OutFrame::Log {
                task_id: task_id.to_string(),
                level: "info".into(),
                message: format!("任务已结束，最终模型已自动保存为 {ckpt_id}"),
            });
            return;
        };
        let path = new_checkpoint_path(task_id, &ckpt_id)
            .to_string_lossy()
            .to_string();
        let _ = tx.send(SaveRequest {
            ckpt_id,
            path,
            ep_return,
        });
    }

    async fn handle_apply_checkpoint(&self, task_id: &str, ckpt_id: &str) {
        // 1. 优先从内存任务状态中查询（支持纯内存模式与正在运行的任务）
        let in_memory_ckpt = {
            let tasks = self.tasks.lock().await;
            tasks
                .get(task_id)
                .and_then(|t| t.checkpoints.iter().find(|c| c.id == ckpt_id).cloned())
        };

        if let Some(item) = in_memory_ckpt {
            info!("从内存状态加载 Checkpoint {} (任务 {})", ckpt_id, task_id);
            let _ = self.event_tx.send(OutFrame::CheckpointLoaded {
                task_id: task_id.to_string(),
                checkpoint: item,
            });
            return;
        }

        // 2. 从数据库中查询历史 checkpoint
        match self.repo.get_checkpoint(task_id, ckpt_id).await {
            Ok(Some(cp)) => {
                let id = Path::new(&cp.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| ckpt_id.to_string());
                let item = CheckpointItem {
                    id,
                    step: cp.step as usize,
                    path: cp.path.clone(),
                    ep_return: cp.ep_return,
                    created_at: cp.created_at.to_rfc3339(),
                };
                let _ = self.event_tx.send(OutFrame::CheckpointLoaded {
                    task_id: task_id.to_string(),
                    checkpoint: item,
                });
            }
            Ok(None) => {
                let _ = self.event_tx.send(OutFrame::Log {
                    task_id: task_id.to_string(),
                    level: "warn".into(),
                    message: format!("Checkpoint {} 未找到", ckpt_id),
                });
            }
            Err(e) => {
                error!("查询 checkpoint {} 失败: {}", ckpt_id, e);
            }
        }
    }

    async fn broadcast_task_list(&self) {
        let tasks_lock = self.tasks.lock().await;
        let mut task_items: Vec<TaskOverviewItem> = tasks_lock
            .values()
            .map(|t| TaskOverviewItem {
                id: t.id.clone(),
                name: t.name.clone(),
                algorithm: t.config.algorithm,
                backbone: t.config.backbone,
                engine_mode: t.config.engine_mode,
                env_name: t.env_name.clone(),
                status: t.status.clone(),
                current_step: t.current_step,
                ep_return: t.ep_return,
                checkpoints_count: t.checkpoints.len(),
                hidden_dim: t.config.hidden_dim,
                parallel_envs: t.config.parallel_envs,
                lr: t.config.lr,
                total_iterations: t.config.total_iterations,
                rollout_steps_per_env: t.config.rollout_steps_per_env,
                created_at: t.created_at.clone(),
            })
            .collect();
        task_items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let _ = self.event_tx.send(OutFrame::TaskList { tasks: task_items });
    }


    pub fn spawn_command_handler(self: Arc<Self>, mut rx: mpsc::Receiver<InFrame>) {
        tokio::spawn(async move {
            while let Some(frame) = rx.recv().await {
                self.handle_frame(frame).await;
            }
        });
    }
}

/// 在当前线程执行一次异步 DB 写（若无外部 tokio runtime 上下文则自动构建临时 runtime）
fn block_on_db<F: std::future::Future>(fut: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(fut),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("创建临时 tokio runtime 失败")
            .block_on(fut),
    }
}

/// 写 safetensors 权重文件 + 登记 checkpoint 到 DB + 广播。仅在训练循环内（agent 存活时）调用。
fn persist_checkpoint(
    task_id: &str,
    req: SaveRequest,
    agent: &RlAgent,
    tasks: &Arc<Mutex<HashMap<String, TaskState>>>,
    repo: &Arc<dyn RlRepo>,
    event_tx: &broadcast::Sender<OutFrame>,
) {
    // 任务已被删除 → 不再写任何 checkpoint（防止训练循环末尾自动保存重建模型文件）
    if !tasks.blocking_lock().contains_key(task_id) {
        return;
    }
    // 同一展示 id 已保存过则跳过，避免运行中手动保存与结束自动保存重复
    let already = {
        let t = tasks.blocking_lock();
        t.get(task_id)
            .map(|x| x.checkpoints.iter().any(|c| c.id == req.ckpt_id))
            .unwrap_or(false)
    };
    if already {
        return;
    }

    if let Err(e) = agent.save(Path::new(&req.path)) {
        error!("保存模型文件失败 {}: {e}", req.path);
        return;
    }

    let iter = req
        .ckpt_id
        .strip_prefix("iter-")
        .or_else(|| req.ckpt_id.strip_prefix("ckpt-"))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let now = Utc::now();

    // 同步写入自描述元数据 JSON（包含环境、输入输出维度与训练指标）
    let meta_path = Path::new(&req.path).with_extension("meta.json");
    let task_meta_opt = {
        let t = tasks.blocking_lock();
        t.get(task_id).map(|task| {
            (
                task.env_name.clone(),
                task.config.hidden_dim,
                task.agent_type.clone(),
                task.current_step,
            )
        })
    };
    if let Some((env_name, hidden_dim, agent_type, total_steps)) = task_meta_opt {
        let meta_json = serde_json::json!({
            "task_id": task_id,
            "ckpt_id": req.ckpt_id,
            "iter": iter,
            "step": total_steps,
            "ep_return": req.ep_return,
            "agent_type": agent_type,
            "env_name": env_name,
            "hidden_dim": hidden_dim,
            "created_at": now.to_rfc3339(),
        });
        if let Ok(meta_str) = serde_json::to_string_pretty(&meta_json) {
            let _ = std::fs::write(&meta_path, meta_str);
        }
    }

    let cp_row = CheckpointRow {
        id: Uuid::new_v4(),
        task_id: Uuid::parse_str(task_id).unwrap_or_default(),
        step: iter as i64,
        path: req.path.clone(),
        ep_return: req.ep_return,
        created_at: now,
    };
    // DB 插入：复用当前 runtime
    if let Err(e) = block_on_db(repo.insert_checkpoint(&cp_row)) {
        error!("写入 checkpoint DB 失败: {e}");
        return;
    }

    let ckpt_item = CheckpointItem {
        id: req.ckpt_id,
        step: iter,
        path: req.path,
        ep_return: req.ep_return,
        created_at: now.to_rfc3339(),
    };
    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(task_id) {
            task.checkpoints.push(ckpt_item.clone());
        }
    }
    let _ = event_tx.send(OutFrame::CheckpointMsg {
        task_id: task_id.to_string(),
        checkpoint: ckpt_item.clone(),
    });
    let _ = event_tx.send(OutFrame::Log {
        task_id: task_id.to_string(),
        level: "info".into(),
        message: format!("已为任务 {task_id} 保存 Checkpoint 到 {}", ckpt_item.path),
    });
}

fn run_training_loop_for_task(
    event_tx: broadcast::Sender<OutFrame>,
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    repo: Arc<dyn RlRepo>,
    task_id: String,
) {
    let task_config = {
        let t = tasks.blocking_lock();
        t.get(&task_id)
            .map(|s| s.config.clone())
            .unwrap_or_default()
    };

    macro_rules! dispatch_task_env {
        ($(($env_ty:ty, $env_name:expr)),*) => {
            match task_config.env_name.as_str() {
                $(
                    s if s == $env_name => {
                        run_generic_training_loop::<$env_ty>(
                            event_tx,
                            tasks,
                            repo,
                            task_id,
                            task_config,
                        );
                    }
                )*
                unknown => {
                    tracing::warn!("未知环境名称 {unknown}，降级使用默认环境");
                    run_generic_training_loop::<lol_env::FioraV2Env>(
                        event_tx,
                        tasks,
                        repo,
                        task_id,
                        task_config,
                    );
                }
            }
        };
    }

    lol_env::for_all_rl_environments!(dispatch_task_env);
}

fn run_generic_training_loop<E: lol_env::RlEnvironment + 'static>(
    event_tx: broadcast::Sender<OutFrame>,
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    repo: Arc<dyn RlRepo>,
    task_id: String,
    task_config: TaskConfigPayload,
) {
    let rollout_steps = task_config.rollout_steps_per_env.max(1);
    let state_dim = E::state_dim();
    let action_space = E::action_space();
    let total_iterations = task_config.total_iterations.max(1);
    let hidden_dim = task_config.hidden_dim.max(32);
    let device = crate::device::select_device().unwrap_or(candle_core::Device::Cpu);
    let backbone = task_config.backbone();
    let (mut tuned, is_custom) = if task_config.parallel_envs > 0 {
        let n = task_config.parallel_envs;
        let total_samples = n * rollout_steps * E::num_agents().max(1);
        let train_batch_size = (total_samples / 4).clamp(16, 256);
        let infer_batch_size = n.next_power_of_two().min(128);
        info!(
            "🎯 为任务 {} (主干: {}) 应用自定义并发: 并发 Actors={}, 训练 MiniBatch={}, 推理 Batch={} (跳过 AutoTuner 探测，极速启动)",
            task_id, backbone, n, train_batch_size, infer_batch_size
        );
        (
            crate::autotune::TunedConfig {
                num_parallel_envs: n,
                infer_batch_size,
                train_batch_size,
                dynamic_batch_timeout_us: 200,
                estimated_sps: 0.0,
            },
            true,
        )
    } else {
        // 1. 自动调整模式：运行 AutoTuner 全面硬件算力探测与最优配置求解
        let tuned = match AutoTuner::profile_with_algo_and_backbone::<E>(
            state_dim,
            hidden_dim,
            &action_space,
            &device,
            task_config.algorithm,
            backbone,
        ) {
            Ok(profile) => {
                let res = AutoTuner::solve(&profile, rollout_steps, task_config.ppo_epochs.max(1));
                info!(
                    "🎯 [AutoTuner] 为任务 {} (算法: {}, 主干: {}) 自动求解最优配置: 并发 Actors={}, 推理 Batch={}, 训练 MiniBatch={}, 预估 SPS: {:.1}",
                    task_id,
                    task_config.algorithm,
                    backbone,
                    res.num_parallel_envs,
                    res.infer_batch_size,
                    res.train_batch_size,
                    res.estimated_sps
                );
                res
            }
            Err(e) => {
                let fallback_actors = num_cpus::get().clamp(2, 16);
                tracing::warn!("AutoTuner 探测失败 ({e}), 降级使用配置: {fallback_actors}");
                crate::autotune::TunedConfig {
                    num_parallel_envs: fallback_actors,
                    infer_batch_size: fallback_actors.min(32),
                    train_batch_size: (fallback_actors * 16).clamp(32, 256),
                    dynamic_batch_timeout_us: 200,
                    estimated_sps: 2000.0,
                }
            }
        };
        (tuned, false)
    };

    let num_parallel_envs = tuned.num_parallel_envs;

    // 1b. 真实校准（仅在自动调优模式下执行）：用实际生效配置跑 K 轮迭代实测 SPS
    if !is_custom {
        let mut calib_config = tuned.clone();
        calib_config.num_parallel_envs = num_parallel_envs;
        match AutoTuner::calibrate_with_algo_and_backbone::<E>(
            state_dim,
            hidden_dim,
            &action_space,
            &device,
            rollout_steps,
            task_config.ppo_epochs.max(1),
            &calib_config,
            task_config.algorithm,
            backbone,
        ) {
            Ok(measured) => {
                tuned.estimated_sps = measured;
                info!(
                    "🎯 [AutoTuner] 为任务 {} 真实校准完成，实测 SPS: {:.1}",
                    task_id, measured
                );
            }
            Err(e) => {
                tracing::warn!(
                    "AutoTuner 校准失败 ({e})，沿用组件级预估 SPS: {:.1}",
                    tuned.estimated_sps
                );
            }
        }
    }

    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.config.parallel_envs = num_parallel_envs;
        }
    }

    let rl_agent: RlAgent = if task_config.is_grpo() {
        let grpo_config = GRPOConfig {
            lr: task_config.lr as f64,
            gamma: task_config.gamma,
            clip_eps: task_config.clip_eps,
            grpo_epochs: task_config.ppo_epochs.max(1),
            group_size: task_config.grpo_group_size.unwrap_or(4),
            max_grad_norm: 0.5,
        };
        match GRPOAgent::create_for_env_with_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space.clone(),
            grpo_config,
            device.clone(),
            backbone,
        ) {
            Ok(a) => a.into(),
            Err(e) => {
                error!("创建 GRPOAgent 失败: {e}");
                return;
            }
        }
    } else {
        let ppo_config = PPOConfig {
            lr: task_config.lr as f64,
            gamma: task_config.gamma,
            gae_lambda: task_config.gae_lambda,
            clip_eps: task_config.clip_eps,
            c1: 0.5,
            ppo_epochs: task_config.ppo_epochs.max(1),
            clip_vloss: true,
            max_grad_norm: 0.5,
        };
        match PPOAgent::create_for_env_with_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space.clone(),
            ppo_config,
            device.clone(),
            backbone,
        ) {
            Ok(a) => a.into(),
            Err(e) => {
                error!("创建 PPOAgent 失败: {e}");
                return;
            }
        }
    };

    let summary = rl_agent.parameter_summary();
    let _ = event_tx.send(OutFrame::Log {
        task_id: task_id.clone(),
        level: "info".into(),
        message: format!(
            "🧠 [模型网络结构] 算法: {}, 引擎: {}, 主干: {:?}, 总可训练参数量: {} ({})",
            if task_config.is_grpo() { "GRPO" } else { "PPO" },
            task_config.engine_mode,
            backbone,
            summary.total_params,
            crate::policy::format_param_k_m(summary.total_params)
        ),
    });

    // 2. 根据 engine_mode 构建面向统一 TrainingEngine Trait 的训练引擎
    let mut engine: Box<dyn TrainingEngine> = match task_config.engine_mode {
        EngineMode::Async => Box::new(AsyncTrainingSession::<E>::new(
            rl_agent,
            num_parallel_envs,
            state_dim,
            rollout_steps,
            tuned.train_batch_size,
            tuned.infer_batch_size,
            tuned.dynamic_batch_timeout_us,
            device.clone(),
        )),
        EngineMode::Sync => Box::new(SyncTrainingSession::<E>::new(
            rl_agent,
            num_parallel_envs,
            state_dim,
            rollout_steps,
            candle_core::Device::Cpu,
        )),
    };

    // 初始化课程学习调度器（支持任务显式配置，或对 SoloV0 自动启用默认课程）
    let mut curriculum_scheduler = if let Some(ref c_cfg) = task_config.curriculum {
        Some(CurriculumScheduler::new(c_cfg.clone()))
    } else if task_config.env_name == "SoloV0" {
        Some(CurriculumScheduler::new(CurriculumConfig::default()))
    } else {
        None
    };

    // 初始课程参数下发
    if let Some(ref c) = curriculum_scheduler {
        engine.update_curriculum(
            c.minion_hp_scale(),
            c.cs_reward(),
            c.attack_no_cs_penalty(),
            c.harass_coef(),
        );
        info!("🎓 [Curriculum] 已启用课程学习: {}", c.summary());
    }

    let mut recent_ep_returns: VecDeque<f32> = VecDeque::with_capacity(50);
    let mut recent_ep_cs: VecDeque<f32> = VecDeque::with_capacity(50);
    let mut recent_ep_steps: VecDeque<usize> = VecDeque::with_capacity(50);

    // 注册保存通道
    let (save_tx, mut save_rx) = mpsc::unbounded_channel::<SaveRequest>();
    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.save_tx = Some(save_tx);
        }
    }

    let mut final_saved_iter = 0usize;
    let mut final_saved_return = 0.0f32;

    for iter in 1..=total_iterations {
        {
            let t = tasks.blocking_lock();
            if let Some(task) = t.get(&task_id) {
                if task.status != "running" {
                    break;
                }
            } else {
                break;
            }
        }

        // 处理实时保存模型请求
        while let Ok(req) = save_rx.try_recv() {
            persist_checkpoint(&task_id, req, engine.agent(), &tasks, &repo, &event_tx);
        }

        // 平滑余弦退火学习率 (Cosine Schedule)
        let progress = if total_iterations > 1 {
            (iter - 1) as f32 / (total_iterations - 1) as f32
        } else {
            1.0
        };
        let cos_progress = (1.0 + (std::f32::consts::PI * progress).cos()) * 0.5;
        let initial_lr = task_config.lr as f64;
        let current_lr = (initial_lr * 0.1
            + (initial_lr - initial_lr * 0.1) * (cos_progress as f64))
            .max(initial_lr * 0.05);

        // 3. 一次真实训练迭代（面向 TrainingEngine trait）
        let outcome = match engine.step_once(iter, current_lr, tuned.train_batch_size) {
            Ok(o) => o,
            Err(e) => {
                error!("训练迭代失败: {e}");
                break;
            }
        };

        let total_steps = engine.total_steps();
        let num_samples = outcome.num_samples;
        let sps = outcome.sps;
        let stats = outcome.stats;
        let mean_value = outcome.mean_value;

        // 回合回报/补刀/步数合并进近 50 轮滑动窗口
        for ret in outcome.ep_returns {
            if recent_ep_returns.len() >= 50 {
                recent_ep_returns.pop_front();
            }
            recent_ep_returns.push_back(ret);
        }
        for cs in outcome.ep_cs {
            if recent_ep_cs.len() >= 50 {
                recent_ep_cs.pop_front();
            }
            recent_ep_cs.push_back(cs);
        }
        for s in outcome.ep_steps {
            if recent_ep_steps.len() >= 50 {
                recent_ep_steps.pop_front();
            }
            recent_ep_steps.push_back(s);
        }

        let ep_return = if !recent_ep_returns.is_empty() {
            recent_ep_returns.iter().sum::<f32>() / recent_ep_returns.len() as f32
        } else {
            0.0
        };

        let ep_cs_avg = if !recent_ep_cs.is_empty() {
            recent_ep_cs.iter().sum::<f32>() / recent_ep_cs.len() as f32
        } else {
            0.0
        };

        // 课程学习调度：每轮根据步数和平均补刀更新课程状态并广播至 Worker
        if let Some(ref mut c) = curriculum_scheduler {
            c.tick(iter, ep_cs_avg);
            engine.update_curriculum(
                c.minion_hp_scale(),
                c.cs_reward(),
                c.attack_no_cs_penalty(),
                c.harass_coef(),
            );
        }

        final_saved_iter = iter;
        final_saved_return = ep_return;

        let (ep_steps_max, ep_steps_min, ep_steps_avg) = if !recent_ep_steps.is_empty() {
            let max = recent_ep_steps.iter().copied().max().unwrap_or(0);
            let min = recent_ep_steps.iter().copied().min().unwrap_or(0);
            let avg = recent_ep_steps.iter().sum::<usize>() as f32 / recent_ep_steps.len() as f32;
            (max, min, avg)
        } else {
            (0, 0, 0.0)
        };

        let real_reward_breakdown: Vec<RewardItem> = outcome
            .reward_breakdown
            .into_iter()
            .map(|(k, v)| RewardItem {
                name: k,
                value: v / (num_samples as f32).max(1.0),
            })
            .collect();

        {
            let mut t = tasks.blocking_lock();
            if let Some(task) = t.get_mut(&task_id) {
                task.current_step = total_steps;
                task.current_iter = iter;
                task.ep_return = ep_return;
            }
        }

        if iter % 5 == 0 {
            let _ = block_on_db(repo.update_progress(&task_id, total_steps as i64, ep_return));
        }

        let metric_row = lol_rl_protocol::MetricsRow {
            step: total_steps,
            ep_return,
            loss: stats.total_loss,
            policy_loss: stats.policy_loss,
            value_loss: stats.value_loss,
            total_loss: stats.total_loss,
            kl: stats.kl,
            entropy: stats.entropy,
            clip_frac: stats.clip_frac,
            value: mean_value,
            fps: sps as usize,
            ep_steps_max,
            ep_steps_min,
            ep_steps_avg,
            reward_breakdown: real_reward_breakdown.clone(),
        };

        {
            let mut t = tasks.blocking_lock();
            if let Some(task) = t.get_mut(&task_id) {
                task.metrics_history.push(metric_row.clone());
            }
        }

        let _ = block_on_db(repo.insert_metric(&task_id, &metric_row));

        let reward_formula = E::reward_formula_spec();

        let out_metrics = OutFrame::Metrics {
            task_id: task_id.clone(),
            step: total_steps,
            ep_return,
            loss: stats.policy_loss + stats.value_loss,
            policy_loss: stats.policy_loss,
            value_loss: stats.value_loss,
            total_loss: stats.total_loss,
            kl: stats.kl,
            entropy: stats.entropy,
            clip_frac: stats.clip_frac,
            clip_eps: task_config.clip_eps,
            value: mean_value,
            fps: sps as usize,
            ep_steps_max,
            ep_steps_min,
            ep_steps_avg,
            reward_breakdown: real_reward_breakdown,
            obs_feature: outcome.obs_payload,
            reward_formula,
            reward_variables: Some(outcome.last_reward_variables),
            curriculum: curriculum_scheduler
                .as_ref()
                .map(|c| c.to_telemetry(ep_cs_avg)),
        };

        let _ = event_tx.send(out_metrics);

        // 每 10 次迭代自动保存一个 Checkpoint
        if iter % 10 == 0 {
            let ckpt_id = format!("iter-{}", iter);
            let path = new_checkpoint_path(&task_id, &ckpt_id)
                .to_string_lossy()
                .to_string();
            persist_checkpoint(
                &task_id,
                SaveRequest {
                    ckpt_id,
                    path,
                    ep_return,
                },
                engine.agent(),
                &tasks,
                &repo,
                &event_tx,
            );
        }

        if iter % 5 == 0 || iter == 1 {
            let curriculum_tag = if let Some(ref c) = curriculum_scheduler {
                format!(" | [{}]", c.summary())
            } else {
                String::new()
            };
            let log_msg = format!(
                "[{}] Iter {:2}/{} | SPS: {:6.1} | Reward: {:6.2} | CS: {:4.1} | P-Loss: {:7.4} | V-Loss: {:7.4}{}",
                task_id,
                iter,
                total_iterations,
                sps,
                ep_return,
                ep_cs_avg,
                stats.policy_loss,
                stats.value_loss,
                curriculum_tag,
            );
            {
                let mut t = tasks.blocking_lock();
                if let Some(task) = t.get_mut(&task_id) {
                    task.logs.push(format!("[info] {}", log_msg));
                }
            }
            let _ = block_on_db(repo.insert_log(&task_id, "info", &log_msg));
            let _ = event_tx.send(OutFrame::Log {
                task_id: task_id.clone(),
                level: "info".into(),
                message: log_msg,
            });
        }
    }

    // 关闭 Worker 线程池
    engine.stop();

    // 4. 训练收敛：自动保存最终模型
    let ckpt_id = format!("iter-{}", final_saved_iter);
    let path = new_checkpoint_path(&task_id, &ckpt_id)
        .to_string_lossy()
        .to_string();
    persist_checkpoint(
        &task_id,
        SaveRequest {
            ckpt_id,
            path,
            ep_return: final_saved_return,
        },
        engine.agent(),
        &tasks,
        &repo,
        &event_tx,
    );

    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.status = "finished".to_string();
            task.save_tx = None;
        }
    }
    let _ = block_on_db(repo.update_status(&task_id, "finished"));
    let _ = event_tx.send(OutFrame::Status {
        task_id: task_id.clone(),
        status: "finished".into(),
    });
    let _ = event_tx.send(OutFrame::Log {
        task_id,
        level: "info".into(),
        message: "[lol_rl] PPO 训练任务已收敛完成！".into(),
    });
}

/// 直接启动环境训练会话（无需外部数据库），内部复用完整的生产级通用训练循环。
/// 可供命令行、快速验证 bin、自动化基准测试直接调用。
pub fn run_direct_training<E: lol_env::RlEnvironment + 'static>(
    task_config: TaskConfigPayload,
) -> (broadcast::Receiver<OutFrame>, std::thread::JoinHandle<()>) {
    let task_id = uuid::Uuid::new_v4().to_string();
    let (event_tx, rx) = broadcast::channel(512);
    let repo = Arc::new(crate::db::NoopRlRepo);
    let mut initial_tasks = HashMap::new();
    initial_tasks.insert(
        task_id.clone(),
        TaskState {
            id: task_id.clone(),
            name: task_config.name.clone(),
            agent_type: format!(
                "{} ({})",
                task_config.algorithm.display_name(),
                task_config.backbone.display_name()
            ),
            env_name: task_config.env_name.clone(),
            status: "running".to_string(),
            current_step: 0,
            current_iter: 0,
            ep_return: 0.0,
            config: task_config.clone(),
            checkpoints: Vec::new(),
            metrics_history: Vec::new(),
            logs: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            save_tx: None,
        },
    );
    let tasks = Arc::new(Mutex::new(initial_tasks));

    let handle = std::thread::spawn(move || {
        run_generic_training_loop::<E>(event_tx, tasks, repo, task_id, task_config);
    });

    (rx, handle)
}
