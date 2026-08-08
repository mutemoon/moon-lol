use serde::{Deserialize, Serialize};

pub const DEFAULT_RL_SERVER_ADDR: &str = "127.0.0.1:8765";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricsRow {
    pub step: usize,
    pub ep_return: f32,
    pub loss: f32,
    pub kl: f32,
    pub entropy: f32,
    pub value: f32,
    pub fps: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObsFeaturePayload {
    pub fiora_hp_pct: f32,
    pub riven_hp_pct: f32,
    pub distance: f32,
    pub q_ready: bool,
    pub w_ready: bool,
    pub e_ready: bool,
    pub r_ready: bool,
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_direction: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskConfigPayload {
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
    pub max_steps: usize,
}

impl Default for TaskConfigPayload {
    fn default() -> Self {
        Self {
            name: "RL 对战训练任务".to_string(),
            agent_type: "PPO (Candle)".to_string(),
            env_name: "FioraVsRivenEnv-v0".to_string(),
            lr: 5e-4,
            gamma: 0.99,
            gae_lambda: 0.95,
            clip_eps: 0.2,
            ppo_epochs: 4,
            hidden_dim: 64,
            parallel_envs: 4,
            rollout_steps_per_env: 80,
            total_iterations: 80,
            max_steps: 25600,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskOverviewItem {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub status: String,
    pub current_step: usize,
    pub ep_return: f32,
    pub checkpoints_count: usize,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutFrame {
    TaskList {
        tasks: Vec<TaskOverviewItem>,
    },
    Status {
        task_id: String,
        status: String,
    },
    Metrics {
        task_id: String,
        step: usize,
        ep_return: f32,
        loss: f32,
        kl: f32,
        entropy: f32,
        value: f32,
        fps: usize,
        policy: Vec<PolicyItem>,
        reward_breakdown: Vec<RewardItem>,
        obs_feature: Option<ObsFeaturePayload>,
    },
    Log {
        task_id: String,
        level: String,
        message: String,
    },
    CheckpointMsg {
        task_id: String,
        checkpoint: CheckpointItem,
    },
    CheckpointLoaded {
        task_id: String,
        checkpoint: CheckpointItem,
    },
    TaskDetail {
        task_id: String,
        checkpoints: Vec<CheckpointItem>,
        metrics_history: Vec<MetricsRow>,
        logs: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicyItem {
    pub action_id: usize,
    pub action: String,
    pub prob: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RewardItem {
    pub name: String,
    pub value: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointItem {
    pub id: String,
    pub step: usize,
    pub path: String,
    pub ep_return: f32,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InFrame {
    GetTaskList,
    GetTaskDetail {
        task_id: String,
    },
    CreateTask {
        config: TaskConfigPayload,
    },
    Control {
        task_id: String,
        command: String,
        config_json: Option<String>,
    },
    SaveCheckpoint {
        task_id: String,
    },
    ApplyCheckpoint {
        task_id: String,
        id: String,
    },
    DeleteTask {
        task_id: String,
    },
}

// ── Visual subprocess protocol ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualObsFrame {
    pub step: usize,
    pub obs: ObsFeaturePayload,
    pub reward: f32,
    pub reward_breakdown: Vec<RewardItem>,
    pub policy: Vec<PolicyItem>,
    pub terminated: bool,
    pub truncated: bool,
    pub fiora_alive: bool,
    pub riven_alive: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualOutFrame {
    Ready {
        checkpoint_path: String,
        env_max_steps: usize,
    },
    Frame(VisualObsFrame),
    Log {
        level: String,
        message: String,
    },
    Exited {
        code: Option<i32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualInFrame {
    Reset,
    Pause,
    Resume,
    StepOnce,
    StepWithAction { action_id: usize },
}
