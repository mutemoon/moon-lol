use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use candle_core::Tensor;
use chrono::Utc;
pub use lol_rl_protocol::{
    CheckpointItem, InFrame, ObsFeaturePayload, OutFrame, RewardItem, TaskConfigPayload,
    TaskOverviewItem,
};
use sqlx::PgPool;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::{error, info};
use uuid::Uuid;

use crate::autotune::AutoTuner;
use crate::db::{self, CheckpointRow, PgRlRepo, RlRepo, TaskRow};
use crate::model_store::{checkpoint_dir, new_checkpoint_path};
use crate::ppo::{PPOAgent, PPOConfig, RolloutBuffer};
use crate::worker::TrainingWorkerPool;

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
        let interrupted = repo.mark_all_running_interrupted().await?;
        info!("连接 Postgres…恢复 {} 个中断任务", interrupted);

        let db_tasks = repo.list_tasks().await?;
        let mut initial_tasks = HashMap::new();
        for t in &db_tasks {
            let mut state = TaskState::from_row(t);
            let cps = repo.list_checkpoints(&state.id).await?;
            for cp in &cps {
                state.checkpoints.push(CheckpointItem {
                    // 统一用展示 id "ckpt-{step}"，与运行中保存的一致
                    id: format!("ckpt-{}", cp.step),
                    step: cp.step as usize,
                    path: cp.path.clone(),
                    ep_return: cp.ep_return,
                    created_at: cp.created_at.to_rfc3339(),
                });
            }
            if let Ok(metrics) = repo.list_metrics(&state.id).await {
                state.metrics_history = metrics;
            }
            if let Ok(logs) = repo.list_logs(&state.id).await {
                state.logs = logs;
            }
            initial_tasks.insert(state.id.clone(), state);
        }
        info!("从数据库载入 {} 个任务", initial_tasks.len());

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
                let row = TaskRow {
                    id: task_id,
                    name: config.name.clone(),
                    agent_type: config.agent_type.clone(),
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
                    agent_type: config.agent_type.clone(),
                    env_name: config.env_name.clone(),
                    status: "queued".to_string(),
                    current_step: 0,
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
                    .map(|cp| CheckpointItem {
                        id: format!("ckpt-{}", cp.step),
                        step: cp.step as usize,
                        path: cp.path,
                        ep_return: cp.ep_return,
                        created_at: cp.created_at.to_rfc3339(),
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
        let (save_tx, step, ep_return) = {
            let tasks = self.tasks.lock().await;
            match tasks.get(task_id) {
                Some(t) => (t.save_tx.clone(), t.current_step, t.ep_return),
                None => return,
            }
        };
        let ckpt_id = format!("ckpt-{}", step);
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
        match self.repo.get_checkpoint(task_id, ckpt_id).await {
            Ok(Some(cp)) => {
                let item = CheckpointItem {
                    // 与保存流程一致，返回展示 id "ckpt-{step}"，供前端匹配 running_visual_model
                    id: ckpt_id.to_string(),
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
                agent_type: t.agent_type.clone(),
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

/// 在 `spawn_blocking` 线程内复用当前 tokio runtime 执行一次异步 DB 写，
/// 避免每处都 `Builder::new_current_thread()` 新建临时 runtime。
fn block_on_db<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Handle::current().block_on(fut)
}

/// 写 safetensors 权重文件 + 登记 checkpoint 到 DB + 广播。仅在训练循环内（agent 存活时）调用。
fn persist_checkpoint(
    task_id: &str,
    req: SaveRequest,
    agent: &PPOAgent,
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

    let step = req
        .ckpt_id
        .strip_prefix("ckpt-")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let now = Utc::now();

    // 同步写入自描述元数据 JSON（包含环境、输入输出维度与训练指标）
    let meta_path = Path::new(&req.path).with_extension("meta.json");
    let task_meta_opt = {
        let t = tasks.blocking_lock();
        t.get(task_id).map(|task| (task.env_name.clone(), task.config.hidden_dim, task.agent_type.clone()))
    };
    if let Some((env_name, hidden_dim, agent_type)) = task_meta_opt {
        let meta_json = serde_json::json!({
            "task_id": task_id,
            "ckpt_id": req.ckpt_id,
            "step": step,
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
        step: step as i64,
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
        step,
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
                    run_generic_training_loop::<lol_env::FioraVsRivenRealEnv>(
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
    let hidden_dim = task_config.hidden_dim.max(64);
    let device = crate::device::select_device().unwrap_or(candle_core::Device::Cpu);

    // 1. 自动吞吐探测与求解
    let tuned = match AutoTuner::profile::<E>(state_dim, hidden_dim, &action_space, &device) {
        Ok(profile) => {
            let res = AutoTuner::solve(&profile, rollout_steps, task_config.ppo_epochs.max(1));
            info!(
                "🎯 [AutoTuner] 为任务 {} 自动求解最优吞吐配置: 并发 Actors={}, 推理 Batch={}, 训练 MiniBatch={}, 预估 SPS: {:.1}",
                task_id,
                res.num_parallel_envs,
                res.infer_batch_size,
                res.train_batch_size,
                res.estimated_sps
            );
            res
        }
        Err(e) => {
            let fallback_actors = if task_config.parallel_envs > 0 {
                task_config.parallel_envs
            } else {
                num_cpus::get().clamp(2, 16)
            };
            tracing::warn!("AutoTuner 探测失败 ({e}), 降级使用默认配置: {fallback_actors}");
            crate::autotune::TunedConfig {
                num_parallel_envs: fallback_actors,
                infer_batch_size: fallback_actors.min(32),
                train_batch_size: (fallback_actors * 16).clamp(32, 256),
                dynamic_batch_timeout_us: 200,
                estimated_sps: 2000.0,
            }
        }
    };

    let num_parallel_envs = if task_config.parallel_envs > 0 {
        task_config.parallel_envs
    } else {
        tuned.num_parallel_envs
    };

    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.config.parallel_envs = num_parallel_envs;
        }
    }

    let ppo_config = PPOConfig {
        lr: task_config.lr as f64,
        gamma: task_config.gamma,
        gae_lambda: task_config.gae_lambda,
        clip_eps: task_config.clip_eps,
        c1: 0.5,
        c2: 0.05,
        ppo_epochs: task_config.ppo_epochs.max(1),
        clip_vloss: true,
        max_grad_norm: 0.5,
    };

    let mut agent = match PPOAgent::new(
        state_dim,
        hidden_dim,
        action_space.clone(),
        ppo_config,
        device.clone(),
    ) {
        Ok(a) => a,
        Err(e) => {
            error!("创建 PPOAgent 失败: {e}");
            return;
        }
    };

    // 2. 启动长驻持久化 Worker 线程池（环境只在任务启动时初始化一次）
    let horizon = rollout_steps;
    struct WorkerTrajectory<O> {
        buffer: RolloutBuffer,
        last_value: f32,
        ep_returns: Vec<f32>,
        completed_steps: Vec<usize>,
        reward_breakdown: HashMap<String, f32>,
        last_reward_variables: HashMap<String, f32>,
        last_obs: Option<O>,
    }

    enum WorkerCommand {
        Rollout(Arc<crate::policy::ActorCritic>),
        Stop,
    }

    let mut cmd_senders = Vec::with_capacity(num_parallel_envs);
    let mut resp_receivers = Vec::with_capacity(num_parallel_envs);
    let mut thread_handles = Vec::with_capacity(num_parallel_envs);

    for _ in 0..num_parallel_envs {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<WorkerCommand>();
        let (resp_tx, resp_rx) = crossbeam_channel::unbounded::<WorkerTrajectory<E::Obs>>();

        let handle = std::thread::spawn(move || {
            let mut env = E::new();
            let mut current_obs = env.reset();
            let mut cur_return = 0.0f32;
            let mut cur_steps = 0usize;

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    WorkerCommand::Rollout(policy) => {
                        let mut buffer = RolloutBuffer::new();
                        let mut ep_returns = Vec::new();
                        let mut completed_steps = Vec::new();
                        let mut reward_breakdown = HashMap::new();
                        let mut last_reward_variables = HashMap::new();

                        for _ in 0..horizon {
                            let state_vec = E::obs_to_vector(&current_obs);
                            let action_mask = E::action_mask(&current_obs);
                            let state_tensor = match Tensor::from_vec(
                                state_vec.clone(),
                                (1, state_dim),
                                &candle_core::Device::Cpu,
                            ) {
                                Ok(t) => t,
                                Err(_) => break,
                            };

                            let (encoded, log_prob, val) =
                                match policy.sample_action(&state_tensor, action_mask.as_deref()) {
                                    Ok(res) => res,
                                    Err(_) => break,
                                };

                            let act = E::action_from_encoding(&encoded);
                            let res = env.step(act);
                            let done = res.terminated || res.truncated;

                            cur_return += res.reward;
                            cur_steps += 1;

                            if !res.reward_variables.is_empty() {
                                last_reward_variables = res.reward_variables;
                            }
                            for item in res.reward_breakdown {
                                *reward_breakdown.entry(item.name).or_insert(0.0) += item.value;
                            }

                            buffer.push(
                                state_vec,
                                encoded,
                                log_prob,
                                res.reward,
                                val,
                                done,
                                action_mask,
                            );

                            if done {
                                ep_returns.push(cur_return);
                                completed_steps.push(cur_steps);
                                cur_return = 0.0;
                                cur_steps = 0;
                                current_obs = env.reset();
                            } else {
                                current_obs = res.obs;
                            }
                        }

                        let last_state_vec = E::obs_to_vector(&current_obs);
                        let last_state_tensor = Tensor::from_vec(
                            last_state_vec.clone(),
                            (1, state_dim),
                            &candle_core::Device::Cpu,
                        )
                        .unwrap();
                        let last_value = policy
                            .get_values(&last_state_tensor)
                            .map(|v| v.first().copied().unwrap_or(0.0))
                            .unwrap_or(0.0);

                        let _ = resp_tx.send(WorkerTrajectory {
                            buffer,
                            last_value,
                            ep_returns,
                            completed_steps,
                            reward_breakdown,
                            last_reward_variables,
                            last_obs: Some(current_obs.clone()),
                        });
                    }
                    WorkerCommand::Stop => break,
                }
            }
        });

        cmd_senders.push(cmd_tx);
        resp_receivers.push(resp_rx);
        thread_handles.push(handle);
    }

    let mut total_steps = 0usize;
    let mut recent_ep_returns: std::collections::VecDeque<f32> =
        std::collections::VecDeque::with_capacity(50);

    // 注册保存通道
    let (save_tx, mut save_rx) = mpsc::unbounded_channel::<SaveRequest>();
    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.save_tx = Some(save_tx);
        }
    }

    let mut final_saved_step = 0usize;
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
            persist_checkpoint(&task_id, req, &agent, &tasks, &repo, &event_tx);
        }

        let iter_start = Instant::now();

        // 动态退火策略熵与学习率 (PPO2 Industrial Standard)
        let progress = if total_iterations > 1 {
            (iter - 1) as f32 / (total_iterations - 1) as f32
        } else {
            1.0
        };
        let current_c2 = (0.05 * (1.0 - progress) + 0.001 * progress).max(0.001);
        agent.set_entropy_coef(current_c2);

        let initial_lr = task_config.lr as f64;
        let current_lr = (initial_lr * (1.0 - progress as f64)).max(initial_lr * 0.05);
        let _ = agent.set_lr(current_lr);

        // 1. 克隆 CPU 采样策略
        let cpu_policy = match agent.actor_critic.to_device(&candle_core::Device::Cpu) {
            Ok(p) => Arc::new(p),
            Err(e) => {
                error!("迁移策略至 CPU 失败: {e}");
                break;
            }
        };

        // 2. 触发持久化 Worker 并行采样
        for tx in &cmd_senders {
            let _ = tx.send(WorkerCommand::Rollout(cpu_policy.clone()));
        }

        let mut env_buffers = Vec::with_capacity(num_parallel_envs);
        let mut last_values = Vec::with_capacity(num_parallel_envs);
        let mut completed_ep_steps = Vec::new();
        let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();
        let mut last_reward_variables = HashMap::new();
        let mut sample_obs: Option<E::Obs> = None;

        for rx in &resp_receivers {
            let traj = match rx.recv() {
                Ok(t) => t,
                Err(_) => break,
            };

            for ret in traj.ep_returns {
                if recent_ep_returns.len() >= 50 {
                    recent_ep_returns.pop_front();
                }
                recent_ep_returns.push_back(ret);
            }
            completed_ep_steps.extend(traj.completed_steps);
            for (k, v) in traj.reward_breakdown {
                *iter_reward_breakdown.entry(k).or_insert(0.0) += v;
            }
            if !traj.last_reward_variables.is_empty() {
                last_reward_variables = traj.last_reward_variables;
            }
            if sample_obs.is_none() {
                sample_obs = traj.last_obs;
            }

            env_buffers.push(traj.buffer);
            last_values.push(traj.last_value);
        }

        let num_samples = num_parallel_envs * horizon;
        total_steps += num_samples;

        // 3. GPU Mini-Batch PPO 更新
        let stats =
            match agent.update_multi_buffer(&env_buffers, &last_values, tuned.train_batch_size) {
                Ok(s) => s,
                Err(e) => {
                    error!("PPO update_multi_buffer 失败: {e}");
                    break;
                }
            };

        let elapsed_sec = iter_start.elapsed().as_secs_f64();
        let sps = (num_samples as f64) / elapsed_sec.max(0.0001);

        let ep_return = if !recent_ep_returns.is_empty() {
            recent_ep_returns.iter().sum::<f32>() / recent_ep_returns.len() as f32
        } else {
            0.0
        };

        final_saved_step = total_steps;
        final_saved_return = ep_return;

        let (ep_steps_max, ep_steps_min, ep_steps_avg) = if !completed_ep_steps.is_empty() {
            let max = completed_ep_steps.iter().copied().max().unwrap_or(0);
            let min = completed_ep_steps.iter().copied().min().unwrap_or(0);
            let avg =
                completed_ep_steps.iter().sum::<usize>() as f32 / completed_ep_steps.len() as f32;
            (max, min, avg)
        } else {
            (0, 0, 0.0)
        };

        let real_reward_breakdown: Vec<RewardItem> = iter_reward_breakdown
            .into_iter()
            .map(|(k, v)| RewardItem {
                name: k,
                value: v / (num_samples as f32).max(1.0),
            })
            .collect();

        let val_sum: f32 = env_buffers
            .iter()
            .map(|b| b.values.iter().sum::<f32>())
            .sum();
        let val_cnt: usize = env_buffers.iter().map(|b| b.values.len()).sum();
        let mean_value = val_sum / (val_cnt as f32).max(1.0);

        {
            let mut t = tasks.blocking_lock();
            if let Some(task) = t.get_mut(&task_id) {
                task.current_step = total_steps;
                task.ep_return = ep_return;
            }
        }

        if iter % 5 == 0 {
            let _ = block_on_db(repo.update_progress(&task_id, total_steps as i64, ep_return));
        }

        let metric_row = lol_rl_protocol::MetricsRow {
            step: total_steps,
            ep_return,
            loss: stats.policy_loss + stats.value_loss,
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

        let obs_payload = sample_obs.as_ref().and_then(|o| E::obs_to_payload(o));

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
            obs_feature: obs_payload,
            reward_formula,
            reward_variables: Some(last_reward_variables),
        };

        let _ = event_tx.send(out_metrics);

        // 每 10 次迭代自动保存一个 Checkpoint
        if iter % 10 == 0 {
            let ckpt_id = format!("ckpt-{}", total_steps);
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
                &agent,
                &tasks,
                &repo,
                &event_tx,
            );
        }

        if iter % 5 == 0 || iter == 1 {
            let log_msg = format!(
                "[{}] Iter {:2}/{} | SPS: {:6.1} | Reward: {:6.2} | P-Loss: {:7.4} | V-Loss: {:7.4}",
                task_id,
                iter,
                total_iterations,
                sps,
                ep_return,
                stats.policy_loss,
                stats.value_loss
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
    for tx in cmd_senders {
        let _ = tx.send(WorkerCommand::Stop);
    }
    for h in thread_handles {
        let _ = h.join();
    }

    // 4. 训练收敛：自动保存最终模型
    let ckpt_id = format!("ckpt-{}", final_saved_step);
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
        &agent,
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
