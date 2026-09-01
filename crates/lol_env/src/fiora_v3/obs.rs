use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::damage::Damage;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::missile::{Missile, MissileState};
use lol_core::team::Team;
use lol_rl_protocol::{ObsFeaturePayload, ObsSchema};

use crate::modifier_obs::{ModifierNameId, ModifierSlotObs, extract_entity_modifiers};
use crate::obs_plugins::{extract_attack_state, extract_champion_base};

pub const FIORA_V3_MAX_VISIBLE_UNITS: usize = 12;
pub const FIORA_V3_MAX_VISIBLE_MISSILES: usize = 4;
pub const FIORA_V3_OBS_DISTANCE_SCALE: f32 = 100.0;

pub static FIORA_V3_OBS_SCHEMA: std::sync::LazyLock<ObsSchema> = std::sync::LazyLock::new(|| {
    super::FIORA_V3_SPEC
        .obs_schema
        .clone()
        .expect("FIORA_V3_SPEC 缺少 obs_schema")
});

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
