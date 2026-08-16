use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const DEFAULT_RL_SERVER_ADDR: &str = "127.0.0.1:8765";

/// 强化学习动作空间描述，供训练/可视化循环区分离散与连续策略。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionSpace {
    /// 纯离散分类，动作数 n（legacy 环境）。
    Discrete(usize),
    /// 纯连续高斯，维度 d。
    Continuous(usize),
    /// 混合：连续高斯 d 维 + 一个离散分类 k 类。
    Hybrid {
        continuous_dims: usize,
        discrete_classes: usize,
    },
}

impl ActionSpace {
    /// Actor 头输出维度：Discrete(n)=n，Continuous(d)=d，Hybrid=d+k。
    pub fn actor_head_dim(&self) -> usize {
        match self {
            Self::Discrete(n) => *n,
            Self::Continuous(d) => *d,
            Self::Hybrid {
                continuous_dims,
                discrete_classes,
            } => continuous_dims + discrete_classes,
        }
    }

    /// Rollout 缓冲区中单个动作的扁平编码长度：
    /// Discrete=1（分类索引），Continuous=d，Hybrid=d+1（末位为攻击分类索引）。
    pub fn encoding_dim(&self) -> usize {
        match self {
            Self::Discrete(_) => 1,
            Self::Continuous(d) => *d,
            Self::Hybrid {
                continuous_dims, ..
            } => continuous_dims + 1,
        }
    }
}

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
    Exp(Box<RewardExpr>),
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
            Self::Exp(a) => a.eval(vars).exp(),
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
            Self::Exp(a) => format!("exp({})", a.to_display_string()),
        }
    }

    /// 转换为 LaTeX 符号公式，如 `0.02 \cdot \mathbb{1}_{\text{newly aligned}}`
    pub fn to_latex(&self) -> String {
        self.to_latex_inner(None)
    }

    /// 转换为代入真实变量值后的 LaTeX 公式，变量叶节点替换为其数值
    pub fn to_latex_substituted(&self, vars: &HashMap<String, f32>) -> String {
        self.to_latex_inner(Some(vars))
    }

    fn to_latex_inner(&self, vars: Option<&HashMap<String, f32>>) -> String {
        match self {
            Self::Constant(c) => fmt_math_num(*c),
            Self::Variable(name) => match vars {
                Some(vars) => fmt_math_num(vars.get(name).copied().unwrap_or(0.0)),
                None => format!(r"\mathbb{{1}}_{{\text{{{}}}}}", latex_var_text(name)),
            },
            Self::Add(a, b) => format!("{} + {}", a.to_latex_inner(vars), b.to_latex_inner(vars)),
            Self::Sub(a, b) => format!("{} - {}", a.to_latex_inner(vars), b.to_latex_inner(vars)),
            Self::Mul(a, b) => format!(
                r"{} \cdot {}",
                a.to_latex_inner(vars),
                b.to_latex_inner(vars)
            ),
            Self::IfElse {
                cond,
                then_branch,
                else_branch,
            } => format!(
                r"\begin{{cases}} {} & \text{{if }} {} \\ {} & \text{{otherwise}} \end{{cases}}",
                then_branch.to_latex_inner(vars),
                cond.to_latex_inner(vars),
                else_branch.to_latex_inner(vars)
            ),
            Self::Gt(a, b) => format!("{} > {}", a.to_latex_inner(vars), b.to_latex_inner(vars)),
            Self::Max(a, b) => {
                format!(
                    r"\max({}, {})",
                    a.to_latex_inner(vars),
                    b.to_latex_inner(vars)
                )
            }
            Self::Min(a, b) => {
                format!(
                    r"\min({}, {})",
                    a.to_latex_inner(vars),
                    b.to_latex_inner(vars)
                )
            }
            Self::Exp(a) => {
                format!(r"\exp\left({}\right)", a.to_latex_inner(vars))
            }
        }
    }
}

/// 把数值格式化成干净的 LaTeX 数字（`2.0`→`2`、`-0.002`→`-0.002`）。
fn fmt_math_num(v: f32) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// 变量名转可读的指示函数下标：`is_newly_aligned` → `newly aligned`。
fn latex_var_text(name: &str) -> String {
    let stripped = name.strip_prefix("is_").unwrap_or(name);
    stripped.replace('_', " ")
}

/// 用 ` + `/` - ` 拼接各项，避免出现 `+ -0.02` 这类连号。
fn join_with_signs(parts: &[String]) -> String {
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            out.push_str(part);
        } else if let Some(rest) = part.strip_prefix('-') {
            out.push_str(" - ");
            out.push_str(rest);
        } else {
            out.push_str(" + ");
            out.push_str(part);
        }
    }
    out
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

    /// 符号版总公式：`R = t_1 + t_2 + ...`
    pub fn to_latex(&self) -> String {
        let parts: Vec<String> = self.terms.iter().map(|t| t.expr.to_latex()).collect();
        format!("R = {}", join_with_signs(&parts))
    }

    /// 代入真实变量值后的总公式：`R = t_1 + t_2 + ... = total`
    pub fn to_latex_substituted(&self, vars: &HashMap<String, f32>) -> String {
        let parts: Vec<String> = self
            .terms
            .iter()
            .map(|t| t.expr.to_latex_substituted(vars))
            .collect();
        let total = self.compute(vars).0;
        format!("R = {} = {}", join_with_signs(&parts), fmt_math_num(total))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricsRow {
    pub step: usize,
    pub ep_return: f32,
    /// 兼容旧数据：policy_loss + value_loss 之和
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
}

pub const ENV_FIORA_V2: &str = "FioraV2";
pub const ENV_FIORA_V1: &str = "FioraV1";
pub const ENV_FIORA_V0: &str = "FioraV0";

/// 环境自带的训练超参数规范（作为环境默认超参数的唯一真实来源）。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EnvTrainingParams {
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
}

/// 环境规范与展示元数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub default_params: EnvTrainingParams,
}

pub const ENV_FIORA_V2_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V2,
    label: "全技能实战 (V2)",
    tag: "V2",
    description: "无缝融合 Q/W/E/R 全技能连招与真实移动走位，高维状态空间与复杂动作分支",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 8,
        hidden_dim: 256,
        rollout_steps_per_env: 160,
        total_iterations: 300,
    },
};

pub const ENV_FIORA_V1_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V1,
    label: "真实移动 (V1)",
    tag: "V1",
    description: "模拟真实微操移动与普攻破绽打击，连续空间离散化动作",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 4,
        hidden_dim: 64,
        rollout_steps_per_env: 80,
        total_iterations: 80,
    },
};

pub const ENV_FIORA_V0_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V0,
    label: "瞬移站位 (V0)",
    tag: "V0",
    description: "简化版瞬移站位打弱点机制，快速收敛验证基础 PPO 策略",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 4,
        hidden_dim: 64,
        rollout_steps_per_env: 80,
        total_iterations: 50,
    },
};

pub const AVAILABLE_ENVS: &[EnvSpec] = &[ENV_FIORA_V2_SPEC, ENV_FIORA_V1_SPEC, ENV_FIORA_V0_SPEC];

pub fn get_env_spec(name: &str) -> Option<&'static EnvSpec> {
    AVAILABLE_ENVS.iter().find(|e| e.name == name)
}

pub fn get_env_training_params(name: &str) -> EnvTrainingParams {
    get_env_spec(name)
        .map(|s| s.default_params.clone())
        .unwrap_or(ENV_FIORA_V2_SPEC.default_params)
}

impl TaskConfigPayload {
    pub fn default_for_env(env_name: &str) -> Self {
        let params = get_env_training_params(env_name);
        Self {
            name: "RL 对战训练任务".to_string(),
            agent_type: "PPO".to_string(),
            env_name: env_name.to_string(),
            lr: params.lr,
            gamma: params.gamma,
            gae_lambda: params.gae_lambda,
            clip_eps: params.clip_eps,
            ppo_epochs: params.ppo_epochs,
            hidden_dim: params.hidden_dim,
            parallel_envs: 0,
            rollout_steps_per_env: params.rollout_steps_per_env,
            total_iterations: params.total_iterations,
        }
    }
}

impl Default for TaskConfigPayload {
    fn default() -> Self {
        Self::default_for_env(ENV_FIORA_V2)
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
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub lr: f32,
    pub total_iterations: usize,
    pub rollout_steps_per_env: usize,
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
    /// Mask 后的有效采样概率
    pub prob: f32,
    /// 未 Mask 的网络原始 Softmax 概率
    #[serde(default)]
    pub raw_prob: f32,
    /// 当前步是否被 Action Mask 屏蔽
    #[serde(default)]
    pub is_masked: bool,
}

/// 可视化策略展示：按真实动作空间区分（连续/混合环境不再用 preset 伪分布）。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PolicyDisplay {
    /// 离散动作空间：每个离散动作的概率分布。
    Discrete(Vec<PolicyItem>),
    /// 混合动作空间（连续移动 + 离散攻击）：连续均值 + 攻击概率。
    Hybrid {
        move_x: f32,
        move_z: f32,
        attack_prob: f32,
        #[serde(default)]
        raw_attack_prob: f32,
        #[serde(default)]
        is_attack_masked: bool,
    },
    /// 增强混合动作空间（多维连续偏移均值 + 多离散动作概率分布）
    HybridMulti {
        continuous_means: Vec<f32>,
        discrete_probs: Vec<PolicyItem>,
    },
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
    /// 本局从开始累积到当前步的总奖励（每次对局重置后归零）。
    #[serde(default)]
    pub episode_reward: f32,
    pub reward_breakdown: Vec<RewardItem>,
    pub policy: PolicyDisplay,
    pub terminated: bool,
    pub truncated: bool,
    #[serde(default)]
    pub self_alive: bool,
    #[serde(default)]
    pub target_alive: bool,
    #[serde(default)]
    pub fiora_alive: bool,
    #[serde(default)]
    pub riven_alive: bool,
    pub reward_formula: Option<RewardFormulaSpec>,
    pub reward_variables: Option<HashMap<String, f32>>,
    /// 策略真实输入的观测向量（与 `obs_labels` 一一对应）。
    #[serde(default)]
    pub obs_vector: Vec<f32>,
    /// 观测向量每一维的简要说明。
    #[serde(default)]
    pub obs_labels: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualOutFrame {
    Ready {
        checkpoint_path: String,
        env_name: String,
        env_max_steps: usize,
        action_labels: Vec<String>,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_reward_expr_eval() {
        let vars = HashMap::from([("a".to_string(), 2.0), ("b".to_string(), 3.0)]);
        let expr = RewardExpr::Add(
            Box::new(RewardExpr::Variable("a".into())),
            Box::new(RewardExpr::Mul(
                Box::new(RewardExpr::Constant(4.0)),
                Box::new(RewardExpr::Variable("b".into())),
            )),
        );
        assert_eq!(expr.eval(&vars), 2.0 + 4.0 * 3.0);
    }

    #[test]
    fn test_reward_expr_gt_and_if_else() {
        let vars = HashMap::from([("x".to_string(), 5.0)]);
        let gt = RewardExpr::Gt(
            Box::new(RewardExpr::Variable("x".into())),
            Box::new(RewardExpr::Constant(3.0)),
        );
        assert_eq!(gt.eval(&vars), 1.0);
        let if_else = RewardExpr::IfElse {
            cond: Box::new(gt),
            then_branch: Box::new(RewardExpr::Constant(10.0)),
            else_branch: Box::new(RewardExpr::Constant(0.0)),
        };
        assert_eq!(if_else.eval(&vars), 10.0);
    }

    #[test]
    fn test_reward_expr_exp() {
        let vars = HashMap::from([("t".to_string(), 4.0)]);
        let expr = RewardExpr::Sub(
            Box::new(RewardExpr::Mul(
                Box::new(RewardExpr::Constant(3.0)),
                Box::new(RewardExpr::Exp(Box::new(RewardExpr::Mul(
                    Box::new(RewardExpr::Constant(0.6)),
                    Box::new(RewardExpr::Sub(
                        Box::new(RewardExpr::Constant(4.0)),
                        Box::new(RewardExpr::Variable("t".into())),
                    )),
                )))),
            )),
            Box::new(RewardExpr::Constant(3.0)),
        );
        // t = 4.0 => 3.0 * (exp(0) - 1) = 0.0
        assert!((expr.eval(&vars) - 0.0).abs() < 1e-5);

        let vars_1s = HashMap::from([("t".to_string(), 1.0)]);
        // t = 1.0 => 3.0 * (exp(1.8) - 1) = 3.0 * (6.0496 - 1) = 15.1489
        let val_1s = expr.eval(&vars_1s);
        assert!((val_1s - 15.1489).abs() < 1e-2);
    }

    #[test]
    fn test_reward_formula_compute() {
        let spec = RewardFormulaSpec {
            name: "test".into(),
            terms: vec![
                RewardTermSpec::new("c", "常数", RewardExpr::Constant(-0.5)),
                RewardTermSpec::new("v", "变量", RewardExpr::Variable("hit".into())),
            ],
        };
        let vars = HashMap::from([("hit".to_string(), 0.8)]);
        let (total, items) = spec.compute(&vars);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].value, -0.5);
        assert_eq!(items[1].value, 0.8);
        assert!((total - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_action_space_dims() {
        assert_eq!(ActionSpace::Discrete(5).actor_head_dim(), 5);
        assert_eq!(ActionSpace::Discrete(5).encoding_dim(), 1);
        assert_eq!(ActionSpace::Continuous(3).actor_head_dim(), 3);
        assert_eq!(ActionSpace::Continuous(3).encoding_dim(), 3);
        let hybrid = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        };
        assert_eq!(hybrid.actor_head_dim(), 4);
        assert_eq!(hybrid.encoding_dim(), 3);
    }
}
