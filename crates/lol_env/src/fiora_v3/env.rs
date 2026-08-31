use bevy::prelude::*;
use lol_core::team::Team;
use lol_rl_protocol::{ActionSchema, ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec};

use super::action::{FIORA_V3_ACTION_SCHEMA, FioraV3Action, FioraV3DiscreteAction};
use super::obs::{FIORA_V3_OBS_SCHEMA, FioraV3Obs, get_ego_obs_from_world};
use super::step::{setup_fiora_v3_env_world, step_fiora_v3_world};
use crate::base_env::{LolBaseEnv, fiora_champion_spec};
use crate::curriculum::CurriculumRewardConfig;
use crate::traits::{EnvConfig, EnvMeta, RenderMode, RlEnvironment, StepResult, VisualEnvironment};

pub struct FioraV3Env {
    pub base: LolBaseEnv,
}

impl std::ops::Deref for FioraV3Env {
    type Target = LolBaseEnv;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FioraV3Env {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl FioraV3Env {
    pub const DEFAULT_MAX_STEPS: usize = 160;

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
            .window_title("Fiora V3 (Last Hit Viewer)")
            .map_name("solo")
            .enable_barrack(true)
            .warmup_secs(30.0)
            .add_champion(fiora_champion_spec(
                Team::Order,
                Vec3::new(2350.0, 0.0, 12750.0),
                [0, 0, 0, 0],
                true,
            ))
            .on_ready(setup_fiora_v3_env_world)
            .on_reset(setup_fiora_v3_env_world)
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

    pub fn max_steps(&self) -> usize {
        self.base.max_steps()
    }

    pub fn step_count(&self) -> usize {
        self.base.step_count()
    }
}

impl RlEnvironment for FioraV3Env {
    type Action = FioraV3Action;
    type Obs = FioraV3Obs;

    fn num_agents() -> usize {
        1
    }

    fn agent_names() -> &'static [&'static str] {
        &["Fiora"]
    }

    fn env_name() -> &'static str {
        "FioraV3"
    }

    fn display_name() -> &'static str {
        "Fiora V3 (补刀训练)"
    }

    fn description() -> &'static str {
        "剑姬在召唤师峡谷上路Solo地图进行单人补刀训练（补刀成功奖励，普通攻击未补刀惩罚）"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 3,
        }
    }

    fn action_dim() -> usize {
        Self::action_space().actor_head_dim()
    }

    fn state_dim() -> usize {
        FioraV3Obs::dim()
    }

    fn action_labels() -> &'static [&'static str] {
        &["保持当前 (NoOp)", "移动 (Move)", "普通攻击 (Attack)"]
    }

    fn obs_schema() -> Option<ObsSchema> {
        Some(FIORA_V3_OBS_SCHEMA.clone())
    }

    fn action_schema() -> Option<ActionSchema> {
        Some(FIORA_V3_ACTION_SCHEMA.clone())
    }

    fn action_from_index(idx: usize) -> Self::Action {
        FioraV3Action::preset_from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.preset_index()
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        FioraV3Action::from_encoding(encoded)
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
        self.base.reset_base();
        let fiora = self.base.fiora();
        vec![get_ego_obs_from_world(self.base.world(), fiora, 0.0)]
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let fiora_action = actions.first().copied().unwrap_or(FioraV3Action::new(
            0.0,
            0.0,
            FioraV3DiscreteAction::NoOp,
        ));

        self.base.increment_step();
        let fiora = self.base.fiora();
        let res = step_fiora_v3_world(
            &mut self.base.app,
            fiora,
            fiora_action,
            self.base.step_count,
            self.base.max_steps,
        );
        vec![res]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        let is_cooldown = obs.attack_is_cooldown;

        Some(vec![true, true, !is_cooldown])
    }

    fn action_masks(obs: &Self::Obs) -> Option<lol_rl_protocol::ActionMasks> {
        Some(FIORA_V3_ACTION_SCHEMA.eval_action_masks(&obs.to_context()))
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        super::FIORA_V3_SPEC.reward_formula.clone()
    }

    fn default_curriculum() -> Option<lol_rl_protocol::CurriculumConfig> {
        None
    }

    fn update_curriculum(
        &mut self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    ) {
        let cfg = CurriculumRewardConfig {
            cs_reward,
            attack_no_cs_penalty,
            harass_coef,
            minion_hp_scale: hp_scale,
        };
        self.base.app.world_mut().insert_resource(cfg);
    }
}

impl VisualEnvironment for FioraV3Env {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.base.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora V3 (Last Hit Viewer)"
    }

    fn is_assets_loaded(&self, world: &World) -> bool {
        self.base.is_assets_loaded(world)
    }

    fn on_assets_loaded(&mut self, app: &mut App) {
        self.base.on_assets_ready(app);
    }

    fn reset_world(&mut self, app: &mut App) -> Vec<Self::Obs> {
        let champions = self.base.reset_app(app);
        let fiora = champions[0];
        vec![get_ego_obs_from_world(app.world(), fiora, 0.0)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_ego_obs_from_world(world, self.base.fiora(), 0.0)]
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        let fiora_action = actions.first().copied().unwrap_or(FioraV3Action::new(
            0.0,
            0.0,
            FioraV3DiscreteAction::NoOp,
        ));

        self.base.increment_step();
        let res = step_fiora_v3_world(
            app,
            self.base.fiora(),
            fiora_action,
            self.base.step_count,
            self.base.max_steps,
        );
        vec![res]
    }
}
