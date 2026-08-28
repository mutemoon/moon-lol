use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::reward::{RewardFormulaSpec, RewardItem};
use crate::task::{TaskConfigPayload, TaskOverviewItem};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricsRow {
    pub step: usize,
    pub ep_return: f32,
    /// 训练总损失（等同于 total_loss）
    pub loss: f32,
    pub policy_loss: f32,
    pub value_loss: f32,
    pub total_loss: f32,
    pub kl: f32,
    pub entropy: f32,
    /// 本迭代被 clip 的比例（相对 clip_eps 界）
    pub clip_frac: f32,
    /// 迭代内各状态 critic 预测值的均值
    pub value: f32,
    pub fps: usize,
    /// 本迭代完成的各局步数最大值 / 最小值 / 平均值
    pub ep_steps_max: usize,
    pub ep_steps_min: usize,
    pub ep_steps_avg: f32,
    /// 本迭代各奖励项的每步平均贡献（时间惩罚/对齐/错位/空挥/破绽/击杀）
    pub reward_breakdown: Vec<RewardItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ObsFeaturePayload {
    // ── 通用战斗与环境遥测指标 ──
    #[serde(default)]
    pub self_hp_pct: f32,
    #[serde(default)]
    pub target_hp_pct: f32,
    #[serde(default)]
    pub distance: f32,
    #[serde(default)]
    pub metrics: HashMap<String, f32>,
    #[serde(default)]
    pub tags: HashMap<String, String>,

    // ── 向下兼容字段（供特定环境或现有 UI 无缝使用） ──
    #[serde(default)]
    pub fiora_hp_pct: f32,
    #[serde(default)]
    pub riven_hp_pct: f32,
    #[serde(default)]
    pub q_ready: bool,
    #[serde(default)]
    pub w_ready: bool,
    #[serde(default)]
    pub e_ready: bool,
    #[serde(default)]
    pub r_ready: bool,
    #[serde(default)]
    pub has_vital: bool,
    #[serde(default)]
    pub vital_is_active: bool,
    #[serde(default)]
    pub vital_direction: String,
    #[serde(default)]
    pub vital_active_time: f32,
    #[serde(default)]
    pub has_r_vital: bool,
    #[serde(default)]
    pub r_is_active: bool,
    #[serde(default)]
    pub attack_state: String,
    #[serde(default)]
    pub attack_timer: f32,
}

/// 通用的课程学习实时遥测状态（零业务侵入）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CurriculumTelemetry {
    /// 当前阶段序号（0-based，如 0 表示第一阶段）
    pub phase_index: usize,
    /// 课程总阶段数（如 2）
    pub total_phases: usize,
    /// 当前阶段显示名称（如 "第一阶段: 基础补刀练习"）
    pub phase_name: String,
    /// 所有阶段的名称清单（用于渲染步骤流水线）
    pub all_phase_names: Vec<String>,
    /// 当前阶段内部进度 (0.0 ~ 1.0)
    pub progress: f32,
    /// 晋级下一阶段的判定说明文本（如 "需滑动平均 CS ≥ 2.0 (当前: 1.45)"）
    #[serde(default)]
    pub transition_condition: Option<String>,
    /// 当前生效的动态超参数列表（键值对，用于通用动态参数卡片渲染）
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
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
        policy_loss: f32,
        value_loss: f32,
        total_loss: f32,
        kl: f32,
        entropy: f32,
        clip_frac: f32,
        /// 本任务固定的 PPO clip 界，前端用于 KL 参考线
        clip_eps: f32,
        value: f32,
        fps: usize,
        ep_steps_max: usize,
        ep_steps_min: usize,
        ep_steps_avg: f32,
        reward_breakdown: Vec<RewardItem>,
        obs_feature: Option<ObsFeaturePayload>,
        reward_formula: Option<RewardFormulaSpec>,
        reward_variables: Option<HashMap<String, f32>>,
        #[serde(default)]
        curriculum: Option<CurriculumTelemetry>,
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
