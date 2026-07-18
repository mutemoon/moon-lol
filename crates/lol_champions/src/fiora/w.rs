//! Fiora W（斗转刺 / Riposte）。
//!
//! 核心机制：0.75s 招架期内免疫所有非真实伤害（`BuffDamageReduction{1.0, None}`）
//! 与控制（`ImmuneToCC`，硬控被招架后记为 `parried_hard_cc`）。招架结束时向前方
//! 矩形刺出，对首个敌方英雄造成魔法伤害；若招架期间被硬控命中，则改为眩晕，
//! 否则施加减速。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, is_in_shape};
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::buffs::cc_debuffs::{ControlTag, DebuffSlow, DebuffStun, ImmuneToCC};
use lol_core::buffs::common_buffs::BuffCastBlock;
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Death;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};
use lol_core::team::Team;

use crate::fiora::Fiora;

/// 招架持续时间（ron ParryDuration = 0.75s）。
pub const FIORA_W_PARRY_DURATION: f32 = 0.75;
/// 反刺射程（矩形长度）。
pub const FIORA_W_THRUST_RANGE: f32 = 750.0;
/// 反刺线宽（ron lineWidth = 95）。
pub const FIORA_W_THRUST_WIDTH: f32 = 95.0;
/// 反刺减速持续时间。
pub const FIORA_W_SLOW_DURATION: f32 = 2.0;

/// Fiora W 招架追踪 buff：挂在菲奥娜自身，记录招架计时与反刺参数。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "FioraW" })]
pub struct BuffFioraW {
    pub parry_timer: Timer,
    pub parried_hard_cc: bool,
    pub cast_point: Vec2,
    pub stab_damage: f32,
    pub slow_percent: f32,
    pub cc_duration: f32,
}

impl BuffFioraW {
    pub fn new(cast_point: Vec2, stab_damage: f32, slow_percent: f32, cc_duration: f32) -> Self {
        Self {
            parry_timer: Timer::from_seconds(FIORA_W_PARRY_DURATION, TimerMode::Once),
            parried_hard_cc: false,
            cast_point,
            stab_damage,
            slow_percent,
            cc_duration,
        }
    }
}

pub fn on_fiora_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let spell = res_spells.get(&skill.spell);
    let stab_damage = spell
        .and_then(|s| get_skill_data_value(s, "BaseDamage", skill.level))
        .unwrap_or(70.0);
    let cc_duration = spell
        .and_then(|s| get_skill_data_value(s, "CCDuration", skill.level))
        .unwrap_or(FIORA_W_SLOW_DURATION);
    let slow_percent = spell
        .and_then(|s| get_skill_data_value(s, "MSSlowPercent", skill.level))
        .map(|v| v.abs())
        .unwrap_or(0.5);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2_in".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    commands.entity(entity).insert(ImmuneToCC);

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCastBlock::new(FIORA_W_PARRY_DURATION));

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDamageReduction::new(1.0, None));

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraW::new(
            trigger.point,
            stab_damage,
            slow_percent,
            cc_duration,
        ));
}

/// 招架计时：到点触发反刺并回收所有招架期状态。
pub fn update_fiora_w(
    mut commands: Commands,
    mut q_buff: Query<(Entity, &BuffOf, &mut BuffFioraW)>,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_reduction: Query<&BuffDamageReduction>,
    q_cast_block: Query<&BuffCastBlock>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_targets: Query<(Entity, &Transform, &Team), (With<Champion>, Without<Death>)>,
    time: Res<Time<Fixed>>,
) {
    for (buff_entity, buff_of, mut buff) in q_buff.iter_mut() {
        buff.parry_timer.tick(time.delta());
        if !buff.parry_timer.is_finished() {
            continue;
        }

        let caster = buff_of.0;

        counter_thrust(
            &mut commands,
            caster,
            &buff,
            &q_transform,
            &q_team,
            &q_targets,
        );

        commands.entity(caster).remove::<ImmuneToCC>();
        for (e, bo) in q_buffof.iter() {
            if bo.0 != caster {
                continue;
            }
            if q_reduction.get(e).is_ok() || q_cast_block.get(e).is_ok() {
                commands.entity(e).despawn();
            }
        }
        commands.entity(buff_entity).despawn();
    }
}

/// 反刺：从施法者朝 `cast_point` 方向的矩形内选取最近的敌方英雄。
fn counter_thrust(
    commands: &mut Commands,
    caster: Entity,
    buff: &BuffFioraW,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_targets: &Query<(Entity, &Transform, &Team), (With<Champion>, Without<Death>)>,
) {
    let Ok(caster_tf) = q_transform.get(caster) else {
        return;
    };
    let Ok(caster_team) = q_team.get(caster) else {
        return;
    };
    let caster_pos = caster_tf.translation;

    let mut dir = buff.cast_point - caster_pos.xz();
    if dir.length_squared() < 1e-6 {
        dir = Vec2::new(1.0, 0.0);
    }
    let dir = dir.normalize_or_zero();

    let shape = DamageShape::Rectangle {
        width: FIORA_W_THRUST_WIDTH,
        length: FIORA_W_THRUST_RANGE,
        start_distance: 0.0,
    };

    let mut hit: Option<(Entity, f32)> = None;
    for (target, t_tf, t_team) in q_targets.iter() {
        if t_team == caster_team {
            continue;
        }
        if !is_in_shape(t_tf.translation, caster_pos, dir, &shape) {
            continue;
        }
        let proj = (t_tf.translation.xz() - caster_pos.xz()).dot(dir);
        if hit.map_or(true, |(_, d)| proj < d) {
            hit = Some((target, proj));
        }
    }

    let Some((target, _)) = hit else {
        return;
    };

    commands.entity(target).trigger(|e| CommandDamageCreate {
        entity: e,
        source: caster,
        damage_type: DamageType::Magic,
        amount: buff.stab_damage,
        tag: None,
    });

    if buff.parried_hard_cc {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffStun::new(buff.cc_duration));
    } else {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffSlow::new(buff.slow_percent, FIORA_W_SLOW_DURATION));
    }
}

/// 招架硬控侦测：任一可净化 CC buff 生成时，若其目标正处菲奥娜 W 招架期，
/// 标记 `parried_hard_cc = true`（反刺将改为眩晕）。
pub fn on_fiora_w_parried_cc(
    trigger: On<Add, ControlTag>,
    q_buffof: Query<(Entity, &BuffOf)>,
    mut q_fiora_w: Query<(&BuffOf, &mut BuffFioraW)>,
) {
    let buff_entity = trigger.entity;
    let Ok((_, buffof)) = q_buffof.get(buff_entity) else {
        return;
    };
    let char = buffof.0;

    for (bo, mut fiora_w) in q_fiora_w.iter_mut() {
        if bo.0 == char && !fiora_w.parry_timer.is_finished() {
            fiora_w.parried_hard_cc = true;
            return;
        }
    }
}
