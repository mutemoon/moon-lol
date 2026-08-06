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
    pub parallel_envs: usize,
    pub max_steps: usize,
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
}

// ── Visual subprocess protocol ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualObsFrame {
    pub step: usize,
    pub obs: ObsFeaturePayload,
    pub reward: f32,
    pub reward_breakdown: Vec<RewardItem>,
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
}
