use bevy::prelude::*;
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec};

use super::action::{FIORA_V0_ACTION_SCHEMA, FioraVsRivenAction};
use super::step::{dispatch_action_world, step_world};
use crate::base_env::{LolBaseEnv, fiora_champion_spec, riven_champion_spec};
use crate::fiora_riven_common::{FioraVsRivenObs, get_obs_from_world};
use crate::traits::{EnvConfig, EnvMeta, RenderMode, RlEnvironment, StepResult, VisualEnvironment};

pub struct FioraVsRivenEnv {
    pub base: LolBaseEnv,
}

impl std::ops::Deref for FioraVsRivenEnv {
    type Target = LolBaseEnv;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FioraVsRivenEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl FioraVsRivenEnv {
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
            .window_title("Fiora vs Riven (Teleport) - RL Visual Viewer")
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

    pub fn reset(&mut self) -> FioraVsRivenObs {
        self.base.reset_base();
        self.get_obs()
    }

    pub fn get_obs(&self) -> FioraVsRivenObs {
        get_obs_from_world(self.base.world(), self.base.fiora(), self.base.riven())
    }

    pub fn dispatch_action(&mut self, action: FioraVsRivenAction) {
        let fiora = self.base.fiora();
        let riven = self.base.riven();
        dispatch_action_world(self.base.world_mut(), fiora, riven, action);
    }

    pub fn step(&mut self, action: FioraVsRivenAction) -> StepResult<FioraVsRivenObs> {
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

impl RlEnvironment for FioraVsRivenEnv {
    type Action = FioraVsRivenAction;
    type Obs = FioraVsRivenObs;

    fn env_name() -> &'static str {
        "FioraV0"
    }

    fn display_name() -> &'static str {
        "剑姬 vs 瑞雯 (瞬移-5动作)"
    }

    fn description() -> &'static str {
        "剑姬 vs 瑞雯 对战强化学习环境（V0 基准：瞬移走位 + 普攻，5 离散动作）"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Discrete(5)
    }

    fn action_dim() -> usize {
        5
    }

    fn state_dim() -> usize {
        FioraVsRivenObs::dim()
    }

    fn action_labels() -> &'static [&'static str] {
        &["东移 50u", "西移 50u", "北移 50u", "南移 50u", "攻击瑞雯"]
    }

    fn obs_schema() -> Option<ObsSchema> {
        Some(super::FIORA_V0_OBS_SCHEMA.clone())
    }

    fn action_schema() -> Option<lol_rl_protocol::ActionSchema> {
        Some(FIORA_V0_ACTION_SCHEMA.clone())
    }

    fn action_from_index(idx: usize) -> Self::Action {
        FioraVsRivenAction::from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.to_index()
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        let idx = encoded.first().copied().unwrap_or(0.0) as usize;
        Self::action_from_index(idx)
    }

    fn action_to_encoding(action: Self::Action) -> Vec<f32> {
        vec![action.to_index() as f32]
    }

    fn action_name(action: Self::Action) -> &'static str {
        action.name()
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
            .unwrap_or(FioraVsRivenAction::AttackRiven);
        vec![self.step(action)]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        Some(FIORA_V0_ACTION_SCHEMA.eval_flat_mask(&obs.to_context()))
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        super::FIORA_V0_SPEC.reward_formula.clone()
    }
}

impl VisualEnvironment for FioraVsRivenEnv {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.base.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora vs Riven (Teleport) - RL Visual Viewer"
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
            .unwrap_or(FioraVsRivenAction::AttackRiven);
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
