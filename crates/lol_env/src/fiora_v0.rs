use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, RewardFormulaSpec};

pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, AttackEventTracker, FioraRivenBaseEnv, FioraVsRivenObs,
    VitalBreakTracker, compute_step_reward, get_obs_from_world, reset_episode_world,
    setup_skill_levels_world, unpause_virtual_time,
};
use crate::reward::{FioraVsRivenRewardModel, RewardModel};
use crate::traits::{EnvConfig, EnvMeta, RenderMode, RlEnvironment, StepResult, VisualEnvironment};

// ── 动作空间 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FioraVsRivenAction {
    MoveEast50 = 0,
    MoveWest50 = 1,
    MoveNorth50 = 2,
    MoveSouth50 = 3,
    AttackRiven = 4,
}

impl FioraVsRivenAction {
    #[allow(non_upper_case_globals)]
    pub const TeleportEast50: Self = Self::MoveEast50;
    #[allow(non_upper_case_globals)]
    pub const TeleportWest50: Self = Self::MoveWest50;
    #[allow(non_upper_case_globals)]
    pub const TeleportNorth50: Self = Self::MoveNorth50;
    #[allow(non_upper_case_globals)]
    pub const TeleportSouth50: Self = Self::MoveSouth50;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::MoveEast50,
            1 => Self::MoveWest50,
            2 => Self::MoveNorth50,
            3 => Self::MoveSouth50,
            4 => Self::AttackRiven,
            _ => Self::AttackRiven,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MoveEast50 => "东移50u",
            Self::MoveWest50 => "西移50u",
            Self::MoveNorth50 => "北移50u",
            Self::MoveSouth50 => "南移50u",
            Self::AttackRiven => "攻击瑞雯",
        }
    }
}

// ── 环境主体 ────────────────────────────────────────────────────────────────

pub struct FioraVsRivenEnv {
    pub base: FioraRivenBaseEnv,
}

impl std::ops::Deref for FioraVsRivenEnv {
    type Target = FioraRivenBaseEnv;
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
        let base = FioraRivenBaseEnv::builder(config, Self::DEFAULT_MAX_STEPS)
            .window_title("Fiora vs Riven (Teleport) - RL Visual Viewer")
            .initial_positions(Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0))
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

    pub fn initial_fiora_pos(&self) -> Vec3 {
        self.base.initial_fiora_pos()
    }

    pub fn initial_riven_pos(&self) -> Vec3 {
        self.base.initial_riven_pos()
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
        let fiora = self.base.fiora;
        let riven = self.base.riven;
        dispatch_action_world(self.base.world_mut(), fiora, riven, action);
    }

    pub fn step(&mut self, action: FioraVsRivenAction) -> StepResult<FioraVsRivenObs> {
        self.base.increment_step();
        step_world(
            &mut self.base.app,
            self.base.fiora,
            self.base.riven,
            action,
            self.base.step_count,
            self.base.max_steps,
        )
    }
}

// ── RlEnvironment Trait 实现 ────────────────────────────────────────────────

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

    fn obs_dim_labels() -> &'static [&'static str] {
        &[
            "破绽方向(+X/东)",
            "破绽方向(-X/西)",
            "破绽方向(+Z/北)",
            "破绽方向(-Z/南)",
            "存在破绽",
            "破绽已激活",
            "相对位置X(归一化)",
            "相对位置Z(归一化)",
            "相对距离(归一化)",
        ]
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

    fn is_action_masked(obs: &Self::Obs, action_idx: usize) -> bool {
        obs.distance > ATTACK_MASK_DISTANCE && action_idx == 4
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        let mut mask = vec![true; 5];
        if obs.distance > ATTACK_MASK_DISTANCE {
            mask[4] = false;
        }
        Some(mask)
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        Some(FioraVsRivenRewardModel.formula_spec())
    }
}

// ── VisualEnvironment Trait 实现 ────────────────────────────────────────────

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

    fn on_assets_loaded(&mut self, world: &mut World) {
        setup_skill_levels_world(world, self.base.fiora, self.base.riven);
    }

    fn reset_world(&mut self, world: &mut World) -> Vec<Self::Obs> {
        let (new_fiora, new_riven) = self.base.reset_world_base(world);
        vec![get_obs_from_world(world, new_fiora, new_riven)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_obs_from_world(world, self.base.fiora, self.base.riven)]
    }

    fn action_from_screen_click(
        &mut self,
        _world: &mut World,
        _screen_pos: Vec2,
    ) -> Option<FioraVsRivenAction> {
        None
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
            self.base.fiora,
            self.base.riven,
            action,
            self.base.step_count,
            self.base.max_steps,
        )]
    }
}

// ── 自由函数 ────────────────────────────────────────────────────────────────

pub fn dispatch_action_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
) {
    match action {
        FioraVsRivenAction::MoveEast50
        | FioraVsRivenAction::MoveWest50
        | FioraVsRivenAction::MoveNorth50
        | FioraVsRivenAction::MoveSouth50 => {
            let rpos = world
                .get::<Transform>(riven)
                .map(|t| t.translation)
                .unwrap_or_default();
            let new_pos = match action {
                FioraVsRivenAction::MoveEast50 => Vec3::new(rpos.x + 50.0, rpos.y, rpos.z),
                FioraVsRivenAction::MoveWest50 => Vec3::new(rpos.x - 50.0, rpos.y, rpos.z),
                FioraVsRivenAction::MoveNorth50 => Vec3::new(rpos.x, rpos.y, rpos.z + 50.0),
                FioraVsRivenAction::MoveSouth50 => Vec3::new(rpos.x, rpos.y, rpos.z - 50.0),
                _ => unreachable!(),
            };
            if let Some(mut t) = world.get_mut::<Transform>(fiora) {
                t.translation = new_pos;
            }
        }
        FioraVsRivenAction::AttackRiven => {}
    }
}

pub fn advance_action_simulation(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
) -> Option<FioraVsRivenObs> {
    if action == FioraVsRivenAction::AttackRiven {
        for _ in 0..300 {
            let is_active = app
                .world()
                .get::<lol_champions::fiora::passive::Vital>(riven)
                .map(|v| v.is_active())
                .unwrap_or(false);
            if is_active {
                break;
            }
            app.update();
        }

        let attack_obs = get_obs_from_world(app.world(), fiora, riven);

        app.world_mut().trigger(CommandAction {
            entity: fiora,
            action: Action::Attack(riven),
        });

        if let Some(mut tracker) = app.world_mut().get_resource_mut::<AttackEventTracker>() {
            tracker.attack_hit = false;
            tracker.attack_ready = false;
        }

        for _ in 0..100 {
            app.update();
            let tracker = app.world().resource::<AttackEventTracker>();
            if tracker.attack_hit && tracker.attack_ready {
                break;
            }
        }

        app.world_mut()
            .trigger(lol_core::attack_auto::CommandAttackAutoStop { entity: fiora });

        Some(attack_obs)
    } else {
        app.update();
        None
    }
}

pub fn step_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraVsRivenObs> {
    let prev_obs = get_obs_from_world(app.world(), fiora, riven);
    let prev_fpos = prev_obs.fiora_pos;
    let prev_riven_hp = prev_obs.riven_hp;

    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    dispatch_action_world(app.world_mut(), fiora, riven, action);
    unpause_virtual_time(app.world_mut());

    let attack_obs = advance_action_simulation(app, fiora, riven, action);

    let obs = get_obs_from_world(app.world(), fiora, riven);
    let curr_fpos = obs.fiora_pos;
    let curr_riven_hp = obs.riven_hp;

    let is_attack = action == FioraVsRivenAction::AttackRiven;
    let tracker_hit = app.world().resource::<VitalBreakTracker>().hit;
    let is_vital_break = tracker_hit && prev_obs.has_vital && prev_obs.vital_is_active;

    let reward_obs = attack_obs.as_ref().unwrap_or(&prev_obs);
    let (reward, reward_breakdown, reward_vars) = compute_step_reward(
        prev_riven_hp,
        curr_riven_hp,
        prev_fpos,
        curr_fpos,
        prev_obs.riven_pos,
        is_attack,
        is_vital_break,
        reward_obs,
        step_count as f32 / 60.0,
    );

    let terminated = curr_riven_hp <= 0.0 || obs.fiora_hp <= 0.0;
    let truncated = step_count >= max_steps;

    StepResult {
        obs,
        reward,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown,
        reward_variables: reward_vars,
    }
}
