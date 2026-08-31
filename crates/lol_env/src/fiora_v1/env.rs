use bevy::prelude::*;
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec};

use super::action::{FIORA_V1_ACTION_SCHEMA, FioraVsRivenRealAction};
use super::step::{dispatch_action_world, step_world};
use crate::base_env::{LolBaseEnv, fiora_champion_spec, riven_champion_spec};
use crate::fiora_riven_common::{FioraVsRivenObs, get_obs_from_world};
use crate::traits::{EnvConfig, EnvMeta, RenderMode, RlEnvironment, StepResult, VisualEnvironment};

pub type FioraVsRivenRealObs = FioraVsRivenObs;

pub struct FioraVsRivenRealEnv {
    pub base: LolBaseEnv,
}

impl std::ops::Deref for FioraVsRivenRealEnv {
    type Target = LolBaseEnv;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FioraVsRivenRealEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl FioraVsRivenRealEnv {
    pub const DEFAULT_MAX_STEPS: usize = 40;

    pub fn new() -> Self {
        Self::with_config(EnvConfig::default())
    }

    pub fn new_with_max_steps(max_steps: usize) -> Self {
        Self::with_config(EnvConfig {
            max_steps,
            render_mode: RenderMode::Headless,
        })
    }

    pub fn with_config(config: EnvConfig) -> Self {
        let base = LolBaseEnv::builder(config, Self::DEFAULT_MAX_STEPS)
            .window_title("Fiora vs Riven (Continuous) - RL Visual Viewer")
            .add_champion(fiora_champion_spec(
                lol_core::team::Team::Order,
                Vec3::ZERO,
                [3, 1, 1, 1],
                true,
            ))
            .add_champion(riven_champion_spec(
                lol_core::team::Team::Chaos,
                Vec3::new(50.0, 0.0, 0.0),
                [3, 1, 1, 1],
                false,
            ))
            .build();
        Self { base }
    }

    pub fn meta() -> EnvMeta {
        EnvMeta {
            name: <Self as RlEnvironment>::env_name().to_string(),
            display_name: <Self as RlEnvironment>::display_name().to_string(),
            description: <Self as RlEnvironment>::description().to_string(),
            action_dim: <Self as RlEnvironment>::action_dim(),
            state_dim: <Self as RlEnvironment>::state_dim(),
            action_labels: <Self as RlEnvironment>::action_labels()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn render_mode(&self) -> RenderMode {
        self.base.render_mode()
    }

    pub fn app(&self) -> &App {
        self.base.app()
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.base.app_mut()
    }

    pub fn fiora(&self) -> Entity {
        self.base.fiora()
    }

    pub fn riven(&self) -> Entity {
        self.base.riven()
    }

    pub fn max_steps(&self) -> usize {
        self.base.max_steps()
    }

    pub fn step_count(&self) -> usize {
        self.base.step_count()
    }

    pub fn reset(&mut self) -> FioraVsRivenRealObs {
        self.base.reset_base();
        self.get_obs()
    }

    pub fn get_obs(&self) -> FioraVsRivenRealObs {
        get_obs_from_world(self.base.world(), self.base.fiora(), self.base.riven())
    }

    pub fn dispatch_action(&mut self, action: FioraVsRivenRealAction) {
        let fiora = self.base.fiora();
        let riven = self.base.riven();
        dispatch_action_world(self.base.world_mut(), fiora, riven, action);
    }

    pub fn step(&mut self, action: FioraVsRivenRealAction) -> StepResult<FioraVsRivenRealObs> {
        self.base.increment_step();
        let fiora = self.base.fiora();
        let riven = self.base.riven();
        step_world(
            &mut self.base.app,
            fiora,
            riven,
            action,
            self.base.step_count,
            self.base.max_steps,
        )
    }
}

impl RlEnvironment for FioraVsRivenRealEnv {
    type Action = FioraVsRivenRealAction;
    type Obs = FioraVsRivenRealObs;

    fn env_name() -> &'static str {
        "FioraV1"
    }

    fn display_name() -> &'static str {
        "剑姬 vs 瑞雯 (真实物理移动-6动作)"
    }

    fn description() -> &'static str {
        "剑姬 vs 瑞雯 对战强化学习环境（V1：真实物理移动 + 普攻，混合动作空间）"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        }
    }

    fn action_dim() -> usize {
        Self::action_space().actor_head_dim()
    }

    fn state_dim() -> usize {
        FioraVsRivenRealObs::dim()
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "东移 50u",
            "西移 50u",
            "北移 50u",
            "南移 50u",
            "追击瑞雯",
            "攻击瑞雯",
        ]
    }

    fn obs_schema() -> Option<ObsSchema> {
        Some(super::FIORA_V1_OBS_SCHEMA.clone())
    }

    fn action_schema() -> Option<lol_rl_protocol::ActionSchema> {
        Some(FIORA_V1_ACTION_SCHEMA.clone())
    }

    fn action_from_index(idx: usize) -> Self::Action {
        FioraVsRivenRealAction::preset_from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.preset_index()
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        FioraVsRivenRealAction::from_encoding(encoded)
    }

    fn action_to_encoding(action: Self::Action) -> Vec<f32> {
        action.to_encoding()
    }

    fn action_name(action: Self::Action) -> &'static str {
        action.desc()
    }

    fn default_max_steps() -> usize {
        Self::DEFAULT_MAX_STEPS
    }

    fn max_steps(&self) -> usize {
        self.base.max_steps()
    }

    fn new() -> Self {
        Self::new()
    }

    fn with_config(config: EnvConfig) -> Self {
        Self::with_config(config)
    }

    fn reset(&mut self) -> Vec<Self::Obs> {
        vec![self.reset()]
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let action = actions
            .first()
            .copied()
            .unwrap_or(FioraVsRivenRealAction::new(0.0, 0.0, true));
        vec![self.step(action)]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        Some(FIORA_V1_ACTION_SCHEMA.eval_flat_mask(&obs.to_context()))
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        super::FIORA_V1_SPEC.reward_formula.clone()
    }
}

impl VisualEnvironment for FioraVsRivenRealEnv {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.base.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora vs Riven (Continuous) - RL Visual Viewer"
    }

    fn is_assets_loaded(&self, world: &World) -> bool {
        self.base.is_assets_loaded(world)
    }

    fn on_assets_loaded(&mut self, app: &mut App) {
        self.base.on_assets_ready(app);
    }

    fn reset_world(&mut self, app: &mut App) -> Vec<Self::Obs> {
        let champions = self.base.reset_app(app);
        let (new_fiora, new_riven) = (champions[0], champions[1]);
        vec![get_obs_from_world(app.world(), new_fiora, new_riven)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_obs_from_world(world, self.base.fiora(), self.base.riven())]
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        self.base.increment_step();
        let action = actions
            .first()
            .copied()
            .unwrap_or(FioraVsRivenRealAction::new(0.0, 0.0, true));
        vec![step_world(
            app,
            self.base.fiora(),
            self.base.riven(),
            action,
            self.base.step_count,
            self.base.max_steps,
        )]
    }
}
