use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::base::level::Level;
use lol_core::damage::Damage;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::missile::{Missile, MissileState};
use lol_core::skill::{
    CoolDown, Skill, SkillPoints, SkillRecastWindow, Skills, can_level_up_skill, is_skill_ready,
};
use lol_core::team::Team;
use lol_rl_protocol::{ObsFeaturePayload, ObsSchema};

use crate::obs_plugins::{extract_attack_state, extract_champion_base};

pub const FIORA_V3_MAX_VISIBLE_UNITS: usize = 12;
pub const FIORA_V3_MAX_VISIBLE_MISSILES: usize = 4;

pub static FIORA_V3_OBS_SCHEMA: std::sync::LazyLock<ObsSchema> = std::sync::LazyLock::new(|| {
    super::FIORA_V3_SPEC
        .obs_schema
        .clone()
        .expect("FIORA_V3_SPEC 缺少 obs_schema")
});

#[derive(Debug, Clone)]
pub struct FioraV3Obs {
    pub self_pos: Vec3,
    pub self_ad: f32,

    pub hero_level: u32,
    pub skill_points: u32,
    pub skill_levels: [usize; 4],
    pub skill_ready: [bool; 4],
    pub can_level_up: [bool; 4],

    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    pub visible_units: Vec<lol_rl_protocol::ObsContext>,
    pub visible_unit_entities: Vec<Option<Entity>>,
    pub visible_missiles: Vec<lol_rl_protocol::ObsContext>,
}

impl FioraV3Obs {
    pub fn to_context(&self) -> lol_rl_protocol::ObsContext {
        let mut ctx = lol_rl_protocol::ObsContext::new();
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

        ctx.set_var("self_ad", self.self_ad);

        ctx.set_var("hero_level", self.hero_level as f32);
        ctx.set_var("skill_points", self.skill_points as f32);
        ctx.set_var("q_level", self.skill_levels[0] as f32);
        ctx.set_var("w_level", self.skill_levels[1] as f32);
        ctx.set_var("e_level", self.skill_levels[2] as f32);
        ctx.set_var("r_level", self.skill_levels[3] as f32);

        ctx.set_var("q_ready", if self.skill_ready[0] { 1.0 } else { 0.0 });
        ctx.set_var("w_ready", if self.skill_ready[1] { 1.0 } else { 0.0 });
        ctx.set_var("e_ready", if self.skill_ready[2] { 1.0 } else { 0.0 });
        ctx.set_var("r_ready", if self.skill_ready[3] { 1.0 } else { 0.0 });

        ctx.set_var(
            "can_cast_any",
            if self.skill_ready.iter().any(|&r| r) {
                1.0
            } else {
                0.0
            },
        );
        ctx.set_var(
            "can_level_up_any",
            if self.can_level_up.iter().any(|&u| u) {
                1.0
            } else {
                0.0
            },
        );
        ctx.set_var(
            "can_level_up_q",
            if self.can_level_up[0] { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "can_level_up_w",
            if self.can_level_up[1] { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "can_level_up_e",
            if self.can_level_up[2] { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "can_level_up_r",
            if self.can_level_up[3] { 1.0 } else { 0.0 },
        );

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
            fiora_hp_pct: 1.0,
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

pub fn get_ego_obs_from_world(world: &World, self_entity: Entity) -> FioraV3Obs {
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

    let hero_level = world
        .get::<Level>(self_entity)
        .map(|l| l.value)
        .unwrap_or(1);
    let skill_points = world
        .get::<SkillPoints>(self_entity)
        .map(|sp| sp.0)
        .unwrap_or(0);

    let mut skill_levels = [0usize; 4];
    let mut skill_ready = [false; 4];
    let mut can_level_up = [false; 4];

    if let Some(skills) = world.get::<Skills>(self_entity) {
        for (i, s_entity) in skills.iter().enumerate().take(4) {
            let skill_comp = world.get::<Skill>(s_entity);
            let lvl = skill_comp.map(|s| s.level).unwrap_or(0);
            skill_levels[i] = lvl;

            let cd_comp = world.get::<CoolDown>(s_entity);
            let recast_comp = world.get::<SkillRecastWindow>(s_entity);
            skill_ready[i] = lvl > 0
                && cd_comp
                    .map(|cd| is_skill_ready(cd, recast_comp))
                    .unwrap_or(false);

            can_level_up[i] = can_level_up_skill(hero_level, i, lvl, skill_points);
        }
    }

    let (visible_units, visible_unit_entities) =
        extract_visible_units_from_world(world, self_base.pos, self_team);
    let visible_missiles = extract_visible_missiles_from_world(world, self_base.pos, self_team);

    FioraV3Obs {
        self_pos: self_base.pos,
        self_ad,
        hero_level,
        skill_points,
        skill_levels,
        skill_ready,
        can_level_up,
        attack_state: atk.state_code,
        attack_is_windup: atk.is_windup,
        attack_is_cooldown: atk.is_cooldown,
        attack_timer_remaining: atk.timer_remaining,
        visible_units,
        visible_unit_entities,
        visible_missiles,
    }
}
