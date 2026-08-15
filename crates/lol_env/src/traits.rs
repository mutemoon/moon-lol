use std::collections::HashMap;

use bevy::app::App;
use bevy::ecs::world::World;
use bevy::math::Vec2;
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, RewardFormulaSpec};

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
        Self: Sized;

    fn action_labels() -> &'static [&'static str]
    where
        Self: Sized;

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

    fn new(max_steps: usize) -> Self
    where
        Self: Sized;

    fn with_config(config: EnvConfig) -> Self
    where
        Self: Sized;

    fn reset(&mut self) -> Self::Obs;

    fn step(&mut self, action: Self::Action) -> StepResult<Self::Obs>;

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32>;

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload>;

    fn is_action_masked(obs: &Self::Obs, action_idx: usize) -> bool;

    /// 返回动作有效性掩码（true 为有效动作，false 为被掩码屏蔽的非法动作）。
    /// 纯离散空间对应各分类，混合空间对应离散控制头分类。
    fn action_mask(_obs: &Self::Obs) -> Option<Vec<bool>> {
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
}

/// Visual Environment Trait: Extends RlEnvironment to provide hooks for winit window event loop and rendering.
pub trait VisualEnvironment: RlEnvironment {
    fn take_app(&mut self) -> App;
    fn window_title(&self) -> &'static str;
    fn is_assets_loaded(&self, world: &World) -> bool;
    fn on_assets_loaded(&mut self, world: &mut World);
    fn reset_world(&mut self, world: &mut World);
    fn get_current_obs(&self, world: &World) -> Self::Obs;
    /// 将可视化窗口的鼠标点击（逻辑视口坐标）翻译为一步动作；默认无点击控制。
    fn action_from_screen_click(
        &mut self,
        _world: &mut World,
        _screen_pos: Vec2,
    ) -> Option<Self::Action> {
        None
    }
    fn step_world(
        &mut self,
        app: &mut App,
        action: Self::Action,
        step_count: usize,
        max_steps: usize,
    ) -> StepResult<Self::Obs>;
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
            ($crate::fiora_v2::FioraV2Env, lol_rl_protocol::ENV_FIORA_V2),
            ($crate::fiora_v1::FioraVsRivenRealEnv, lol_rl_protocol::ENV_FIORA_V1),
            ($crate::fiora_v0::FioraVsRivenEnv, lol_rl_protocol::ENV_FIORA_V0)
        );
    };
}

pub fn list_available_envs() -> Vec<EnvMeta> {
    vec![
        crate::fiora_v2::FioraV2Env::meta(),
        crate::fiora_v1::FioraVsRivenRealEnv::meta(),
        crate::fiora_v0::FioraVsRivenEnv::meta(),
    ]
}

pub fn get_env_meta(name: &str) -> Option<EnvMeta> {
    list_available_envs().into_iter().find(|e| e.name == name)
}
