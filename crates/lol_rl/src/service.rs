use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use candle_core::Tensor;
use chrono::Utc;
use lol_env::fiora_vs_riven::{FioraVsRivenAction, FioraVsRivenObs};
use lol_env::parallel::ParallelFioraVsRivenEnvs;
pub use lol_rl_protocol::{
    CheckpointItem, InFrame, ObsFeaturePayload, OutFrame, PolicyItem, RewardItem,
    TaskConfigPayload, TaskOverviewItem,
};
use sqlx::PgPool;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::{error, info};
use uuid::Uuid;

use crate::db::{self, CheckpointRow, PgRlRepo, RlRepo, TaskRow};
use crate::model_store::new_checkpoint_path;
use crate::ppo::{PPOAgent, PPOConfig, RolloutBuffer};

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
    pub created_at: String,
}

impl TaskState {
    fn from_row(r: &TaskRow) -> Self {
        let config: TaskConfigPayload = serde_json::from_value(r.config_json.clone())
            .unwrap_or_else(|_| TaskConfigPayload {
                name: r.name.clone(),
                agent_type: r.agent_type.clone(),
                env_name: r.env_name.clone(),
                lr: 5e-4,
                parallel_envs: 4,
                max_steps: 10000,
            });
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
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

pub struct RLService {
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    event_tx: broadcast::Sender<OutFrame>,
    repo: Arc<dyn RlRepo>,
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
                    id: cp.id.to_string(),
                    step: cp.step as usize,
                    path: cp.path.clone(),
                    ep_return: cp.ep_return,
                    created_at: cp.created_at.to_rfc3339(),
                });
            }
            initial_tasks.insert(state.id.clone(), state);
        }
        info!("从数据库载入 {} 个任务", initial_tasks.len());

        let (event_tx, rx) = broadcast::channel(event_capacity);
        Ok((
            Self {
                tasks: Arc::new(Mutex::new(initial_tasks)),
                event_tx,
                repo,
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
                    created_at: now.to_rfc3339(),
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
                tokio::task::spawn_blocking(move || {
                    run_training_loop_for_task(event_tx, tasks_arc, repo, tid_clone);
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
                    tokio::task::spawn_blocking(move || {
                        run_training_loop_for_task(event_tx, tasks_arc, repo, tid);
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
        }
    }

    async fn handle_save_checkpoint(&self, task_id: &str) {
        let tasks_guard = self.tasks.lock().await;
        let Some(t) = tasks_guard.get(task_id) else {
            return;
        };
        let step = t.current_step;
        let ep_return = t.ep_return;
        drop(tasks_guard);

        let ckpt_id = format!("ckpt-{}", step);
        let ckpt_path = new_checkpoint_path(task_id, &ckpt_id);
        let path_str = ckpt_path.to_string_lossy().to_string();

        let cp_id = Uuid::new_v4();
        let now = Utc::now();
        let cp_row = CheckpointRow {
            id: cp_id,
            task_id: Uuid::parse_str(task_id).unwrap_or_default(),
            step: step as i64,
            path: path_str.clone(),
            ep_return,
            created_at: now,
        };
        if let Err(e) = self.repo.insert_checkpoint(&cp_row).await {
            error!("写入 checkpoint DB 失败: {e}");
            return;
        }

        let ckpt_item = CheckpointItem {
            id: ckpt_id,
            step,
            path: path_str,
            ep_return,
            created_at: now.to_rfc3339(),
        };

        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.checkpoints.push(ckpt_item.clone());
        }
        drop(tasks);

        let _ = self.event_tx.send(OutFrame::CheckpointMsg {
            task_id: task_id.to_string(),
            checkpoint: ckpt_item.clone(),
        });
        let _ = self.event_tx.send(OutFrame::Log {
            task_id: task_id.to_string(),
            level: "info".into(),
            message: format!("已为任务 {} 保存 Checkpoint 到 {}", task_id, ckpt_item.path),
        });
    }

    async fn handle_apply_checkpoint(&self, task_id: &str, ckpt_id: &str) {
        match self.repo.get_checkpoint(task_id, ckpt_id).await {
            Ok(Some(cp)) => {
                let item = CheckpointItem {
                    id: cp.id.to_string(),
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

fn run_training_loop_for_task(
    event_tx: broadcast::Sender<OutFrame>,
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    repo: Arc<dyn RlRepo>,
    task_id: String,
) {
    let env_max_steps = 100;
    let state_dim = FioraVsRivenObs::dim();
    let action_dim = 9;
    let num_parallel_envs = 4;
    let rollout_steps_per_env = 80;
    let total_iterations = 80;

    let config = PPOConfig {
        lr: 5e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        c2: 0.05,
        ppo_epochs: 4,
    };

    let device = crate::device::select_device().unwrap_or(candle_core::Device::Cpu);

    let mut agent = match PPOAgent::new(state_dim, 64, action_dim, config, device.clone()) {
        Ok(a) => a,
        Err(e) => {
            error!("创建 PPOAgent 失败: {e}");
            return;
        }
    };

    let par_envs = ParallelFioraVsRivenEnvs::new(num_parallel_envs, env_max_steps);
    let mut buffer = RolloutBuffer::new();
    let mut current_obss = par_envs.reset_all();

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

        let iter_start = Instant::now();

        buffer.clear();
        let mut iter_reward_sum = 0.0;
        let mut iter_episodes = 0;
        let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();

        for _step in 0..rollout_steps_per_env {
            let mut actions = Vec::with_capacity(num_parallel_envs);
            let mut action_indices = Vec::with_capacity(num_parallel_envs);
            let mut log_probs = Vec::with_capacity(num_parallel_envs);
            let mut values = Vec::with_capacity(num_parallel_envs);

            for obs in &current_obss {
                let state_vec = obs.to_vector();
                let state_tensor =
                    match Tensor::from_vec(state_vec.clone(), (1, state_dim), &device) {
                        Ok(t) => t,
                        Err(_) => break,
                    };
                if let Ok((act_idx, log_prob, val)) = agent
                    .actor_critic
                    .select_action_masked(&state_tensor, &state_vec)
                {
                    actions.push(FioraVsRivenAction::from_index(act_idx));
                    action_indices.push(act_idx);
                    log_probs.push(log_prob);
                    values.push(val);
                } else {
                    actions.push(FioraVsRivenAction::from_index(0));
                    action_indices.push(0);
                    log_probs.push(0.0);
                    values.push(0.0);
                }
            }

            let step_results = par_envs.step_all(&actions);
            for i in 0..num_parallel_envs {
                let res = &step_results[i];
                iter_reward_sum += res.reward;

                for item in &res.reward_breakdown {
                    *iter_reward_breakdown
                        .entry(item.name.clone())
                        .or_insert(0.0) += item.value;
                }

                buffer.push(
                    current_obss[i].to_vector(),
                    action_indices[i],
                    log_probs[i],
                    res.reward,
                    values[i],
                    res.terminated,
                );

                if res.terminated || res.truncated {
                    current_obss[i] = par_envs.reset_all()[i].clone();
                    iter_episodes += 1;
                } else {
                    current_obss[i] = res.obs.clone();
                }
            }
        }

        let last_obs = &current_obss[0];
        let last_state_tensor =
            match Tensor::from_vec(last_obs.to_vector(), (1, state_dim), &device) {
                Ok(t) => t,
                Err(_) => break,
            };
        let last_val_scalar = match agent.actor_critic.forward(&last_state_tensor) {
            Ok((_, last_val)) => last_val
                .squeeze(0)
                .unwrap()
                .squeeze(0)
                .unwrap()
                .to_scalar()
                .unwrap_or(0.0),
            Err(_) => 0.0,
        };

        if let Ok(stats) = agent.update(&buffer, last_val_scalar) {
            let total_steps = iter * rollout_steps_per_env * num_parallel_envs;
            let ep_return = iter_reward_sum / iter_episodes.max(1) as f32;

            {
                let mut t = tasks.blocking_lock();
                if let Some(task) = t.get_mut(&task_id) {
                    task.current_step = total_steps;
                    task.ep_return = ep_return;
                }
            }

            // Update DB progress periodically
            if iter % 5 == 0 {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                if let Ok(rt) = rt {
                    let _ =
                        rt.block_on(repo.update_progress(&task_id, total_steps as i64, ep_return));
                }
            }

            let obs_payload = ObsFeaturePayload {
                fiora_hp_pct: if last_obs.fiora_max_hp > 0.0 {
                    last_obs.fiora_hp / last_obs.fiora_max_hp
                } else {
                    1.0
                },
                riven_hp_pct: if last_obs.riven_max_hp > 0.0 {
                    last_obs.riven_hp / last_obs.riven_max_hp
                } else {
                    1.0
                },
                distance: last_obs.distance,
                q_ready: last_obs.q_ready,
                w_ready: last_obs.w_ready,
                e_ready: last_obs.e_ready,
                r_ready: last_obs.r_ready,
                has_vital: last_obs.has_vital,
                vital_is_active: last_obs.vital_is_active,
                vital_direction: if last_obs.vital_dir_x > 0.5 {
                    "+X (东侧)"
                } else if last_obs.vital_dir_neg_x > 0.5 {
                    "-X (西侧)"
                } else if last_obs.vital_dir_z > 0.5 {
                    "+Z (北侧)"
                } else if last_obs.vital_dir_neg_z > 0.5 {
                    "-Z (南侧)"
                } else {
                    "None"
                }
                .into(),
            };

            let elapsed = iter_start.elapsed().as_secs_f64();
            let steps_done = rollout_steps_per_env * num_parallel_envs;
            let fps = if elapsed > 0.0 {
                (steps_done as f64 / elapsed) as usize
            } else {
                0
            };

            let obs_vec = last_obs.to_vector();
            let real_policy = agent
                .actor_critic
                .policy_probs(&last_state_tensor, &obs_vec)
                .map(|probs| {
                    let actions: [&str; 9] = [
                        "MoveEast50 (东侧 50u 站位)",
                        "MoveWest50 (西侧 50u 站位)",
                        "MoveNorth50 (北侧 50u 站位)",
                        "MoveSouth50 (南侧 50u 站位)",
                        "AttackRiven (普通攻击 瑞雯)",
                        "CastQ (Q: 破空斩)",
                        "CastW (W: 劳伦特心眼刀)",
                        "CastE (E: 夺命连刺)",
                        "CastR (R: 无双挑战)",
                    ];
                    probs
                        .iter()
                        .enumerate()
                        .map(|(i, &p)| PolicyItem {
                            action_id: i,
                            action: actions[i].to_string(),
                            prob: p,
                        })
                        .collect()
                })
                .unwrap_or_default();

            let count = iter_episodes.max(1) as f32;
            let real_reward_breakdown: Vec<RewardItem> = iter_reward_breakdown
                .iter()
                .map(|(k, v)| RewardItem {
                    name: k.clone(),
                    value: v / count,
                })
                .collect();

            let out_metrics = OutFrame::Metrics {
                task_id: task_id.clone(),
                step: total_steps,
                ep_return,
                loss: stats.policy_loss + stats.value_loss,
                kl: stats.kl,
                entropy: stats.entropy_loss,
                value: last_val_scalar,
                fps,
                policy: real_policy,
                reward_breakdown: real_reward_breakdown,
                obs_feature: Some(obs_payload),
            };

            let _ = event_tx.send(out_metrics);

            if iter % 5 == 0 || iter == 1 {
                let log_msg = format!(
                    "[{}] Iter {:2}/{} | Avg Reward: {:6.2} | P-Loss: {:7.4} | V-Loss: {:7.4}",
                    task_id, iter, total_iterations, ep_return, stats.policy_loss, stats.value_loss
                );
                let _ = event_tx.send(OutFrame::Log {
                    task_id: task_id.clone(),
                    level: "info".into(),
                    message: log_msg,
                });
            }
        }
    }

    {
        let mut t = tasks.blocking_lock();
        if let Some(task) = t.get_mut(&task_id) {
            task.status = "finished".to_string();
        }
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    if let Ok(rt) = rt {
        let _ = rt.block_on(repo.update_status(&task_id, "finished"));
    }
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
