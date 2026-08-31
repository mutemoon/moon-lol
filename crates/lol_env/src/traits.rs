use std::collections::HashMap;

use bevy::app::App;
use bevy::ecs::world::World;
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec};

/// Controls whether the Env runs headless (for training) or with a window (for visualization).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// No window, MinimalPlugins, maximum throughput.
    #[default]
    Headless,
    /// With window + render/particle plugins (standard Bevy WinitPlugin).
    Window,
    /// With window + render/particle plugins, but **without** WinitPlugin.
    /// Used by `visual_runner` which drives its own custom winit event loop.
    WindowCustomLoop,
}

/// Configuration for constructing an RL environment.
#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub max_steps: usize,
    pub render_mode: RenderMode,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            max_steps: 0,
            render_mode: RenderMode::Headless,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RewardBreakdownItem {
    pub name: String,
    pub value: f32,
}

/// Step result returned from environment step.
#[derive(Debug, Clone)]
pub struct StepResult<O> {
    pub obs: O,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
    pub step: usize,
    pub reward_breakdown: Vec<RewardBreakdownItem>,
    pub reward_variables: HashMap<String, f32>,
}

/// Core Reinforcement Learning Environment Trait.
pub trait RlEnvironment: 'static {
    type Action: Copy + Send + PartialEq + 'static;
    type Obs: Send + Clone + 'static;

    fn env_name() -> &'static str
    where
        Self: Sized;

    fn display_name() -> &'static str
    where
        Self: Sized;

    fn description() -> &'static str
    where
        Self: Sized;

    /// 环境固有的单回合默认最大步数（Horizon）。
    fn default_max_steps() -> usize
    where
        Self: Sized;

    /// 环境自带的推荐训练超参数（唯一真实来源直接从 lol_rl_protocol 获取）。
    fn default_training_params() -> lol_rl_protocol::EnvTrainingParams
    where
        Self: Sized,
    {
        lol_rl_protocol::get_env_training_params(Self::env_name())
    }

    /// 获取当前环境实例配置的最大步数（0 为无限制或由环境自身决定）。
    fn max_steps(&self) -> usize;

    /// 动作空间描述（离散 / 连续 / 混合），训练与可视化循环据此分派采样与更新。
    fn action_space() -> ActionSpace
    where
        Self: Sized,
    {
        ActionSpace::Discrete(0)
    }

    fn action_dim() -> usize
    where
        Self: Sized;

    fn state_dim() -> usize
    where
        Self: Sized,
    {
        Self::obs_schema()
            .map(|s| s.raw_dim())
            .expect("Environment must implement obs_schema or override state_dim")
    }

    fn action_labels() -> &'static [&'static str]
    where
        Self: Sized;

    /// 结构化观测空间规范 AST（供策略网络自动推导 Embedding/EntityMLP 架构与前端结构化动态展示）。
    fn obs_schema() -> Option<ObsSchema>
    where
        Self: Sized,
    {
        None
    }

    fn action_from_index(idx: usize) -> Self::Action
    where
        Self: Sized;

    fn action_to_index(action: Self::Action) -> usize
    where
        Self: Sized;

    fn action_name(action: Self::Action) -> &'static str
    where
        Self: Sized;

    /// 从 PPO 扁平编码动作恢复具体动作。默认实现兼容纯离散（编码即分类索引）。
    fn action_from_encoding(encoded: &[f32]) -> Self::Action
    where
        Self: Sized,
    {
        Self::action_from_index(encoded.first().copied().unwrap_or(0.0) as usize)
    }

    /// 将具体动作编码为 PPO 扁平向量。默认实现兼容纯离散。
    fn action_to_encoding(action: Self::Action) -> Vec<f32>
    where
        Self: Sized,
    {
        vec![Self::action_to_index(action) as f32]
    }

    /// 智能体数量（单智能体为 1，2人自博弈为 2，未来 5v5 为 10）。
    fn num_agents() -> usize
    where
        Self: Sized,
    {
        1
    }

    /// 各智能体角色名称（如 ["Fiora", "Riven"] 或 ["Agent"]）。
    fn agent_names() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &["Agent"]
    }

    /// 构造一个使用默认配置（使用环境自身默认最大步数、Headless 模式）的环境实例。
    fn new() -> Self
    where
        Self: Sized;

    /// 使用指定配置构造环境实例。若 config.max_steps == 0，则采用环境默认步数。
    fn with_config(config: EnvConfig) -> Self
    where
        Self: Sized;

    /// 重置环境，返回所有智能体的初始观测（长度与 num_agents 一致）。
    fn reset(&mut self) -> Vec<Self::Obs>;

    /// 接收所有智能体的动作切片（actions.len() == num_agents()），推演并返回所有智能体的 StepResult。
    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>>;

    /// 单智能体便捷 reset（仅限 num_agents() == 1）。
    fn reset_single(&mut self) -> Self::Obs {
        self.reset().into_iter().next().expect("empty obs")
    }

    /// 单智能体便捷 step（仅限 num_agents() == 1）。
    fn step_single(&mut self, action: Self::Action) -> StepResult<Self::Obs> {
        self.step(&[action])
            .into_iter()
            .next()
            .expect("empty step result")
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32>;

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload>;

    /// 返回动作有效性掩码（true 为有效动作，false 为被掩码屏蔽的非法动作）。
    /// 纯离散空间对应各分类，混合空间对应离散控制头分类。
    fn action_mask(_obs: &Self::Obs) -> Option<Vec<bool>> {
        None
    }

    /// 结构化动作空间规范 AST（供策略网络自动推导多头 Actor 架构）。
    /// 返回 None 时降级为旧的 action_space() + action_mask() 路径。
    fn action_schema() -> Option<lol_rl_protocol::ActionSchema>
    where
        Self: Sized,
    {
        None
    }

    /// 因式分解的动作掩码（每个动作分支独立的有效性过滤）。
    /// 返回 None 时降级为旧的 action_mask() 路径。
    fn action_masks(_obs: &Self::Obs) -> Option<lol_rl_protocol::ActionMasks> {
        None
    }

    /// 关联/静态获取环境奖励公式规范（用于 UI 符号展示、参数解析及遥测指标）。
    fn reward_formula_spec() -> Option<RewardFormulaSpec>
    where
        Self: Sized,
    {
        None
    }

    fn reward_formula(&self) -> Option<RewardFormulaSpec> {
        None
    }

    /// 默认课程学习配置（环境声明自身是否支持课程学习及其默认参数）。
    /// 若返回 Some(config)，在任务未显式指定自定义 curriculum_json 时，训练引擎自动激活该课程。
    fn default_curriculum() -> Option<lol_rl_protocol::CurriculumConfig>
    where
        Self: Sized,
    {
        None
    }

    /// 运行时更新课程学习参数（默认无操作；支持课程学习的环境应覆盖此方法）。
    fn update_curriculum(
        &mut self,
        _hp_scale: f32,
        _cs_reward: f32,
        _attack_no_cs_penalty: f32,
        _harass_coef: f32,
    ) {
    }
}

/// Visual Environment Trait: Extends RlEnvironment to provide hooks for winit window event loop and rendering.
pub trait VisualEnvironment: RlEnvironment {
    fn take_app(&mut self) -> App;
    fn window_title(&self) -> &'static str;
    fn is_assets_loaded(&self, world: &World) -> bool;
    fn on_assets_loaded(&mut self, app: &mut App);
    fn reset_world(&mut self, app: &mut App) -> Vec<Self::Obs>;

    /// 回合起点不变量对齐：每次「实体已重建、即将开始新对局」时统一调用，
    /// 保证第一次构造与每次 reset 的初始状态语义完全一致（headless / visual 共用）。
    ///
    /// 默认实现不做任何对齐（实体生成时即为就绪态）；需要补技能等级、
    /// 冷却复位、固定血量等回合起点状态的环境在此覆盖。
    ///
    /// **单一事实来源**：headless `RlEnvironment::reset`、无头构造路径与
    /// `reset_world` 都应调用此方法，避免各 reset 配方各自手写导致漏项
    /// （例如可视化 reset 重建实体后丢失 FlashCooldown 导致闪现永远无冷却）。
    fn on_episode_ready(&mut self, _world: &mut World) {}

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs>;
    fn get_current_obs(&self, world: &World) -> Self::Obs {
        self.get_current_obs_all(world)
            .into_iter()
            .next()
            .expect("empty obs")
    }
    fn step_world(&mut self, app: &mut App, actions: &[Self::Action])
    -> Vec<StepResult<Self::Obs>>;
}

/// Metadata description of an RL environment for registry & UI listings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvMeta {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub action_dim: usize,
    pub state_dim: usize,
    pub action_labels: Vec<String>,
}

/// 集中注册的环境类型列表宏，任何需要按环境派发的地方均可直接复用该宏
#[macro_export]
macro_rules! for_all_rl_environments {
    ($macro_name:ident) => {
        $macro_name!(
            ($crate::solo_v0::SoloV0Env, lol_rl_protocol::ENV_SOLO_V0),
            ($crate::fiora_v3::FioraV3Env, lol_rl_protocol::ENV_FIORA_V3),
            ($crate::fiora_v2::FioraV2Env, lol_rl_protocol::ENV_FIORA_V2),
            (
                $crate::fiora_v1::FioraVsRivenRealEnv,
                lol_rl_protocol::ENV_FIORA_V1
            ),
            (
                $crate::fiora_v0::FioraVsRivenEnv,
                lol_rl_protocol::ENV_FIORA_V0
            )
        );
    };
}

pub fn list_available_envs() -> Vec<EnvMeta> {
    vec![
        crate::solo_v0::SoloV0Env::meta(),
        crate::fiora_v3::FioraV3Env::meta(),
        crate::fiora_v2::FioraV2Env::meta(),
        crate::fiora_v1::FioraVsRivenRealEnv::meta(),
        crate::fiora_v0::FioraVsRivenEnv::meta(),
    ]
}

pub fn get_env_meta(name: &str) -> Option<EnvMeta> {
    list_available_envs().into_iter().find(|e| e.name == name)
}
