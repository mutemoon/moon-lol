use std::collections::HashMap;

use bevy::prelude::*;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_base_render::camera::Focus;
use lol_champions::fiora::Fiora;
use lol_champions::fiora::passive::Vital;
use lol_champions::riven::Riven;
use lol_core::base::direction::Direction;
use lol_core::character::{CharacterReady, SpawnTransform};
use lol_core::damage::{Armor, DamageType, EventDamageCreate};
use lol_core::game::WaitCharacterReady;
use lol_core::life::Health;
use lol_core::movement::Movement;
use lol_core::skill::{CoolDown, Skill, SkillRecastWindow, Skills, is_skill_ready};
use lol_core::team::Team;
use lol_render::controller::SelfPlayer;

use crate::base_env::{LolBaseEnv, LolBaseEnvBuilder};
use crate::reward::{FioraRewardContext, FioraVsRivenRewardModel, RewardModel};
use crate::traits::RewardBreakdownItem;

/// 攻击类动作的掩码距离阈值：超过该距离不允许攻击（单一事实来源）。
pub const ATTACK_MASK_DISTANCE: f32 = 220.0;

/// obs 向量中「相对距离归一化列」的下标，与 [`FioraVsRivenObs::to_vector`] 的布局一致。
pub const OBS_DISTANCE_IDX: usize = 8;
/// obs 向量中距离的归一化缩放：`to_vector` 写入 `distance / OBS_DISTANCE_SCALE`。
pub const OBS_DISTANCE_SCALE: f32 = 100.0;

/// 英雄初始技能等级配置（Q, W, E, R）
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChampionInitialSkillLevels(pub [usize; 4]);

impl Default for ChampionInitialSkillLevels {
    fn default() -> Self {
        Self([3, 1, 1, 1])
    }
}

/// 通用环境别名
pub type FioraRivenBaseEnv = LolBaseEnv;
pub type FioraRivenEnvBuilder = LolBaseEnvBuilder;

// ── 观测 ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FioraVsRivenObs {
    pub fiora_pos: Vec3,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_pos: Vec3,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
    pub distance: f32,
    pub q_ready: bool,
    pub w_ready: bool,
    pub e_ready: bool,
    pub r_ready: bool,
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,
}

pub static FIORA_COMMON_OBS_SCHEMA: std::sync::LazyLock<lol_rl_protocol::ObsSchema> =
    std::sync::LazyLock::new(|| {
        lol_rl_protocol::SPEC_FIORA_V0
            .obs_schema
            .clone()
            .expect("SPEC_FIORA_V0 缺少 obs_schema")
    });

impl FioraVsRivenObs {
    pub fn to_context(&self) -> lol_rl_protocol::ObsContext {
        lol_rl_protocol::ObsContext::new()
            .with_var("vital_dir_x", self.vital_dir_x)
            .with_var("vital_dir_neg_x", self.vital_dir_neg_x)
            .with_var("vital_dir_z", self.vital_dir_z)
            .with_var("vital_dir_neg_z", self.vital_dir_neg_z)
            .with_var("has_vital", if self.has_vital { 1.0 } else { 0.0 })
            .with_var(
                "vital_is_active",
                if self.vital_is_active { 1.0 } else { 0.0 },
            )
            .with_var("fiora_x", self.fiora_pos.x)
            .with_var("fiora_z", self.fiora_pos.z)
            .with_var("riven_x", self.riven_pos.x)
            .with_var("riven_z", self.riven_pos.z)
            .with_var("distance", self.distance)
    }

    /// 转换为强化学习策略网络输入向量（由 ObsSchema AST 自动求值）。
    pub fn to_vector(&self) -> Vec<f32> {
        FIORA_COMMON_OBS_SCHEMA.eval_to_vector(&self.to_context())
    }

    pub fn dim() -> usize {
        FIORA_COMMON_OBS_SCHEMA.raw_dim()
    }

    pub fn to_payload(&self) -> lol_rl_protocol::ObsFeaturePayload {
        let vital_dir = if self.vital_dir_x > 0.5 {
            "+X (东)".to_string()
        } else if self.vital_dir_neg_x > 0.5 {
            "-X (西)".to_string()
        } else if self.vital_dir_z > 0.5 {
            "+Z (北)".to_string()
        } else if self.vital_dir_neg_z > 0.5 {
            "-Z (南)".to_string()
        } else {
            "无".to_string()
        };

        lol_rl_protocol::ObsFeaturePayload {
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
            w_ready: self.w_ready,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital: self.has_vital,
            vital_is_active: self.vital_is_active,
            vital_direction: vital_dir,
            ..Default::default()
        }
    }
}

// ── 事件追踪 ────────────────────────────────────────────────────────────────

/// 普攻事件追踪：攻击命中（EventAttackEnd）与攻击就绪（EventAttackReady）。
#[derive(Resource, Default, Debug, Clone)]
pub struct AttackEventTracker {
    pub attack_hit: bool,
    pub attack_ready: bool,
}

pub fn on_attack_end(
    _trigger: On<lol_core::attack::EventAttackEnd>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_hit = true;
}

pub fn on_attack_ready(
    _trigger: On<lol_core::attack::EventAttackReady>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_ready = true;
}

/// 环境中的 Fiora / Riven 实体，供观察者过滤事件来源。
#[derive(Resource)]
pub struct FioraRivenEntities {
    pub fiora: Entity,
    pub riven: Entity,
}

/// 真实破绽击破信号：菲奥娜被动击破要害会对目标造成一次真实伤害
#[derive(Resource, Default, Debug, Clone)]
pub struct VitalBreakTracker {
    pub hit: bool,
}

pub fn on_vital_break_damage(
    trigger: On<EventDamageCreate>,
    entities: Option<Res<FioraRivenEntities>>,
    q_fiora: Query<&Fiora>,
    mut tracker: ResMut<VitalBreakTracker>,
) {
    if trigger.damage_type == DamageType::True {
        let is_fiora = if let Some(entities) = entities {
            trigger.source == entities.fiora
        } else {
            q_fiora.contains(trigger.source)
        };
        if is_fiora {
            tracker.hit = true;
        }
    }
}

/// 注册两个环境共用的资源与观察者（在 `App::finish()` 之前调用）。
pub fn add_common_observers(app: &mut App) {
    app.init_resource::<AttackEventTracker>();
    app.init_resource::<VitalBreakTracker>();
    app.add_observer(on_attack_end);
    app.add_observer(on_attack_ready);
    app.add_observer(on_vital_break_damage);
    app.add_observer(on_character_ready_set_skill_levels);
}

/// 角色配置写入完成后同步设置 Q/W/E/R 技能等级。
pub fn on_character_ready_set_skill_levels(
    trigger: On<Add, CharacterReady>,
    q_skills: Query<&Skills>,
    mut q_skill: Query<&mut Skill>,
    levels_res: Option<Res<ChampionInitialSkillLevels>>,
) {
    let entity = trigger.entity;
    let Ok(skills) = q_skills.get(entity) else {
        return;
    };
    let skill_entities = skills.to_vec();
    if skill_entities.len() < 4 {
        return;
    }
    let levels = levels_res.map(|r| r.0).unwrap_or([3, 1, 1, 1]);
    for (idx, level) in levels.into_iter().enumerate() {
        if let Ok(mut skill) = q_skill.get_mut(skill_entities[idx]) {
            skill.level = level;
        }
    }
}

// ── 世界读写 ────────────────────────────────────────────────────────────────

/// Extract observation from the Bevy ECS world.
pub fn get_obs_from_world(world: &World, fiora: Entity, riven: Entity) -> FioraVsRivenObs {
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();
    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let dist = fpos.distance(rpos);

    let fhp = world.get::<Health>(fiora);
    let rhp = world.get::<Health>(riven);

    let vital = world.get::<Vital>(riven);
    let (has_vital, vital_is_active, v_x, v_neg_x, v_z, v_neg_z) = match vital {
        Some(v) => {
            let (vx, vnx, vz, vnz) = match v.direction {
                Direction::X => (1.0, 0.0, 0.0, 0.0),
                Direction::NegX => (0.0, 1.0, 0.0, 0.0),
                Direction::Z => (0.0, 0.0, 1.0, 0.0),
                Direction::NegZ => (0.0, 0.0, 0.0, 1.0),
            };
            (true, v.is_active(), vx, vnx, vz, vnz)
        }
        None => (false, false, 0.0, 0.0, 0.0, 0.0),
    };

    let (q_ready, w_ready, e_ready, r_ready) = {
        let mut ready = (true, true, true, true);
        if let Some(skills) = world.get::<Skills>(fiora) {
            let skill_entities = skills.to_vec();
            let check_ready = |idx: usize| -> bool {
                if idx < skill_entities.len() {
                    let s_entity = skill_entities[idx];
                    let cd = world.get::<CoolDown>(s_entity);
                    let recast = world.get::<SkillRecastWindow>(s_entity);
                    match cd {
                        Some(c) => is_skill_ready(c, recast),
                        None => true,
                    }
                } else {
                    true
                }
            };
            ready = (
                check_ready(0),
                check_ready(1),
                check_ready(2),
                check_ready(3),
            );
        }
        ready
    };

    FioraVsRivenObs {
        fiora_pos: fpos,
        fiora_hp: fhp.map(|h| h.value).unwrap_or(0.0),
        fiora_max_hp: fhp.map(|h| h.max).unwrap_or(500.0),
        riven_pos: rpos,
        riven_hp: rhp.map(|h| h.value).unwrap_or(0.0),
        riven_max_hp: rhp.map(|h| h.max).unwrap_or(500.0),
        distance: dist,
        q_ready,
        w_ready,
        e_ready,
        r_ready,
        has_vital,
        vital_is_active,
        vital_dir_x: v_x,
        vital_dir_neg_x: v_neg_x,
        vital_dir_z: v_z,
        vital_dir_neg_z: v_neg_z,
    }
}

/// 在世界中重新生成 Fiora 和 Riven 实体
pub fn spawn_champions_world(
    world: &mut World,
    fiora_config_handle: Handle<DynamicWorld>,
    riven_config_handle: Handle<DynamicWorld>,
    fiora_skin_handle: Option<Handle<DynamicWorld>>,
    riven_skin_handle: Option<Handle<DynamicWorld>>,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    render: bool,
) -> (Entity, Entity) {
    let mut fiora_builder = world.spawn((
        Fiora::default(),
        Transform::from_translation(initial_fiora_pos),
        SpawnTransform(Transform::from_translation(initial_fiora_pos)),
        WaitCharacterReady,
        Team::Order,
        ConfigCharacterRecord {
            character_record: fiora_config_handle,
        },
        Health::new(500.0),
        Armor(35.0),
        Movement { speed: 345.0 },
    ));

    if render {
        if let Some(skin) = fiora_skin_handle {
            fiora_builder.insert((SelfPlayer, Focus, ConfigSkin { skin }));
        }
    }

    let fiora = fiora_builder.id();

    let mut riven_builder = world.spawn((
        Riven::default(),
        Transform::from_translation(initial_riven_pos),
        SpawnTransform(Transform::from_translation(initial_riven_pos)),
        WaitCharacterReady,
        Team::Chaos,
        ConfigCharacterRecord {
            character_record: riven_config_handle,
        },
        Health::new(500.0),
        Armor(33.0),
        Movement { speed: 340.0 },
    ));

    if render {
        if let Some(skin) = riven_skin_handle {
            riven_builder.insert(ConfigSkin { skin });
        }
    }

    let riven = riven_builder.id();

    (fiora, riven)
}

/// Set skill levels for Fiora and Riven in the Bevy ECS world.
pub fn setup_skill_levels_world(world: &mut World, fiora: Entity, riven: Entity) {
    let levels = world
        .get_resource::<ChampionInitialSkillLevels>()
        .map(|r| r.0)
        .unwrap_or([3, 1, 1, 1]);
    setup_custom_skill_levels_world(world, fiora, riven, levels);
}

/// 使用指定技能等级数组设置 Fiora 与 Riven 的技能等级
pub fn setup_custom_skill_levels_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    levels: [usize; 4],
) {
    for champion in [fiora, riven] {
        if let Some(skills) = world.get::<Skills>(champion) {
            let skill_entities = skills.to_vec();
            for (idx, &level) in levels.iter().enumerate() {
                if idx < skill_entities.len() {
                    if let Some(mut s) = world.get_mut::<Skill>(skill_entities[idx]) {
                        s.level = level;
                    }
                }
            }
        }
    }
}

/// Helper function to check if a 3D position is aligned with the vital's direction quadrant relative to target.
pub fn is_position_aligned_with_vital(fpos: Vec3, rpos: Vec3, obs: &FioraVsRivenObs) -> bool {
    let delta_x = fpos.x - rpos.x;
    let delta_z = fpos.z - rpos.z;
    let abs_delta_x = delta_x.abs();
    let abs_delta_z = delta_z.abs();

    if obs.vital_dir_x > 0.5 {
        delta_x > 0.0 && abs_delta_x > abs_delta_z
    } else if obs.vital_dir_neg_x > 0.5 {
        delta_x < 0.0 && abs_delta_x > abs_delta_z
    } else if obs.vital_dir_z > 0.5 {
        delta_z > 0.0 && abs_delta_z > abs_delta_x
    } else if obs.vital_dir_neg_z > 0.5 {
        delta_z < 0.0 && abs_delta_z > abs_delta_x
    } else {
        false
    }
}

/// Compute step reward and its breakdown items using the structured RewardModel.
pub fn compute_step_reward(
    prev_riven_hp: f32,
    curr_riven_hp: f32,
    prev_fpos: Vec3,
    curr_fpos: Vec3,
    riven_pos: Vec3,
    is_attack: bool,
    is_vital_break: bool,
    prev_obs: &FioraVsRivenObs,
    elapsed_secs: f32,
) -> (f32, Vec<RewardBreakdownItem>, HashMap<String, f32>) {
    let prev_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(prev_fpos, riven_pos, prev_obs);
    let curr_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(curr_fpos, riven_pos, prev_obs);

    let ctx = FioraRewardContext {
        prev_aligned,
        curr_aligned,
        is_vital_break,
        is_attack,
        prev_riven_hp,
        curr_riven_hp,
        elapsed_secs,
    };

    let model = FioraVsRivenRewardModel;
    let (reward, items, vars) = model.evaluate(&ctx);

    let breakdown = items
        .into_iter()
        .map(|it| RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    (reward, breakdown, vars)
}

/// Helper functions for controlling Virtual time during visual stepping.
pub fn pause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.pause();
    }
}

pub fn unpause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.unpause();
    }
}
