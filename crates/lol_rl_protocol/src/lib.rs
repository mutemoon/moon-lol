use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const DEFAULT_RL_SERVER_ADDR: &str = "127.0.0.1:8765";

/// 结构化奖励表达式 AST
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RewardExpr {
    Constant(f32),
    Variable(String),
    Add(Box<RewardExpr>, Box<RewardExpr>),
    Sub(Box<RewardExpr>, Box<RewardExpr>),
    Mul(Box<RewardExpr>, Box<RewardExpr>),
    IfElse {
        cond: Box<RewardExpr>,
        then_branch: Box<RewardExpr>,
        else_branch: Box<RewardExpr>,
    },
    Gt(Box<RewardExpr>, Box<RewardExpr>),
    Max(Box<RewardExpr>, Box<RewardExpr>),
    Min(Box<RewardExpr>, Box<RewardExpr>),
}

impl RewardExpr {
    /// 在给定的环境变量上下文中对表达式求值
    pub fn eval(&self, vars: &HashMap<String, f32>) -> f32 {
        match self {
            Self::Constant(c) => *c,
            Self::Variable(name) => vars.get(name).copied().unwrap_or(0.0),
            Self::Add(a, b) => a.eval(vars) + b.eval(vars),
            Self::Sub(a, b) => a.eval(vars) - b.eval(vars),
            Self::Mul(a, b) => a.eval(vars) * b.eval(vars),
            Self::IfElse {
                cond,
                then_branch,
                else_branch,
            } => {
                if cond.eval(vars) > 0.0 {
                    then_branch.eval(vars)
                } else {
                    else_branch.eval(vars)
                }
            }
            Self::Gt(a, b) => {
                if a.eval(vars) > b.eval(vars) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Max(a, b) => a.eval(vars).max(b.eval(vars)),
            Self::Min(a, b) => a.eval(vars).min(b.eval(vars)),
        }
    }

    /// 转换为数学展示字符串，如 "80.0 × is_vital_break"
    pub fn to_display_string(&self) -> String {
        match self {
            Self::Constant(c) => {
                if c.fract() == 0.0 {
                    format!("{:.0}", c)
                } else {
                    format!("{:.2}", c)
                }
            }
            Self::Variable(v) => v.clone(),
            Self::Add(a, b) => format!("({} + {})", a.to_display_string(), b.to_display_string()),
            Self::Sub(a, b) => format!("({} - {})", a.to_display_string(), b.to_display_string()),
            Self::Mul(a, b) => format!("{} × {}", a.to_display_string(), b.to_display_string()),
            Self::IfElse {
                cond,
                then_branch,
                else_branch,
            } => {
                format!(
                    "if {} then {} else {}",
                    cond.to_display_string(),
                    then_branch.to_display_string(),
                    else_branch.to_display_string()
                )
            }
            Self::Gt(a, b) => format!("({} > {})", a.to_display_string(), b.to_display_string()),
            Self::Max(a, b) => format!("max({}, {})", a.to_display_string(), b.to_display_string()),
            Self::Min(a, b) => format!("min({}, {})", a.to_display_string(), b.to_display_string()),
        }
    }
}

/// 单项奖励定义
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RewardTermSpec {
    pub id: String,
    pub label: String,
    pub expr: RewardExpr,
}

impl RewardTermSpec {
    pub fn new(id: impl Into<String>, label: impl Into<String>, expr: RewardExpr) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            expr,
        }
    }

    pub fn eval(&self, vars: &HashMap<String, f32>) -> f32 {
        self.expr.eval(vars)
    }
}

/// 统一的环境奖励公式规范
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RewardFormulaSpec {
    pub name: String,
    pub terms: Vec<RewardTermSpec>,
}

impl RewardFormulaSpec {
    /// 依据结构化表达式计算总奖励与分解项
    pub fn compute(&self, vars: &HashMap<String, f32>) -> (f32, Vec<RewardItem>) {
        let mut total = 0.0;
        let mut items = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            let val = term.eval(vars);
            total += val;
            items.push(RewardItem {
                name: term.label.clone(),
                value: val,
            });
        }
        (total, items)
    }
}

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
        reward_formula: Option<RewardFormulaSpec>,
        reward_variables: Option<HashMap<String, f32>>,
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
    pub reward_formula: Option<RewardFormulaSpec>,
    pub reward_variables: Option<HashMap<String, f32>>,
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
