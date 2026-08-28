use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_core::base::stats::ChampionStats;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::team::Team;
use lol_rl_protocol::{
    ActionSchema, ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec,
};

use crate::curriculum::CurriculumRewardConfig;
pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, AttackEventTracker, FioraRivenBaseEnv, FioraRivenEntities,
    setup_skill_levels_world, unpause_virtual_time,
};
pub use crate::flash_plugin::{
    FLASH_COOLDOWN_SECS, FLASH_DISTANCE, FlashCooldown, dispatch_flash, extract_flash_obs,
    register_flash_plugin, tick_flash_cooldown,
};
use crate::modifier_obs::{ModifierNameId, ModifierSlotObs, extract_entity_modifiers};
use crate::obs_plugins::{extract_attack_state, extract_champion_base, extract_skill_cds};
use crate::traits::{
    EnvConfig, EnvMeta, RenderMode, RewardBreakdownItem, RlEnvironment, StepResult,
    VisualEnvironment,
};

// ── 常量定义 ─────────────────────────────────────────────────────────────────

pub const FIORA_V3_OFFSET_SCALE: f32 = 100.0;
pub const FIORA_V3_MAX_VISIBLE_UNITS: usize = 20;
pub const FIORA_V3_OBS_DISTANCE_SCALE: f32 = 100.0;

pub static FIORA_V3_SPEC: std::sync::LazyLock<&'static lol_rl_protocol::EnvDslSpec> =
    std::sync::LazyLock::new(|| &lol_rl_protocol::SPEC_FIORA_V3);

pub static FIORA_V3_OBS_SCHEMA: std::sync::LazyLock<ObsSchema> = std::sync::LazyLock::new(|| {
    FIORA_V3_SPEC
        .obs_schema
        .clone()
        .expect("FIORA_V3_SPEC 缺少 obs_schema")
});

pub static FIORA_V3_ACTION_SCHEMA: std::sync::LazyLock<ActionSchema> =
    std::sync::LazyLock::new(|| {
        FIORA_V3_SPEC
            .action_schema
            .clone()
            .expect("FIORA_V3_SPEC 缺少 action_schema")
    });

// ── 初始化与重置 ─────────────────────────────────────────────────────────────

pub fn setup_fiora_v3_health_world(world: &mut World, fiora: Entity, riven: Entity) {
    for champion in [fiora, riven] {
        if let Some(mut hp) = world.get_mut::<Health>(champion) {
            hp.value = hp.max;
        }
        if let Some(mut flash) = world.get_mut::<FlashCooldown>(champion) {
            flash.reset();
        } else {
            world.entity_mut(champion).insert(FlashCooldown::default());
        }
        if let Some(mut stats) = world.get_mut::<ChampionStats>(champion) {
            stats.kills = 0;
            stats.deaths = 0;
            stats.assists = 0;
            stats.minion_kills = 0;
        }
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FioraV3DiscreteAction {
    NoOp = 0,
    Move = 1,
    Attack = 2,
    CastQ = 3,
    CastW = 4,
    CastE = 5,
    CastR = 6,
    CastFlash = 7,
}

impl FioraV3DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            3 => Self::CastQ,
            4 => Self::CastW,
            5 => Self::CastE,
            6 => Self::CastR,
            7 => Self::CastFlash,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FioraV3Action {
    pub offset_x: f32,
    pub offset_z: f32,
    pub target_idx: u8,
    pub discrete: FioraV3DiscreteAction,
}

impl FioraV3Action {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: FioraV3DiscreteAction) -> Self {
        Self {
            offset_x,
            offset_z,
            target_idx: 0,
            discrete,
        }
    }

    pub const fn with_target(
        offset_x: f32,
        offset_z: f32,
        target_idx: u8,
        discrete: FioraV3DiscreteAction,
    ) -> Self {
        Self {
            offset_x,
            offset_z,
            target_idx,
            discrete,
        }
    }

    pub fn from_encoding(encoded: &[f32]) -> Self {
        let offset_x = encoded.first().copied().unwrap_or(0.0);
        let offset_z = encoded.get(1).copied().unwrap_or(0.0);
        if encoded.len() >= 4 {
            let target_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
            let discrete_idx = encoded.get(3).copied().unwrap_or(0.0) as u8;
            Self {
                offset_x,
                offset_z,
                target_idx,
                discrete: FioraV3DiscreteAction::from_u8(discrete_idx),
            }
        } else {
            let discrete_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
            Self {
                offset_x,
                offset_z,
                target_idx: 0,
                discrete: FioraV3DiscreteAction::from_u8(discrete_idx),
            }
        }
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![
            self.offset_x,
            self.offset_z,
            self.target_idx as f32,
            self.discrete.to_u8() as f32,
        ]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, FioraV3DiscreteAction::NoOp),
            1 => Self::new(0.5, 0.0, FioraV3DiscreteAction::Move),
            2 => Self::new(0.0, 0.0, FioraV3DiscreteAction::Attack),
            3 => Self::new(0.5, 0.0, FioraV3DiscreteAction::CastQ),
            4 => Self::new(0.0, 0.0, FioraV3DiscreteAction::CastW),
            5 => Self::new(0.0, 0.0, FioraV3DiscreteAction::CastE),
            6 => Self::new(0.0, 0.0, FioraV3DiscreteAction::CastR),
            7 => Self::new(1.0, 0.0, FioraV3DiscreteAction::CastFlash),
            _ => Self::new(0.0, 0.0, FioraV3DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            FioraV3DiscreteAction::NoOp => 0,
            FioraV3DiscreteAction::Move => 1,
            FioraV3DiscreteAction::Attack => 2,
            FioraV3DiscreteAction::CastQ => 3,
            FioraV3DiscreteAction::CastW => 4,
            FioraV3DiscreteAction::CastE => 5,
            FioraV3DiscreteAction::CastR => 6,
            FioraV3DiscreteAction::CastFlash => 7,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.discrete {
            FioraV3DiscreteAction::NoOp => "保持当前 (NoOp)",
            FioraV3DiscreteAction::Move => "移动",
            FioraV3DiscreteAction::Attack => "普通攻击",
            FioraV3DiscreteAction::CastQ => "施放 Q",
            FioraV3DiscreteAction::CastW => "施放 W",
            FioraV3DiscreteAction::CastE => "施放 E",
            FioraV3DiscreteAction::CastR => "施放 R",
            FioraV3DiscreteAction::CastFlash => "闪现",
        }
    }
}

// ── 自我中心化观测数据结构 ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FioraV3Obs {
    pub role_id: f32,

    pub self_pos: Vec3,
    pub self_hp: f32,
    pub self_max_hp: f32,
    pub target_pos: Vec3,
    pub target_hp: f32,
    pub target_max_hp: f32,
    pub distance: f32,

    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    pub q_ready: bool,
    pub q_cd_remaining: f32,
    pub w_ready: bool,
    pub w_cd_remaining: f32,
    pub e_ready: bool,
    pub e_cd_remaining: f32,
    pub r_ready: bool,
    pub r_cd_remaining: f32,

    pub flash_ready: bool,
    pub flash_cd_remaining: f32,

    pub self_modifiers: Vec<ModifierSlotObs>,
    pub target_modifiers: Vec<ModifierSlotObs>,

    pub visible_units: Vec<lol_rl_protocol::ObsContext>,
    pub visible_unit_entities: Vec<Option<Entity>>,
}

impl FioraV3Obs {
    pub fn to_context(&self) -> lol_rl_protocol::ObsContext {
        let mut ctx = lol_rl_protocol::ObsContext::new();
        ctx.set_var("role_id", self.role_id);
        ctx.set_var("self_x", self.self_pos.x);
        ctx.set_var("self_z", self.self_pos.z);
        ctx.set_var("target_x", self.target_pos.x);
        ctx.set_var("target_z", self.target_pos.z);
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
        ctx.set_var("w_ready", if self.w_ready { 1.0 } else { 0.0 });
        ctx.set_var("w_cd", self.w_cd_remaining);
        ctx.set_var("e_ready", if self.e_ready { 1.0 } else { 0.0 });
        ctx.set_var("e_cd", self.e_cd_remaining);
        ctx.set_var("r_ready", if self.r_ready { 1.0 } else { 0.0 });
        ctx.set_var("r_cd", self.r_cd_remaining);
        ctx.set_var("flash_ready", if self.flash_ready { 1.0 } else { 0.0 });
        ctx.set_var("flash_cd", self.flash_cd_remaining);

        ctx.set_var("self_hp", self.self_hp);
        ctx.set_var("self_max_hp", self.self_max_hp);
        ctx.set_var("target_hp", self.target_hp);
        ctx.set_var("target_max_hp", self.target_max_hp);

        let self_mods: Vec<_> = self.self_modifiers.iter().map(|m| m.to_context()).collect();
        ctx.set_repeated("self_modifiers", self_mods);

        let target_mods: Vec<_> = self
            .target_modifiers
            .iter()
            .map(|m| m.to_context())
            .collect();
        ctx.set_repeated("target_modifiers", target_mods);

        ctx.set_repeated("visible_units", self.visible_units.clone());

        ctx
    }

    pub fn to_vector(&self) -> Vec<f32> {
        FIORA_V3_OBS_SCHEMA.eval_to_vector(&self.to_context())
    }

    pub fn dim() -> usize {
        FIORA_V3_OBS_SCHEMA.raw_dim()
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        let (fiora_hp, riven_hp, f_max, r_max) = if self.role_id < 0.5 {
            (
                self.self_hp,
                self.target_hp,
                self.self_max_hp,
                self.target_max_hp,
            )
        } else {
            (
                self.target_hp,
                self.self_hp,
                self.target_max_hp,
                self.self_max_hp,
            )
        };

        let primary_vital = self
            .target_modifiers
            .iter()
            .chain(self.self_modifiers.iter())
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
            fiora_hp_pct: if f_max > 0.0 { fiora_hp / f_max } else { 1.0 },
            riven_hp_pct: if r_max > 0.0 { riven_hp / r_max } else { 1.0 },
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: self.w_ready,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital,
            vital_is_active,
            vital_direction: vital_dir,
            tags: HashMap::from([
                (
                    "role".to_string(),
                    if self.role_id < 0.5 {
                        "剑姬 (Fiora)".to_string()
                    } else {
                        "瑞雯 (Riven)".to_string()
                    },
                ),
                ("q_cd".to_string(), format!("{:.1}s", self.q_cd_remaining)),
                ("w_cd".to_string(), format!("{:.1}s", self.w_cd_remaining)),
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

    /// 检查指定目标槽位是否为有效敌方单位（0 号恒为敌方英雄，1..6 为小兵，需有效且 is_enemy > 0.5）
    pub fn is_target_enemy(&self, target_idx: usize) -> bool {
        if target_idx == 0 {
            true
        } else if let Some(unit) = self.visible_units.get(target_idx) {
            let unit_type = unit.vars.get("unit_type").copied().unwrap_or(0.0);
            let is_enemy = unit.vars.get("is_enemy").copied().unwrap_or(0.0);
            unit_type > 0.0 && is_enemy > 0.5
        } else {
            false
        }
    }
}

// ── 环境主体 ─────────────────────────────────────────────────────────────────

/// 对世界中小兵应用非对称对比课程血量缩放 (Contrastive Curriculum)
///
/// 当 scale < 1.0 时，每队挑选 1 只小兵设置为残血（health.max * scale），其余小兵保持 100% 满血，
/// 创造明确的“残血 vs 满血”对比度，迫使注意力机制学习通过 hp_pct 挑选残血目标。
/// 当 scale >= 1.0 时，所有小兵全部恢复 100% 满血（真实自然对线）。
pub fn apply_minion_hp_scale(world: &mut World, scale: f32) {
    if scale >= 1.0 {
        let mut q = world.query_filtered::<&mut Health, With<Minion>>();
        for mut health in q.iter_mut(world) {
            health.value = health.max;
        }
        return;
    }

    let mut order_minions = Vec::new();
    let mut chaos_minions = Vec::new();

    {
        let mut q = world.query_filtered::<(Entity, &Team), With<Minion>>();
        for (entity, team) in q.iter(world) {
            match team {
                Team::Order => order_minions.push(entity),
                Team::Chaos => chaos_minions.push(entity),
                _ => {}
            }
        }
    }

    for minion_list in [order_minions, chaos_minions] {
        if minion_list.is_empty() {
            continue;
        }
        // 阶梯式残血对比：前 3 只小兵设置递增残血梯度 (scale, scale * 2.5, scale * 4.0)，其余小兵保持 100% 满血
        for (i, entity) in minion_list.into_iter().enumerate() {
            if let Some(mut health) = world.get_mut::<Health>(entity) {
                if i < 3 {
                    let factor = match i {
                        0 => 1.0,
                        1 => 2.5,
                        _ => 4.0,
                    };
                    let target_hp = (health.max * (scale * factor).min(0.95)).max(1.0);
                    health.value = target_hp;
                } else {
                    health.value = health.max;
                }
            }
        }
    }
}

/// 统一的有头/无头世界初始化与重置逻辑（双方满血、闪现重置与小兵课程血量设置）
pub fn setup_fiora_v3_env_world(fiora: Entity, riven: Entity, world: &mut World) {
    setup_fiora_v3_health_world(world, fiora, riven);
    let scale = world
        .get_resource::<CurriculumRewardConfig>()
        .map(|c| c.minion_hp_scale)
        .unwrap_or(1.0);
    apply_minion_hp_scale(world, scale);
}

pub struct FioraV3Env {
    pub base: FioraRivenBaseEnv,
}

impl std::ops::Deref for FioraV3Env {
    type Target = FioraRivenBaseEnv;
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
        let base = FioraRivenBaseEnv::builder(config, Self::DEFAULT_MAX_STEPS)
            .window_title("Fiora V3 (Last Hit Viewer)")
            .map_name("solo")
            .enable_barrack(true)
            .initial_positions(
                Vec3::new(2350.0, 0.0, 12750.0),
                Vec3::new(2450.0, 0.0, 12850.0),
            )
            .initial_skill_levels([1, 0, 0, 0])
            .warmup_secs(30.0)
            .with_plugin(register_flash_plugin)
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

    pub fn riven(&self) -> Entity {
        self.base.riven()
    }

    pub fn max_steps(&self) -> usize {
        self.base.max_steps()
    }

    pub fn step_count(&self) -> usize {
        self.base.step_count()
    }
}

// ── RlEnvironment Trait 实现 ─────────────────────────────────────────────────

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
        "剑姬在召唤师峡谷上路Solo地图进行对线补刀训练（补刀成功奖励，普通攻击未补刀惩罚）"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        }
    }

    fn action_dim() -> usize {
        Self::action_space().actor_head_dim()
    }

    fn state_dim() -> usize {
        FioraV3Obs::dim()
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "保持当前 (NoOp)",
            "移动 (Move)",
            "普通攻击 (Attack)",
            "施放 Q",
            "施放 W",
            "施放 E",
            "施放 R",
            "闪现",
        ]
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
        vec![get_ego_obs_from_world(
            self.base.world(),
            self.base.fiora,
            self.base.riven,
            0.0,
        )]
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let fiora_action = actions.first().copied().unwrap_or(FioraV3Action::new(
            0.0,
            0.0,
            FioraV3DiscreteAction::NoOp,
        ));
        let riven_action = get_default_riven_combat_action(
            self.base.world(),
            self.base.riven,
            self.base.fiora,
        );

        self.base.increment_step();
        let res = step_fiora_v3_world(
            &mut self.base.app,
            self.base.fiora,
            self.base.riven,
            fiora_action,
            riven_action,
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
        let dist_ok = obs.distance <= ATTACK_MASK_DISTANCE;

        Some(vec![
            true,
            true,
            dist_ok && !is_cooldown,
            obs.q_ready,
            obs.w_ready,
            obs.e_ready,
            obs.r_ready,
            obs.flash_ready,
        ])
    }

    fn action_masks(obs: &Self::Obs) -> Option<lol_rl_protocol::ActionMasks> {
        let is_cooldown = obs.attack_is_cooldown;
        let dist_ok = obs.distance <= ATTACK_MASK_DISTANCE;

        let enemy_action_mask = vec![
            true,                    // 0: NoOp
            true,                    // 1: Move
            dist_ok && !is_cooldown, // 2: Attack
            obs.q_ready,             // 3: CastQ
            obs.w_ready,             // 4: CastW
            obs.e_ready,             // 5: CastE
            obs.r_ready,             // 6: CastR
            obs.flash_ready,         // 7: Flash
        ];

        let ally_action_mask = vec![
            true,  // 0: NoOp
            true,  // 1: Move
            false, // 2: Attack (友军不可攻击)
            false, // 3: CastQ (不可对友军施放伤害技能)
            false, // 4: CastW
            false, // 5: CastE
            false, // 6: CastR
            false, // 7: Flash
        ];

        let mut conditional_target_masks = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);
        let mut target_valid_mask = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);

        for target_idx in 0..FIORA_V3_MAX_VISIBLE_UNITS {
            if target_idx == 0 {
                target_valid_mask.push(true);
                conditional_target_masks.push(enemy_action_mask.clone());
            } else if let Some(unit) = obs.visible_units.get(target_idx) {
                let unit_type = unit.vars.get("unit_type").copied().unwrap_or(0.0);
                let is_enemy = unit.vars.get("is_enemy").copied().unwrap_or(0.0);
                let is_valid_unit = unit_type > 0.0;
                target_valid_mask.push(is_valid_unit);
                if is_valid_unit && is_enemy > 0.5 {
                    conditional_target_masks.push(enemy_action_mask.clone());
                } else {
                    conditional_target_masks.push(ally_action_mask.clone());
                }
            } else {
                target_valid_mask.push(false);
                conditional_target_masks.push(ally_action_mask.clone());
            }
        }

        Some(lol_rl_protocol::ActionMasks::with_conditional_target_masks(
            vec![
                None,                          // 0: offset (Continuous)
                Some(target_valid_mask),       // 1: target (UnitSelection)
                Some(enemy_action_mask),       // 2: action_type 兜底基线
            ],
            conditional_target_masks,
        ))
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        FIORA_V3_SPEC.reward_formula.clone()
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

        // 对当前存活的所有小兵应用血量缩放
        apply_minion_hp_scale(self.base.app.world_mut(), hp_scale);
    }
}

// ── VisualEnvironment Trait 实现 ─────────────────────────────────────────────

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
        let (fiora, riven) = self.base.reset_app(app);
        vec![get_ego_obs_from_world(app.world(), fiora, riven, 0.0)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_ego_obs_from_world(
            world,
            self.base.fiora,
            self.base.riven,
            0.0,
        )]
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
        let riven_action = get_default_riven_combat_action(
            app.world(),
            self.base.riven,
            self.base.fiora,
        );

        self.base.increment_step();
        let res = step_fiora_v3_world(
            app,
            self.base.fiora,
            self.base.riven,
            fiora_action,
            riven_action,
            self.base.step_count,
            self.base.max_steps,
        );
        vec![res]
    }
}

// ── 自由函数 ─────────────────────────────────────────────────────────────────

pub fn get_default_riven_combat_action(
    world: &World,
    riven: Entity,
    fiora: Entity,
) -> FioraV3Action {
    let r_base = extract_champion_base(world, riven);
    let f_base = extract_champion_base(world, fiora);
    let dist = r_base.pos.distance(f_base.pos);
    let atk = extract_attack_state(world, riven);
    let skills = extract_skill_cds(world, riven);

    if atk.is_windup {
        return FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::NoOp);
    }

    let target_modifiers = extract_entity_modifiers(world, riven, 4);
    let primary_vital = target_modifiers
        .iter()
        .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);
    let (offset_x, offset_z) = if let Some(v) = primary_vital {
        if v.stack_count > 0.5 {
            (-v.param0 * 0.5, -v.param1 * 0.5)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    };

    if dist <= ATTACK_MASK_DISTANCE {
        if skills[1].ready {
            FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::CastW)
        } else if skills[0].ready {
            FioraV3Action::new(offset_x, offset_z, FioraV3DiscreteAction::CastQ)
        } else if !atk.is_cooldown {
            FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::Attack)
        } else if skills[2].ready {
            FioraV3Action::new(offset_x, offset_z, FioraV3DiscreteAction::CastE)
        } else if skills[3].ready {
            FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::CastR)
        } else {
            FioraV3Action::new(offset_x, offset_z, FioraV3DiscreteAction::Move)
        }
    } else if skills[2].ready {
        FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::CastE)
    } else if skills[0].ready {
        FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::CastQ)
    } else {
        FioraV3Action::new(0.0, 0.0, FioraV3DiscreteAction::Move)
    }
}

pub fn get_visible_minion_entities(
    world: &World,
    self_pos: Vec3,
    self_team: Team,
) -> (Vec<Entity>, Vec<lol_rl_protocol::ObsContext>) {
    let mut enemy_minions: Vec<(Entity, f32, Vec3, Team, f32, f32, Minion)> = Vec::new();
    let mut ally_minions: Vec<(Entity, f32, Vec3, Team, f32, f32, Minion)> = Vec::new();

    for entity_ref in world.iter_entities() {
        if let Some(minion) = entity_ref.get::<Minion>() {
            if let (Some(hp), Some(tf), Some(team)) = (
                entity_ref.get::<Health>(),
                entity_ref.get::<Transform>(),
                entity_ref.get::<Team>(),
            ) {
                if hp.value > 0.0 {
                    let m_pos = tf.translation;
                    let dist = self_pos.distance(m_pos);
                    let item = (entity_ref.id(), dist, m_pos, *team, hp.value, hp.max, *minion);
                    if *team != self_team {
                        enemy_minions.push(item);
                    } else {
                        ally_minions.push(item);
                    }
                }
            }
        }
    }

    enemy_minions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    ally_minions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // 优先填入敌方小兵（可攻击补刀目标），剩余槽位填入友方小兵
    let mut candidates = enemy_minions;
    candidates.extend(ally_minions);

    let mut entities = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS - 1);
    let mut slots = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS - 1);

    for (e, _dist, m_pos, team, hp_val, hp_max, m_type) in
        candidates.into_iter().take(FIORA_V3_MAX_VISIBLE_UNITS - 1)
    {
        let type_code = match m_type {
            Minion::Melee => 2.0,
            Minion::Ranged => 3.0,
            Minion::Siege => 4.0,
            Minion::Super => 5.0,
        };
        slots.push(
            lol_rl_protocol::ObsContext::new()
                .with_var("unit_type", type_code)
                .with_var("rel_pos[0]", m_pos.x - self_pos.x)
                .with_var("rel_pos[1]", m_pos.z - self_pos.z)
                .with_var(
                    "hp_pct",
                    if hp_max > 0.0 {
                        (hp_val / hp_max).clamp(0.0, 1.0)
                    } else {
                        0.0
                    },
                )
                .with_var("is_enemy", if team != self_team { 1.0 } else { 0.0 }),
        );
        entities.push(e);
    }

    (entities, slots)
}

pub fn extract_visible_units_from_world(
    world: &World,
    _self_entity: Entity,
    target_entity: Entity,
    self_pos: Vec3,
    self_team: Team,
    target_pos: Vec3,
    target_hp: f32,
    target_max_hp: f32,
) -> (Vec<lol_rl_protocol::ObsContext>, Vec<Option<Entity>>) {
    let mut slots = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);
    let mut entities = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);

    // Slot 0: 对手英雄
    slots.push(
        lol_rl_protocol::ObsContext::new()
            .with_var("unit_type", 1.0) // Champion
            .with_var("rel_pos[0]", target_pos.x - self_pos.x)
            .with_var("rel_pos[1]", target_pos.z - self_pos.z)
            .with_var(
                "hp_pct",
                if target_max_hp > 0.0 {
                    (target_hp / target_max_hp).clamp(0.0, 1.0)
                } else {
                    0.0
                },
            )
            .with_var("is_enemy", 1.0),
    );
    entities.push(Some(target_entity));

    // Slots 1..: 敌方小兵优先，其次友方小兵
    let (minion_entities, minion_slots) = get_visible_minion_entities(world, self_pos, self_team);
    slots.extend(minion_slots);
    entities.extend(minion_entities.into_iter().map(Some));

    while slots.len() < FIORA_V3_MAX_VISIBLE_UNITS {
        slots.push(lol_rl_protocol::ObsContext::new());
        entities.push(None);
    }

    (slots, entities)
}

pub fn get_ego_obs_from_world(
    world: &World,
    self_entity: Entity,
    target_entity: Entity,
    role_id: f32,
) -> FioraV3Obs {
    let self_base = extract_champion_base(world, self_entity);
    let target_base = extract_champion_base(world, target_entity);
    let dist = self_base.pos.distance(target_base.pos);
    let self_team = world
        .get::<Team>(self_entity)
        .copied()
        .unwrap_or(Team::Order);

    let atk = extract_attack_state(world, self_entity);
    let skills = extract_skill_cds(world, self_entity);
    let (flash_ready, flash_cd) = extract_flash_obs(world, self_entity);

    let (visible_units, visible_unit_entities) = extract_visible_units_from_world(
        world,
        self_entity,
        target_entity,
        self_base.pos,
        self_team,
        target_base.pos,
        target_base.hp,
        target_base.max_hp,
    );

    FioraV3Obs {
        role_id,
        self_pos: self_base.pos,
        self_hp: self_base.hp,
        self_max_hp: self_base.max_hp,
        target_pos: target_base.pos,
        target_hp: target_base.hp,
        target_max_hp: target_base.max_hp,
        distance: dist,
        attack_state: atk.state_code,
        attack_is_windup: atk.is_windup,
        attack_is_cooldown: atk.is_cooldown,
        attack_timer_remaining: atk.timer_remaining,
        q_ready: skills[0].ready,
        q_cd_remaining: skills[0].cd_remaining,
        w_ready: skills[1].ready,
        w_cd_remaining: skills[1].cd_remaining,
        e_ready: skills[2].ready,
        e_cd_remaining: skills[2].cd_remaining,
        r_ready: skills[3].ready,
        r_cd_remaining: skills[3].cd_remaining,
        flash_ready,
        flash_cd_remaining: flash_cd,
        self_modifiers: extract_entity_modifiers(world, self_entity, 4),
        target_modifiers: extract_entity_modifiers(world, target_entity, 4),
        visible_units,
        visible_unit_entities,
    }
}

pub fn dispatch_single_action(
    world: &mut World,
    self_entity: Entity,
    target_entity: Entity,
    action: FioraV3Action,
) {
    let tpos = world
        .get::<Transform>(target_entity)
        .map(|t| t.translation)
        .unwrap_or_default();
    let spos = world
        .get::<Transform>(self_entity)
        .map(|t| t.translation)
        .unwrap_or_default();

    let self_team = world.get::<Team>(self_entity).copied();

    // 解析目标：0 为敌方英雄，1.. 为小兵（敌方小兵优先，其次友方小兵）
    let chosen_target = if action.target_idx == 0 {
        target_entity
    } else {
        let (minion_entities, _) =
            get_visible_minion_entities(world, spos, self_team.unwrap_or(Team::Order));
        let minion_idx = (action.target_idx as usize) - 1;
        minion_entities
            .get(minion_idx)
            .copied()
            .unwrap_or(target_entity)
    };

    let chosen_target_pos = world
        .get::<Transform>(chosen_target)
        .map(|t| t.translation)
        .unwrap_or(tpos);

    let target_offset_pos = Vec3::new(
        chosen_target_pos.x + action.offset_x.clamp(-1.0, 1.0) * FIORA_V3_OFFSET_SCALE,
        chosen_target_pos.y,
        chosen_target_pos.z + action.offset_z.clamp(-1.0, 1.0) * FIORA_V3_OFFSET_SCALE,
    );

    let chosen_target_team = world.get::<Team>(chosen_target).copied();
    let is_target_enemy = match (self_team, chosen_target_team) {
        (Some(st), Some(tt)) => st != tt,
        _ => true,
    };

    // 友方目标防御性降级：若选中的是非敌方目标（友军/自身），普攻和技能自动降级为 Move
    let actual_discrete = if !is_target_enemy {
        match action.discrete {
            FioraV3DiscreteAction::Attack
            | FioraV3DiscreteAction::CastQ
            | FioraV3DiscreteAction::CastW
            | FioraV3DiscreteAction::CastE
            | FioraV3DiscreteAction::CastR => FioraV3DiscreteAction::Move,
            other => other,
        }
    } else {
        action.discrete
    };

    match actual_discrete {
        FioraV3DiscreteAction::NoOp => {}
        FioraV3DiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Move(Vec2::new(target_offset_pos.x, target_offset_pos.z)),
            });
        }
        FioraV3DiscreteAction::Attack => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Attack(chosen_target),
            });
        }
        FioraV3DiscreteAction::CastQ => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 0,
                    point: Vec2::new(target_offset_pos.x, target_offset_pos.z),
                },
            });
        }
        FioraV3DiscreteAction::CastW => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 1,
                    point: Vec2::new(spos.x, spos.z),
                },
            });
        }
        FioraV3DiscreteAction::CastE => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 2,
                    point: Vec2::new(target_offset_pos.x, target_offset_pos.z),
                },
            });
        }
        FioraV3DiscreteAction::CastR => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 3,
                    point: Vec2::new(chosen_target_pos.x, chosen_target_pos.z),
                },
            });
        }
        FioraV3DiscreteAction::CastFlash => {
            let offset_dir = Vec3::new(action.offset_x, 0.0, action.offset_z);
            let dir = if offset_dir.length_squared() > 1e-4 {
                offset_dir.normalize()
            } else {
                let to_target = chosen_target_pos - spos;
                if to_target.length_squared() > 1e-4 {
                    to_target.normalize()
                } else {
                    Vec3::X
                }
            };
            dispatch_flash(world, self_entity, dir, FLASH_DISTANCE);
        }
    }
}

pub fn step_fiora_v3_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    act_fiora: FioraV3Action,
    act_riven: FioraV3Action,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraV3Obs> {
    let prev_f_obs = get_ego_obs_from_world(app.world(), fiora, riven, 0.0);
    let prev_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    // 1. 识别对小兵的普通攻击行为
    let fiora_attacked_minion = act_fiora.discrete == FioraV3DiscreteAction::Attack
        && act_fiora.target_idx > 0
        && prev_f_obs.is_target_enemy(act_fiora.target_idx as usize);

    dispatch_single_action(app.world_mut(), fiora, riven, act_fiora);
    dispatch_single_action(app.world_mut(), riven, fiora, act_riven);
    unpause_virtual_time(app.world_mut());

    for _ in 0..10 {
        app.update();
    }

    let curr_f_obs = get_ego_obs_from_world(app.world(), fiora, riven, 0.0);
    let curr_f_hp = curr_f_obs.self_hp;
    let curr_r_hp = curr_f_obs.target_hp;
    let curr_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    let fiora_cs_diff = curr_f_cs.saturating_sub(prev_f_cs) as f32;

    // 普通攻击但是没产生补刀判定
    let fiora_wasted = if fiora_attacked_minion && fiora_cs_diff == 0.0 {
        1.0
    } else {
        0.0
    };

    let reward_cfg = app
        .world()
        .get_resource::<CurriculumRewardConfig>()
        .cloned()
        .unwrap_or_default();

    let f_vars = HashMap::from([
        ("self_cs".to_string(), fiora_cs_diff),
        ("self_attack_no_cs".to_string(), fiora_wasted),
        ("cs_reward_coef".to_string(), reward_cfg.cs_reward),
        ("penalty_coef".to_string(), reward_cfg.attack_no_cs_penalty),
        ("minion_hp_scale".to_string(), reward_cfg.minion_hp_scale),
    ]);

    let (r_fiora, f_breakdown_items) = FIORA_V3_SPEC
        .reward_formula
        .as_ref()
        .expect("FIORA_V3_SPEC 缺少 reward_formula DSL 规范")
        .compute(&f_vars);

    let f_breakdown = f_breakdown_items
        .into_iter()
        .map(|it| RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    let terminated = curr_f_hp <= 0.0 || curr_r_hp <= 0.0;
    let truncated = step_count >= max_steps;

    StepResult {
        obs: curr_f_obs,
        reward: r_fiora,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown: f_breakdown,
        reward_variables: f_vars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiora_v3_obs_schema_and_dim() {
        let schema = FioraV3Env::obs_schema().expect("FioraV3 obs schema");
        assert_eq!(schema.raw_dim(), FioraV3Env::state_dim());
        assert_eq!(FioraV3Obs::dim(), FioraV3Env::state_dim());
        let labels = schema.to_dim_labels();
        assert_eq!(labels.len(), FioraV3Env::state_dim());
    }

    #[test]
    fn test_fiora_v3_action_schema() {
        let schema = FioraV3Env::action_schema().expect("FioraV3 action schema");
        assert_eq!(schema.encoding_dim(), 4); // 2 continuous + 1 unit selection + 1 categorical
        assert_eq!(schema.num_branches(), 3);
        let labels = schema.to_encoding_labels();
        assert_eq!(labels.len(), 4);
    }

    #[test]
    fn test_fiora_v3_action_encoding_roundtrip() {
        let act = FioraV3Action::with_target(0.5, -0.5, 3, FioraV3DiscreteAction::Attack);
        let encoded = act.to_encoding();
        assert_eq!(encoded.len(), 4);
        assert_eq!(encoded[0], 0.5);
        assert_eq!(encoded[1], -0.5);
        assert_eq!(encoded[2], 3.0);
        assert_eq!(encoded[3], 2.0);

        let decoded = FioraV3Action::from_encoding(&encoded);
        assert_eq!(decoded.offset_x, 0.5);
        assert_eq!(decoded.offset_z, -0.5);
        assert_eq!(decoded.target_idx, 3);
        assert_eq!(decoded.discrete, FioraV3DiscreteAction::Attack);
    }
}

