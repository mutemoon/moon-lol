use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use candle_core::Tensor;
use chrono::Utc;
use lol_env::RewardModel;
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
        let task_items: Vec<TaskOverviewItem> = tasks_lock
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

    if task_config.env_name == lol_rl_protocol::ENV_FIORA_VS_RIVEN_LEGACY {
        run_generic_training_loop::<lol_env::FioraVsRivenEnv>(
            event_tx,
            tasks,
            repo,
            task_id,
            task_config,
        );
    } else {
        // Default to real movement environment
        run_generic_training_loop::<lol_env::FioraVsRivenRealEnv>(
            event_tx,
            tasks,
            repo,
            task_id,
            task_config,
        );
    }
}

fn run_generic_training_loop<E: lol_env::RlEnvironment>(
    event_tx: broadcast::Sender<OutFrame>,
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    repo: Arc<dyn RlRepo>,
    task_id: String,
    task_config: TaskConfigPayload,
) {
    // 单局步数截断（来自任务配置 rollout_steps_per_env），促使 AI 快速打破绽击杀，避免无限移动刷分
    let env_max_steps = task_config.rollout_steps_per_env.max(1);
    let state_dim = E::state_dim();
    let action_space = E::action_space();
    let enc_dim = action_space.encoding_dim();
    let total_iterations = task_config.total_iterations.max(1);
    let hidden_dim = task_config.hidden_dim.max(64);
    let device = crate::device::select_device().unwrap_or(candle_core::Device::Cpu);

    let num_parallel_envs = if task_config.parallel_envs > 0 {
        task_config.parallel_envs
    } else {
        match AutoTuner::profile::<E>(state_dim, hidden_dim, &action_space, &device) {
            Ok(profile) => {
                let tuned =
                    AutoTuner::solve(&profile, env_max_steps, task_config.ppo_epochs.max(1));
                info!(
                    "🎯 [AutoTuner] 为任务 {} 自动求解最优吞吐并发: N={}, 预估 SPS: {:.1}",
                    task_id, tuned.num_parallel_envs, tuned.estimated_sps
                );
                {
                    let mut t = tasks.blocking_lock();
                    if let Some(task) = t.get_mut(&task_id) {
                        task.config.parallel_envs = tuned.num_parallel_envs;
                    }
                }
                tuned.num_parallel_envs
            }
            Err(e) => {
                let fallback = num_cpus::get().clamp(2, 16);
                tracing::warn!("AutoTuner 探测失败 ({e}), 降级使用默认并发数: {fallback}");
                fallback
            }
        }
    };

    let config = PPOConfig {
        lr: task_config.lr as f64,
        gamma: task_config.gamma,
        gae_lambda: task_config.gae_lambda,
        clip_eps: task_config.clip_eps,
        c1: 0.5,
        c2: 0.05,
        ppo_epochs: task_config.ppo_epochs.max(1),
    };

    let mut agent = match PPOAgent::new(state_dim, hidden_dim, action_space, config, device.clone())
    {
        Ok(a) => a,
        Err(e) => {
            error!("创建 PPOAgent 失败: {e}");
            return;
        }
    };

    let par_envs = lol_env::ParallelEnvs::<E>::new(num_parallel_envs, env_max_steps);
    let mut env_buffers = Vec::with_capacity(num_parallel_envs);
    for _ in 0..num_parallel_envs {
        env_buffers.push(RolloutBuffer::new());
    }
    let mut current_obss = par_envs.reset_all();
    let mut env_returns = vec![0.0f32; num_parallel_envs];
    let mut recent_ep_returns: std::collections::VecDeque<f32> =
        std::collections::VecDeque::with_capacity(50);
    // 每环境当前对局已执行步数，对局结束时记入 iter_ep_steps 用于统计单局步数
    let mut env_steps = vec![0usize; num_parallel_envs];
    let mut iter_ep_steps: Vec<usize> = Vec::with_capacity(num_parallel_envs);
    // 累计环境步数（跨迭代累加，用作 step 坐标与 max_steps 总步数预算）
    let mut total_steps = 0usize;

    // 注册保存通道，供「保存模型」请求驱动实时落盘
    let (save_tx, mut save_rx) = mpsc::unbounded_channel::<SaveRequest>();
    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.save_tx = Some(save_tx);
        }
    }

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

        // 达到总步数预算则停止（max_steps > 0 时生效）
        if task_config.max_steps > 0 && total_steps >= task_config.max_steps {
            break;
        }

        // 处理「保存模型」请求：把当前权重写盘
        while let Ok(req) = save_rx.try_recv() {
            persist_checkpoint(&task_id, req, &agent, &tasks, &repo, &event_tx);
        }

        // 动态衰减熵系数（Entropy Annealing）：从 0.05 线性衰减至 0.001，使训练末期策略逼近确定性/贪婪策略
        let progress = if total_iterations > 1 {
            (iter - 1) as f32 / (total_iterations - 1) as f32
        } else {
            1.0
        };
        let current_c2 = (0.05 * (1.0 - progress) + 0.001 * progress).max(0.001);
        agent.set_entropy_coef(current_c2);

        let iter_start = Instant::now();

        for b in &mut env_buffers {
            b.clear();
        }
        let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();
        let mut last_reward_variables: HashMap<String, f32> = HashMap::new();
        let mut iter_value_sum = 0.0f32;
        let mut completed_envs = vec![false; num_parallel_envs];
        let mut iter_steps_count = 0usize;

        // 采集完整对局：持续采样直到所有并行环境本局全部结束（terminated 或 truncated）
        while !completed_envs.iter().all(|&c| c) {
            let mut actions = Vec::with_capacity(num_parallel_envs);
            let mut action_encodings = Vec::with_capacity(num_parallel_envs);
            let mut log_probs = Vec::with_capacity(num_parallel_envs);
            let mut values = Vec::with_capacity(num_parallel_envs);

            for i in 0..num_parallel_envs {
                if completed_envs[i] {
                    actions.push(E::action_from_encoding(&vec![0.0; enc_dim]));
                    action_encodings.push(vec![0.0; enc_dim]);
                    log_probs.push(0.0);
                    values.push(0.0);
                    continue;
                }

                let obs = &current_obss[i];
                let state_vec = E::obs_to_vector(obs);
                let state_tensor =
                    match Tensor::from_vec(state_vec.clone(), (1, state_dim), &device) {
                        Ok(t) => t,
                        Err(_) => break,
                    };
                if let Ok((encoded, log_prob, val)) =
                    agent.actor_critic.sample_action(&state_tensor, &state_vec)
                {
                    actions.push(E::action_from_encoding(&encoded));
                    action_encodings.push(encoded);
                    log_probs.push(log_prob);
                    values.push(val);
                } else {
                    actions.push(E::action_from_encoding(&vec![0.0; enc_dim]));
                    action_encodings.push(vec![0.0; enc_dim]);
                    log_probs.push(0.0);
                    values.push(0.0);
                }
            }

            let step_results = par_envs.step_all(&actions);
            for i in 0..num_parallel_envs {
                if completed_envs[i] {
                    continue;
                }

                let res = &step_results[i];
                env_returns[i] += res.reward;
                iter_steps_count += 1;
                env_steps[i] += 1;
                iter_value_sum += values[i];
                last_reward_variables = res.reward_variables.clone();

                for item in &res.reward_breakdown {
                    *iter_reward_breakdown
                        .entry(item.name.clone())
                        .or_insert(0.0) += item.value;
                }

                env_buffers[i].push(
                    E::obs_to_vector(&current_obss[i]),
                    action_encodings[i].clone(),
                    log_probs[i],
                    res.reward,
                    values[i],
                    res.terminated || res.truncated,
                );

                if res.terminated || res.truncated {
                    let ep_ret = env_returns[i];
                    env_returns[i] = 0.0;
                    if recent_ep_returns.len() >= 50 {
                        recent_ep_returns.pop_front();
                    }
                    recent_ep_returns.push_back(ep_ret);
                    iter_ep_steps.push(env_steps[i]);
                    env_steps[i] = 0;
                    current_obss[i] = par_envs.reset_one(i);
                    completed_envs[i] = true;
                } else {
                    current_obss[i] = res.obs.clone();
                }
            }
        }

        let mut combined_buffer = RolloutBuffer::new();
        for b in &env_buffers {
            for t in 0..b.len() {
                combined_buffer.push(
                    b.states[t].clone(),
                    b.actions[t].clone(),
                    b.log_probs[t],
                    b.rewards[t],
                    b.values[t],
                    b.dones[t],
                );
            }
        }

        // 完整对局结束，终态 Bootstrap value 为 0.0（仅用于 GAE 计算）
        let last_val_scalar = 0.0f32;

        // 迭代内各状态 critic 预测值均值，作为「价值」指标上报
        let mean_value = iter_value_sum / iter_steps_count.max(1) as f32;

        // 本迭代各局步数统计（每环境一局，均为一整局）
        let ep_steps_max = iter_ep_steps.iter().copied().max().unwrap_or(0);
        let ep_steps_min = iter_ep_steps.iter().copied().min().unwrap_or(0);
        let ep_steps_avg = if iter_ep_steps.is_empty() {
            0.0
        } else {
            iter_ep_steps.iter().sum::<usize>() as f32 / iter_ep_steps.len() as f32
        };

        total_steps += iter_steps_count;

        if let Ok(stats) = agent.update(&combined_buffer, last_val_scalar) {
            let ep_return = if !recent_ep_returns.is_empty() {
                let sum: f32 = recent_ep_returns.iter().sum();
                sum / recent_ep_returns.len() as f32
            } else {
                let sum: f32 = env_returns.iter().sum();
                sum / num_parallel_envs.max(1) as f32
            };

            {
                let mut t = tasks.blocking_lock();
                if let Some(task) = t.get_mut(&task_id) {
                    task.current_step = total_steps;
                    task.ep_return = ep_return;
                }
            }

            // Update DB progress periodically
            if iter % 5 == 0 {
                let _ = block_on_db(repo.update_progress(&task_id, total_steps as i64, ep_return));
            }

            let sample_obs = &current_obss[0];
            let obs_payload = E::obs_to_payload(sample_obs);

            // 本迭代环境步进速率（env steps/sec），即训练吞吐
            let fps = (iter_steps_count.max(1) as f64
                / iter_start.elapsed().as_secs_f64().max(1e-5)) as usize;

            let real_reward_breakdown: Vec<RewardItem> = iter_reward_breakdown
                .iter()
                .map(|(k, v)| RewardItem {
                    name: k.clone(),
                    value: v / (iter_steps_count as f32).max(1.0),
                })
                .collect();

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
                fps,
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

            let reward_model = lol_env::reward::FioraVsRivenRewardModel;
            let reward_formula = reward_model.formula_spec();

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
                fps,
                ep_steps_max,
                ep_steps_min,
                ep_steps_avg,
                reward_breakdown: real_reward_breakdown,
                obs_feature: obs_payload,
                reward_formula: Some(reward_formula),
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
                    "[{}] Iter {:2}/{} | Avg Reward: {:6.2} | P-Loss: {:7.4} | V-Loss: {:7.4}",
                    task_id, iter, total_iterations, ep_return, stats.policy_loss, stats.value_loss
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
    }

    // 训练收敛：自动保存最终模型
    {
        let (final_step, ep_return) = {
            let t = tasks.blocking_lock();
            t.get(&task_id)
                .map(|x| (x.current_step, x.ep_return))
                .unwrap_or((0, 0.0))
        };
        let ckpt_id = format!("ckpt-{}", final_step);
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
