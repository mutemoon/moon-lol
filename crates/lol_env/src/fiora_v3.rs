use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_core::base::stats::ChampionStats;
use lol_core::damage::Damage;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::missile::{Missile, MissileState};
use lol_core::team::Team;
use lol_rl_protocol::{ActionSchema, ActionSpace, ObsFeaturePayload, ObsSchema, RewardFormulaSpec};
use rand::Rng;
use rand::seq::SliceRandom;

use crate::base_env::{LolBaseEnv, fiora_champion_spec};
use crate::curriculum::CurriculumRewardConfig;
pub use crate::fiora_riven_common::{
    AttackEventTracker, setup_skill_levels_world, unpause_virtual_time,
};
use crate::modifier_obs::{ModifierNameId, ModifierSlotObs, extract_entity_modifiers};
use crate::obs_plugins::{extract_attack_state, extract_champion_base};
use crate::traits::{
    EnvConfig, EnvMeta, RenderMode, RewardBreakdownItem, RlEnvironment, StepResult,
    VisualEnvironment,
};

// ── 常量定义 ─────────────────────────────────────────────────────────────────

pub const FIORA_V3_OFFSET_SCALE: f32 = 100.0;
pub const FIORA_V3_MAX_VISIBLE_UNITS: usize = 20;
pub const FIORA_V3_MAX_VISIBLE_MISSILES: usize = 4;
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

// ── 初始化与状态重置 ─────────────────────────────────────────────────────────

pub fn setup_fiora_v3_health_world(world: &mut World, fiora: Entity) {
    if let Some(mut hp) = world.get_mut::<Health>(fiora) {
        hp.value = hp.max;
    }
    if let Some(mut stats) = world.get_mut::<ChampionStats>(fiora) {
        stats.kills = 0;
        stats.deaths = 0;
        stats.assists = 0;
        stats.minion_kills = 0;
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FioraV3DiscreteAction {
    NoOp = 0,
    Move = 1,
    Attack = 2,
}

impl FioraV3DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
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
            _ => Self::new(0.0, 0.0, FioraV3DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            FioraV3DiscreteAction::NoOp => 0,
            FioraV3DiscreteAction::Move => 1,
            FioraV3DiscreteAction::Attack => 2,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.discrete {
            FioraV3DiscreteAction::NoOp => "保持当前 (NoOp)",
            FioraV3DiscreteAction::Move => "移动",
            FioraV3DiscreteAction::Attack => "普通攻击",
        }
    }
}

// ── 观测数据结构 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FioraV3Obs {
    pub role_id: f32,

    pub self_pos: Vec3,
    pub self_hp: f32,
    pub self_max_hp: f32,
    pub self_ad: f32,

    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    pub self_modifiers: Vec<ModifierSlotObs>,

    pub visible_units: Vec<lol_rl_protocol::ObsContext>,
    pub visible_unit_entities: Vec<Option<Entity>>,
    pub visible_missiles: Vec<lol_rl_protocol::ObsContext>,
}

impl FioraV3Obs {
    pub fn to_context(&self) -> lol_rl_protocol::ObsContext {
        let mut ctx = lol_rl_protocol::ObsContext::new();
        ctx.set_var("role_id", self.role_id);
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

        ctx.set_var("self_hp", self.self_hp);
        ctx.set_var("self_max_hp", self.self_max_hp);
        ctx.set_var("self_ad", self.self_ad);

        let self_mods: Vec<_> = self.self_modifiers.iter().map(|m| m.to_context()).collect();
        ctx.set_repeated("self_modifiers", self_mods);
        ctx.set_repeated("visible_units", self.visible_units.clone());
        ctx.set_repeated("visible_missiles", self.visible_missiles.clone());

        ctx
    }

    pub fn to_vector(&self) -> Vec<f32> {
        FIORA_V3_OBS_SCHEMA.eval_to_vector(&self.to_context())
    }

    pub fn dim() -> usize {
        FIORA_V3_OBS_SCHEMA.raw_dim()
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        ObsFeaturePayload {
            fiora_hp_pct: if self.self_max_hp > 0.0 {
                self.self_hp / self.self_max_hp
            } else {
                1.0
            },
            tags: HashMap::from([
                ("role".to_string(), "剑姬 (Fiora)".to_string()),
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
                        "Self:{}",
                        self.self_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                    ),
                ),
                (
                    "visible_units".to_string(),
                    format!(
                        "{}",
                        self.visible_units
                            .iter()
                            .filter(|u| u.vars.get("unit_type").copied().unwrap_or(0.0) > 0.0)
                            .count()
                    ),
                ),
            ]),
            ..Default::default()
        }
    }

    /// 检查指定目标槽位是否为有效敌方单位（小兵有效且 is_enemy > 0.5）
    pub fn is_target_enemy(&self, target_idx: usize) -> bool {
        if let Some(unit) = self.visible_units.get(target_idx) {
            let unit_type = unit.vars.get("unit_type").copied().unwrap_or(0.0);
            let is_enemy = unit.vars.get("is_enemy").copied().unwrap_or(0.0);
            unit_type > 0.0 && is_enemy > 0.5
        } else {
            false
        }
    }
}

// ── 环境主体 ─────────────────────────────────────────────────────────────────

/// 对 Fiora V3 环境中的小兵血量进行随机化分配：
/// 保证每波小兵中既有一击必杀（20~55 HP）的残血小兵，也有中等血量和近乎满血的小兵，
/// 促使智能体学会识别血量并精准选取残血目标进行补刀。
pub fn randomize_fiora_v3_minion_health(world: &mut World) {
    let mut rng = rand::rng();

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

    for mut minion_list in [order_minions, chaos_minions] {
        if minion_list.is_empty() {
            continue;
        }
        // 随机打乱顺序，使得一击必杀和残血小兵的位置在每局不同
        minion_list.shuffle(&mut rng);
        let n = minion_list.len();

        for (i, entity) in minion_list.into_iter().enumerate() {
            if let Some(mut health) = world.get_mut::<Health>(entity) {
                // 分桶分配血量：
                // 前 ~30% (至少1个)：一击必杀残血 (20 ~ 55 HP，Fiora 68 AD 一刀必杀)
                // 紧接着 ~35%：中等血量 (120 ~ 240 HP，需多次攻击)
                // 剩余 ~35%：高血量/满血 (320 ~ max_hp)
                let target_hp: f32 = if i < (n.max(3) / 3).max(1) {
                    rng.random_range(20.0f32..=55.0f32).min(health.max)
                } else if i < (2 * n.max(3) / 3).max(2) {
                    rng.random_range(120.0f32..=240.0f32).min(health.max)
                } else {
                    rng.random_range((health.max * 0.7)..=health.max)
                };

                health.value = target_hp.clamp(1.0, health.max);
            }
        }
    }
}

/// 统一的单人世界初始化与重置逻辑（满血重置与小兵随机血量设置）
pub fn setup_fiora_v3_env_world(champions: &[Entity], world: &mut World) {
    if let Some(&fiora) = champions.first() {
        setup_fiora_v3_health_world(world, fiora);
    }
    randomize_fiora_v3_minion_health(world);
}

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
        FIORA_V3_SPEC.reward_formula.clone()
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

// ── 小兵与单位观测提取 ───────────────────────────────────────────────────────

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
                    let item = (
                        entity_ref.id(),
                        dist,
                        m_pos,
                        *team,
                        hp.value,
                        hp.max,
                        *minion,
                    );
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

    let mut entities = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);
    let mut slots = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);

    for (e, _dist, m_pos, team, hp_val, hp_max, m_type) in
        candidates.into_iter().take(FIORA_V3_MAX_VISIBLE_UNITS)
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
                .with_var("hp_norm", hp_val / 1000.0)
                .with_var("is_enemy", if team != self_team { 1.0 } else { 0.0 }),
        );
        entities.push(e);
    }

    (entities, slots)
}

pub fn extract_visible_units_from_world(
    world: &World,
    self_pos: Vec3,
    self_team: Team,
) -> (Vec<lol_rl_protocol::ObsContext>, Vec<Option<Entity>>) {
    let mut slots = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);
    let mut entities = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_UNITS);

    let (minion_entities, minion_slots) = get_visible_minion_entities(world, self_pos, self_team);
    slots.extend(minion_slots);
    entities.extend(minion_entities.into_iter().map(Some));

    while slots.len() < FIORA_V3_MAX_VISIBLE_UNITS {
        slots.push(lol_rl_protocol::ObsContext::new());
        entities.push(None);
    }

    (slots, entities)
}

pub fn extract_visible_missiles_from_world(
    world: &World,
    self_pos: Vec3,
    self_team: Team,
) -> Vec<lol_rl_protocol::ObsContext> {
    let mut candidate_missiles: Vec<(f32, Vec3, Team)> = Vec::new();

    for entity_ref in world.iter_entities() {
        if let (Some(_missile), Some(state), Some(tf)) = (
            entity_ref.get::<Missile>(),
            entity_ref.get::<MissileState>(),
            entity_ref.get::<Transform>(),
        ) {
            // 筛选小兵发射的飞弹（或小兵普通攻击弹道）
            let is_minion_missile = world.get::<Minion>(state.source).is_some();
            if is_minion_missile {
                let team = entity_ref.get::<Team>().copied().unwrap_or(Team::Order);
                let m_pos = tf.translation;
                let dist = self_pos.distance(m_pos);
                candidate_missiles.push((dist, m_pos, team));
            }
        }
    }

    candidate_missiles.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut slots = Vec::with_capacity(FIORA_V3_MAX_VISIBLE_MISSILES);
    for (_dist, m_pos, team) in candidate_missiles
        .into_iter()
        .take(FIORA_V3_MAX_VISIBLE_MISSILES)
    {
        slots.push(
            lol_rl_protocol::ObsContext::new()
                .with_var("rel_pos[0]", m_pos.x - self_pos.x)
                .with_var("rel_pos[1]", m_pos.z - self_pos.z)
                .with_var("is_enemy", if team != self_team { 1.0 } else { 0.0 })
                .with_var("is_active", 1.0),
        );
    }

    while slots.len() < FIORA_V3_MAX_VISIBLE_MISSILES {
        slots.push(
            lol_rl_protocol::ObsContext::new()
                .with_var("rel_pos[0]", 0.0)
                .with_var("rel_pos[1]", 0.0)
                .with_var("is_enemy", 0.0)
                .with_var("is_active", 0.0),
        );
    }

    slots
}

pub fn get_ego_obs_from_world(world: &World, self_entity: Entity, role_id: f32) -> FioraV3Obs {
    let self_base = extract_champion_base(world, self_entity);
    let self_team = world
        .get::<Team>(self_entity)
        .copied()
        .unwrap_or(Team::Order);

    let atk = extract_attack_state(world, self_entity);
    let self_ad = world
        .get::<Damage>(self_entity)
        .map(|d| d.0)
        .unwrap_or(68.0);

    let (visible_units, visible_unit_entities) =
        extract_visible_units_from_world(world, self_base.pos, self_team);
    let visible_missiles = extract_visible_missiles_from_world(world, self_base.pos, self_team);

    FioraV3Obs {
        role_id,
        self_pos: self_base.pos,
        self_hp: self_base.hp,
        self_max_hp: self_base.max_hp,
        self_ad,
        attack_state: atk.state_code,
        attack_is_windup: atk.is_windup,
        attack_is_cooldown: atk.is_cooldown,
        attack_timer_remaining: atk.timer_remaining,
        self_modifiers: extract_entity_modifiers(world, self_entity, 4),
        visible_units,
        visible_unit_entities,
        visible_missiles,
    }
}

pub fn dispatch_single_action(
    world: &mut World,
    self_entity: Entity,
    action: FioraV3Action,
    visible_unit_entities: &[Option<Entity>],
) {
    let spos = world
        .get::<Transform>(self_entity)
        .map(|t| t.translation)
        .unwrap_or_default();

    let self_team = world
        .get::<Team>(self_entity)
        .copied()
        .unwrap_or(Team::Order);

    let chosen_target = visible_unit_entities
        .get(action.target_idx as usize)
        .copied()
        .flatten();

    let chosen_target_pos = chosen_target
        .and_then(|e| world.get::<Transform>(e).map(|t| t.translation))
        .unwrap_or(spos);

    let target_offset_pos = Vec3::new(
        chosen_target_pos.x + action.offset_x.clamp(-1.0, 1.0) * FIORA_V3_OFFSET_SCALE,
        chosen_target_pos.y,
        chosen_target_pos.z + action.offset_z.clamp(-1.0, 1.0) * FIORA_V3_OFFSET_SCALE,
    );

    let is_target_enemy =
        chosen_target.is_some_and(|e| world.get::<Team>(e).is_some_and(|t| *t != self_team));

    // 友方目标或无效目标防御性降级：普攻必须有敌方目标，否则降级为 Move
    let actual_discrete = match action.discrete {
        FioraV3DiscreteAction::Attack if !is_target_enemy => FioraV3DiscreteAction::Move,
        other => other,
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
            if let Some(target) = chosen_target {
                world.trigger(CommandAction {
                    entity: self_entity,
                    action: Action::Attack(target),
                });
            }
        }
    }
}

pub fn step_fiora_v3_world(
    app: &mut App,
    fiora: Entity,
    act_fiora: FioraV3Action,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraV3Obs> {
    let prev_f_obs = get_ego_obs_from_world(app.world(), fiora, 0.0);
    let prev_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    // 1. 识别对有效敌方小兵的普通攻击行为
    let fiora_attacked_minion = act_fiora.discrete == FioraV3DiscreteAction::Attack
        && prev_f_obs.is_target_enemy(act_fiora.target_idx as usize);

    dispatch_single_action(
        app.world_mut(),
        fiora,
        act_fiora,
        &prev_f_obs.visible_unit_entities,
    );
    unpause_virtual_time(app.world_mut());

    for _ in 0..10 {
        app.update();
    }

    let curr_f_obs = get_ego_obs_from_world(app.world(), fiora, 0.0);
    let curr_f_hp = curr_f_obs.self_hp;
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

    let terminated = curr_f_hp <= 0.0;
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
