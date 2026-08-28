use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_core::character::CharacterReady;
use lol_core::life::Health;
use lol_rl_protocol::{
    ActionSchema, ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec, RewardTermSpec,
};

pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, AttackEventTracker, FioraRivenBaseEnv, FioraRivenEntities,
    VitalBreakTracker, setup_skill_levels_world, unpause_virtual_time,
};
pub use crate::flash_plugin::{
    FLASH_COOLDOWN_SECS, FLASH_DISTANCE, FlashCooldown, dispatch_flash, extract_flash_obs,
    register_flash_plugin, tick_flash_cooldown,
};
use crate::modifier_obs::{ModifierNameId, ModifierSlotObs, extract_entity_modifiers};
use crate::obs_plugins::{extract_attack_state, extract_champion_base, extract_skill_cds};
use crate::reward::RewardModel;
use crate::traits::{EnvConfig, EnvMeta, RenderMode, RlEnvironment, StepResult, VisualEnvironment};

/// 连续偏移缩放系数：[-1, 1] 映射到相对瑞雯 ±100 单位
pub const OFFSET_SCALE: f32 = 100.0;
/// obs 向量中相对距离归一化列下标
pub const V2_OBS_DISTANCE_IDX: usize = 3;
/// 距离归一化分母
pub const V2_OBS_DISTANCE_SCALE: f32 = 100.0;
/// 靶子瑞雯在 V2 中的生命值上限
pub const RIVEN_V2_HP: f32 = 10000.0;

pub static FIORA_V2_SPEC: std::sync::LazyLock<&'static lol_rl_protocol::EnvDslSpec> =
    std::sync::LazyLock::new(|| &lol_rl_protocol::SPEC_FIORA_V2);

pub static FIORA_V2_OBS_SCHEMA: std::sync::LazyLock<ObsSchema> = std::sync::LazyLock::new(|| {
    FIORA_V2_SPEC
        .obs_schema
        .clone()
        .expect("FIORA_V2_SPEC 缺少 obs_schema")
});

pub static FIORA_V2_ACTION_SCHEMA: std::sync::LazyLock<ActionSchema> =
    std::sync::LazyLock::new(|| {
        FIORA_V2_SPEC
            .action_schema
            .clone()
            .expect("FIORA_V2_SPEC 缺少 action_schema")
    });

// ── 瑞雯血量设置与 Observer ─────────────────────────────────────────────────

pub fn setup_v2_riven_health_world(world: &mut World, riven: Entity) {
    if let Some(mut hp) = world.get_mut::<Health>(riven) {
        hp.value = RIVEN_V2_HP;
        hp.max = RIVEN_V2_HP;
    }
}

pub fn on_v2_character_ready_setup_riven_health(
    trigger: On<Add, CharacterReady>,
    entities: Res<FioraRivenEntities>,
    mut q_health: Query<&mut Health>,
) {
    if trigger.entity == entities.riven {
        if let Ok(mut hp) = q_health.get_mut(entities.riven) {
            hp.value = RIVEN_V2_HP;
            hp.max = RIVEN_V2_HP;
        }
    }
}

// ── 动作空间 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FioraV2DiscreteAction {
    NoOp = 0,
    Move = 1,
    Attack = 2,
    CastQ = 3,
    CastE = 4,
    CastR = 5,
    CastFlash = 6,
}

impl FioraV2DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            3 => Self::CastQ,
            4 => Self::CastE,
            5 => Self::CastR,
            6 => Self::CastFlash,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FioraV2Action {
    pub offset_x: f32,
    pub offset_z: f32,
    pub discrete: FioraV2DiscreteAction,
}

impl FioraV2Action {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: FioraV2DiscreteAction) -> Self {
        Self {
            offset_x,
            offset_z,
            discrete,
        }
    }

    pub fn from_encoding(encoded: &[f32]) -> Self {
        let offset_x = encoded.first().copied().unwrap_or(0.0);
        let offset_z = encoded.get(1).copied().unwrap_or(0.0);
        let discrete_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
        Self {
            offset_x,
            offset_z,
            discrete: FioraV2DiscreteAction::from_u8(discrete_idx),
        }
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![self.offset_x, self.offset_z, self.discrete.to_u8() as f32]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
            1 => Self::new(0.5, 0.0, FioraV2DiscreteAction::Move),
            2 => Self::new(-0.5, 0.0, FioraV2DiscreteAction::Move),
            3 => Self::new(0.0, 0.5, FioraV2DiscreteAction::Move),
            4 => Self::new(0.0, -0.5, FioraV2DiscreteAction::Move),
            5 => Self::new(0.0, 0.0, FioraV2DiscreteAction::Move),
            6 => Self::new(0.0, 0.0, FioraV2DiscreteAction::Attack),
            7 => Self::new(0.5, 0.0, FioraV2DiscreteAction::CastQ),
            8 => Self::new(0.0, 0.0, FioraV2DiscreteAction::CastE),
            9 => Self::new(0.0, 0.0, FioraV2DiscreteAction::CastR),
            10 => Self::new(1.0, 0.0, FioraV2DiscreteAction::CastFlash),
            _ => Self::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            FioraV2DiscreteAction::NoOp => 0,
            FioraV2DiscreteAction::Move => {
                if self.offset_x > 0.25 {
                    1
                } else if self.offset_x < -0.25 {
                    2
                } else if self.offset_z > 0.25 {
                    3
                } else if self.offset_z < -0.25 {
                    4
                } else {
                    5
                }
            }
            FioraV2DiscreteAction::Attack => 6,
            FioraV2DiscreteAction::CastQ => 7,
            FioraV2DiscreteAction::CastE => 8,
            FioraV2DiscreteAction::CastR => 9,
            FioraV2DiscreteAction::CastFlash => 10,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.preset_index() {
            0 => "保持当前 (NoOp)",
            1 => "东移 50u",
            2 => "西移 50u",
            3 => "北移 50u",
            4 => "南移 50u",
            5 => "追击瑞雯",
            6 => "普通攻击",
            7 => "Q-破空斩(东)",
            8 => "E-夺命连刺",
            9 => "R-无双挑战",
            10 => "闪现(东300u)",
            _ => "未知",
        }
    }
}

// ── 观测数据结构 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FioraV2Obs {
    pub role_id: f32,

    pub fiora_pos: Vec3,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_pos: Vec3,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
    pub distance: f32,

    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    pub q_ready: bool,
    pub q_cd_remaining: f32,
    pub e_ready: bool,
    pub e_cd_remaining: f32,
    pub r_ready: bool,
    pub r_cd_remaining: f32,

    pub flash_ready: bool,
    pub flash_cd_remaining: f32,

    /// 自身修饰符槽位 (4 槽位 × 5 = 20 维)
    pub self_modifiers: Vec<ModifierSlotObs>,
    /// 目标修饰符槽位 (4 槽位 × 5 = 20 维)
    pub target_modifiers: Vec<ModifierSlotObs>,
}

impl FioraV2Obs {
    pub fn to_context(&self) -> lol_rl_protocol::ObsContext {
        let mut ctx = lol_rl_protocol::ObsContext::new();
        ctx.set_var("role_id", self.role_id);
        ctx.set_var("fiora_x", self.fiora_pos.x);
        ctx.set_var("fiora_z", self.fiora_pos.z);
        ctx.set_var("riven_x", self.riven_pos.x);
        ctx.set_var("riven_z", self.riven_pos.z);
        ctx.set_var("distance", self.distance);

        ctx.set_var(
            "attack_is_ready",
            if self.attack_state == 0 { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "attack_is_windup",
            if self.attack_is_windup { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "attack_is_cooldown",
            if self.attack_is_cooldown { 1.0 } else { 0.0 },
        );
        ctx.set_var("attack_timer_remaining", self.attack_timer_remaining);

        ctx.set_var("q_ready", if self.q_ready { 1.0 } else { 0.0 });
        ctx.set_var("q_cd", self.q_cd_remaining);
        ctx.set_var("e_ready", if self.e_ready { 1.0 } else { 0.0 });
        ctx.set_var("e_cd", self.e_cd_remaining);
        ctx.set_var("r_ready", if self.r_ready { 1.0 } else { 0.0 });
        ctx.set_var("r_cd", self.r_cd_remaining);
        ctx.set_var("flash_ready", if self.flash_ready { 1.0 } else { 0.0 });
        ctx.set_var("flash_cd", self.flash_cd_remaining);

        ctx.set_var("fiora_hp", self.fiora_hp);
        ctx.set_var("fiora_max_hp", self.fiora_max_hp);
        ctx.set_var("riven_hp", self.riven_hp);
        ctx.set_var("riven_max_hp", self.riven_max_hp);

        let self_mods: Vec<_> = self.self_modifiers.iter().map(|m| m.to_context()).collect();
        ctx.set_repeated("self_modifiers", self_mods);

        let target_mods: Vec<_> = self
            .target_modifiers
            .iter()
            .map(|m| m.to_context())
            .collect();
        ctx.set_repeated("target_modifiers", target_mods);

        ctx
    }

    pub fn to_vector(&self) -> Vec<f32> {
        FIORA_V2_OBS_SCHEMA.eval_to_vector(&self.to_context())
    }

    pub fn dim() -> usize {
        FIORA_V2_OBS_SCHEMA.raw_dim()
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        let primary_vital = self
            .target_modifiers
            .iter()
            .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);
        let has_vital = primary_vital.is_some();
        let vital_is_active = primary_vital.map(|v| v.stack_count > 0.5).unwrap_or(false);
        let vital_dir = if let Some(v) = primary_vital {
            if v.param0 > 0.5 {
                "+X (东)".to_string()
            } else if v.param0 < -0.5 {
                "-X (西)".to_string()
            } else if v.param1 > 0.5 {
                "+Z (北)".to_string()
            } else if v.param1 < -0.5 {
                "-Z (南)".to_string()
            } else {
                "无".to_string()
            }
        } else {
            "无".to_string()
        };

        ObsFeaturePayload {
            fiora_hp_pct: if self.fiora_max_hp > 0.0 {
                self.fiora_hp / self.fiora_max_hp
            } else {
                1.0
            },
            riven_hp_pct: if self.riven_max_hp > 0.0 {
                self.riven_hp / self.riven_max_hp
            } else {
                1.0
            },
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: true,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital,
            vital_is_active,
            vital_direction: vital_dir,
            tags: HashMap::from([
                ("q_cd".to_string(), format!("{:.1}s", self.q_cd_remaining)),
                ("e_cd".to_string(), format!("{:.1}s", self.e_cd_remaining)),
                ("r_cd".to_string(), format!("{:.1}s", self.r_cd_remaining)),
                (
                    "flash_cd".to_string(),
                    format!("{:.1}s", self.flash_cd_remaining),
                ),
                (
                    "atk_state".to_string(),
                    match self.attack_state {
                        0 => "Ready".to_string(),
                        1 => format!("前摇中({:.2}s)", self.attack_timer_remaining),
                        2 => format!("后摇中({:.2}s)", self.attack_timer_remaining),
                        _ => "未知".to_string(),
                    },
                ),
                (
                    "modifiers_count".to_string(),
                    format!(
                        "Self:{}, Target:{}",
                        self.self_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                        self.target_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                    ),
                ),
            ]),
            ..Default::default()
        }
    }
}

// ── 奖励模型 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct FioraV2RewardContext {
    pub prev_aligned: bool,
    pub curr_aligned: bool,
    pub is_vital_break: bool,
    pub prev_riven_hp: f32,
    pub curr_riven_hp: f32,
    pub riven_max_hp: f32,
    pub elapsed_secs: f32,
}

pub struct FioraV2RewardModel;

impl RewardModel for FioraV2RewardModel {
    type Context = FioraV2RewardContext;

    fn formula_spec(&self) -> RewardFormulaSpec {
        use lol_rl_protocol::RewardExpr;
        RewardFormulaSpec {
            name: "全技能实战公式 (V2)".to_string(),
            terms: vec![
                RewardTermSpec::new("time_penalty", "每步时间惩罚", RewardExpr::Constant(-0.001)),
                RewardTermSpec::new(
                    "damage_dealt",
                    "造成伤害比例奖励",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(2.5)),
                        Box::new(RewardExpr::Variable("damage_ratio".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "kill_reward",
                    "击杀瑞雯奖励",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(2.0)),
                        Box::new(RewardExpr::Variable("is_kill".to_string())),
                    ),
                ),
            ],
        }
    }

    fn extract_variables(&self, ctx: &FioraV2RewardContext) -> HashMap<String, f32> {
        let hp_diff = (ctx.prev_riven_hp - ctx.curr_riven_hp).max(0.0);
        let max_hp = if ctx.riven_max_hp > 0.0 {
            ctx.riven_max_hp
        } else {
            10000.0
        };
        let damage_ratio = hp_diff / max_hp;
        let is_kill = if ctx.curr_riven_hp <= 0.0 && ctx.prev_riven_hp > 0.0 {
            1.0
        } else {
            0.0
        };

        HashMap::from([
            ("damage_ratio".to_string(), damage_ratio),
            ("hp_diff".to_string(), hp_diff),
            ("is_kill".to_string(), is_kill),
            ("elapsed_secs".to_string(), ctx.elapsed_secs),
            ("step_tick".to_string(), 1.0),
        ])
    }
}

// ── 环境主体 ────────────────────────────────────────────────────────────────

/// 统一的有头/无头世界初始化与重置逻辑（重设瑞雯 10000 血量并重置剑姬闪现）
pub fn setup_v2_fiora_riven_world(fiora: Entity, riven: Entity, world: &mut World) {
    setup_v2_riven_health_world(world, riven);
    if let Some(mut flash) = world.get_mut::<FlashCooldown>(fiora) {
        flash.reset();
    } else {
        world.entity_mut(fiora).insert(FlashCooldown::default());
    }
}

pub struct FioraV2Env {
    pub base: FioraRivenBaseEnv,
}

impl std::ops::Deref for FioraV2Env {
    type Target = FioraRivenBaseEnv;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FioraV2Env {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl FioraV2Env {
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
        let base = FioraRivenBaseEnv::builder(config, Self::DEFAULT_MAX_STEPS)
            .window_title("Fiora vs Riven (V2 Full Skills 10f) - RL Visual Viewer")
            .initial_positions(Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0))
            .with_plugin(register_flash_plugin)
            .with_observer(|app| {
                app.add_observer(on_v2_character_ready_setup_riven_health);
            })
            .on_ready(setup_v2_fiora_riven_world)
            .on_reset(setup_v2_fiora_riven_world)
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

    pub fn reset(&mut self) -> FioraV2Obs {
        self.base.reset_base();
        self.get_obs()
    }

    pub fn get_obs(&self) -> FioraV2Obs {
        get_v2_obs_from_world(self.base.world(), self.base.fiora(), self.base.riven())
    }

    pub fn dispatch_action(&mut self, action: FioraV2Action) {
        let fiora = self.base.fiora;
        let riven = self.base.riven;
        dispatch_action_world(self.base.world_mut(), fiora, riven, action);
    }

    pub fn step(&mut self, action: FioraV2Action) -> StepResult<FioraV2Obs> {
        self.base.increment_step();
        step_v2_world(
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

impl RlEnvironment for FioraV2Env {
    type Action = FioraV2Action;
    type Obs = FioraV2Obs;

    fn env_name() -> &'static str {
        "FioraV2"
    }

    fn display_name() -> &'static str {
        "剑姬 vs 瑞雯 (全技能实战-10f)"
    }

    fn description() -> &'static str {
        "剑姬 vs 瑞雯 对战强化学习环境（V2：全技能Q/E/R+闪现+普攻+NoOp，10帧物理推演）"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 7,
        }
    }

    fn action_dim() -> usize {
        Self::action_space().actor_head_dim()
    }

    fn state_dim() -> usize {
        FioraV2Obs::dim()
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "保持当前 (NoOp)",
            "东移 50u",
            "西移 50u",
            "北移 50u",
            "南移 50u",
            "追击瑞雯",
            "普通攻击",
            "Q-破空斩(东)",
            "E-夺命连刺",
            "R-无双挑战",
            "闪现(东300u)",
        ]
    }

    fn obs_schema() -> Option<ObsSchema> {
        Some(FIORA_V2_OBS_SCHEMA.clone())
    }

    fn action_schema() -> Option<ActionSchema> {
        Some(FIORA_V2_ACTION_SCHEMA.clone())
    }

    fn action_from_index(idx: usize) -> Self::Action {
        FioraV2Action::preset_from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.preset_index()
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        FioraV2Action::from_encoding(encoded)
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
        let action = actions.first().copied().unwrap_or(FioraV2Action::new(
            0.0,
            0.0,
            FioraV2DiscreteAction::NoOp,
        ));
        vec![self.step(action)]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        Some(FIORA_V2_ACTION_SCHEMA.eval_flat_mask(&obs.to_context()))
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        FIORA_V2_SPEC.reward_formula.clone()
    }
}

// ── VisualEnvironment Trait 实现 ────────────────────────────────────────────

impl VisualEnvironment for FioraV2Env {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.base.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora vs Riven (V2 Full Skills 10f) - RL Visual Viewer"
    }

    fn is_assets_loaded(&self, world: &World) -> bool {
        self.base.is_assets_loaded(world)
    }

    fn on_assets_loaded(&mut self, app: &mut App) {
        self.base.on_assets_ready(app);
    }

    fn reset_world(&mut self, app: &mut App) -> Vec<Self::Obs> {
        let (new_fiora, new_riven) = self.base.reset_app(app);
        vec![get_v2_obs_from_world(app.world(), new_fiora, new_riven)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_v2_obs_from_world(
            world,
            self.base.fiora,
            self.base.riven,
        )]
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        self.base.increment_step();
        let action = actions.first().copied().unwrap_or(FioraV2Action::new(
            0.0,
            0.0,
            FioraV2DiscreteAction::NoOp,
        ));
        vec![step_v2_world(
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

pub fn get_v2_obs_from_world(world: &World, fiora: Entity, riven: Entity) -> FioraV2Obs {
    let f_base = extract_champion_base(world, fiora);
    let r_base = extract_champion_base(world, riven);
    let dist = f_base.pos.distance(r_base.pos);

    let atk = extract_attack_state(world, fiora);
    let skills = extract_skill_cds(world, fiora);
    let (flash_ready, flash_cd) = extract_flash_obs(world, fiora);

    let self_modifiers = extract_entity_modifiers(world, fiora, 4);
    let target_modifiers = extract_entity_modifiers(world, riven, 4);

    FioraV2Obs {
        role_id: 0.0,
        fiora_pos: f_base.pos,
        fiora_hp: f_base.hp,
        fiora_max_hp: f_base.max_hp,
        riven_pos: r_base.pos,
        riven_hp: r_base.hp,
        riven_max_hp: r_base.max_hp,
        distance: dist,
        attack_state: atk.state_code,
        attack_is_windup: atk.is_windup,
        attack_is_cooldown: atk.is_cooldown,
        attack_timer_remaining: atk.timer_remaining,
        q_ready: skills[0].ready,
        q_cd_remaining: skills[0].cd_remaining,
        e_ready: skills[2].ready,
        e_cd_remaining: skills[2].cd_remaining,
        r_ready: skills[3].ready,
        r_cd_remaining: skills[3].cd_remaining,
        flash_ready,
        flash_cd_remaining: flash_cd,
        self_modifiers,
        target_modifiers,
    }
}

pub fn dispatch_action_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    action: FioraV2Action,
) {
    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();

    let target_pos = Vec3::new(
        rpos.x + action.offset_x.clamp(-1.0, 1.0) * OFFSET_SCALE,
        rpos.y,
        rpos.z + action.offset_z.clamp(-1.0, 1.0) * OFFSET_SCALE,
    );

    match action.discrete {
        FioraV2DiscreteAction::NoOp => {}
        FioraV2DiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Move(Vec2::new(target_pos.x, target_pos.z)),
            });
        }
        FioraV2DiscreteAction::Attack => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Attack(riven),
            });
        }
        FioraV2DiscreteAction::CastQ => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 0,
                    point: Vec2::new(target_pos.x, target_pos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastE => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 2,
                    point: Vec2::new(fpos.x, fpos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastR => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 3,
                    point: Vec2::new(rpos.x, rpos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastFlash => {
            let offset_dir = Vec3::new(action.offset_x, 0.0, action.offset_z);
            let dir = if offset_dir.length_squared() > 1e-4 {
                offset_dir.normalize()
            } else {
                let to_riven = rpos - fpos;
                if to_riven.length_squared() > 1e-4 {
                    to_riven.normalize()
                } else {
                    Vec3::X
                }
            };
            dispatch_flash(world, fiora, dir, FLASH_DISTANCE);
        }
    }

    dispatch_v2_riven_action(world, fiora, riven);
}

fn dispatch_v2_riven_action(world: &mut World, fiora: Entity, riven: Entity) {
    let rhp = world.get::<Health>(riven).map(|h| h.value).unwrap_or(0.0);
    if rhp <= 0.0 {
        return;
    }

    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();

    let roll = rand::random::<f32>();
    let target = if roll < 0.5 {
        let away = (rpos - fpos).normalize_or_zero();
        let dir = if away.length_squared() > 1e-4 {
            away
        } else {
            Vec3::X
        };
        rpos + dir * 300.0
    } else {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        rpos + Vec3::new(angle.cos(), 0.0, angle.sin()) * 300.0
    };

    world.trigger(CommandAction {
        entity: riven,
        action: Action::Move(Vec2::new(target.x, target.z)),
    });
}

pub fn is_v2_aligned_with_vital(fpos: Vec3, rpos: Vec3, obs: &FioraV2Obs) -> bool {
    let delta_x = fpos.x - rpos.x;
    let delta_z = fpos.z - rpos.z;
    let abs_delta_x = delta_x.abs();
    let abs_delta_z = delta_z.abs();

    let primary_vital = obs
        .target_modifiers
        .iter()
        .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);

    if let Some(v) = primary_vital {
        if v.param0 > 0.5 {
            delta_x > 0.0 && abs_delta_x > abs_delta_z
        } else if v.param0 < -0.5 {
            delta_x < 0.0 && abs_delta_x > abs_delta_z
        } else if v.param1 > 0.5 {
            delta_z > 0.0 && abs_delta_z > abs_delta_x
        } else if v.param1 < -0.5 {
            delta_z < 0.0 && abs_delta_z > abs_delta_x
        } else {
            false
        }
    } else {
        false
    }
}

pub fn step_v2_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraV2Action,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraV2Obs> {
    let prev_obs = get_v2_obs_from_world(app.world(), fiora, riven);
    let prev_riven_hp = prev_obs.riven_hp;
    let prev_fpos = prev_obs.fiora_pos;

    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    dispatch_action_world(app.world_mut(), fiora, riven, action);
    unpause_virtual_time(app.world_mut());

    for _ in 0..10 {
        app.update();
    }

    let obs = get_v2_obs_from_world(app.world(), fiora, riven);
    let curr_riven_hp = obs.riven_hp;
    let curr_fpos = obs.fiora_pos;

    let tracker_hit = app.world().resource::<VitalBreakTracker>().hit;
    let had_active_vital = prev_obs
        .target_modifiers
        .iter()
        .any(|m| m.name_id == ModifierNameId::FioraPassiveVital && m.stack_count > 0.5);
    let is_vital_break = tracker_hit && had_active_vital;

    let has_vital = prev_obs
        .target_modifiers
        .iter()
        .any(|m| m.name_id == ModifierNameId::FioraPassiveVital);
    let prev_aligned =
        has_vital && is_v2_aligned_with_vital(prev_fpos, prev_obs.riven_pos, &prev_obs);
    let curr_aligned =
        has_vital && is_v2_aligned_with_vital(curr_fpos, prev_obs.riven_pos, &prev_obs);

    let ctx = FioraV2RewardContext {
        prev_aligned,
        curr_aligned,
        is_vital_break,
        prev_riven_hp,
        curr_riven_hp,
        riven_max_hp: prev_obs.riven_max_hp,
        elapsed_secs: step_count as f32 * (10.0 / 60.0),
    };

    let model = FioraV2RewardModel;
    let (reward, items, vars) = model.evaluate(&ctx);

    let reward_breakdown = items
        .into_iter()
        .map(|it| crate::traits::RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    let terminated = curr_riven_hp <= 0.0 || obs.fiora_hp <= 0.0;
    let truncated = step_count >= max_steps;

    StepResult {
        obs,
        reward,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown,
        reward_variables: vars,
    }
}
