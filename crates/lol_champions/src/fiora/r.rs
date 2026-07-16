//! Fiora R - 无双挑战 (Grand Challenge)
//!
//! 对一名敌方英雄施放，标记四个方向要害，持续 8 秒。
//! 击破要害造成最大生命值真实伤害。四要害全破后触发治疗光环。

use bevy::prelude::*;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::base::direction::{Direction, is_in_direction};
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{CommandDamageCreate, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::{Death, Health};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};
use lol_core::team::Team;

use crate::fiora::Fiora;

/// 要害过期前红色闪烁预警时长（秒），游戏手感常数，不来自 RON。
const VITAL_R_TIMEOUT: f32 = 1.5;
/// R 标记生效前延迟（秒），游戏手感常数，不来自 RON。
const FIORA_R_ACTIVE_DURATION: f32 = 0.5;

#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "FioraR" })]
pub struct BuffFioraR {
    pub vitals: Vec<Direction>,
    pub level: usize,
    pub active_timer: Timer,
    pub remove_timer: Timer,
    pub timeout_red_triggered: bool,
    pub vital_pct: f32,
    pub heal_per_second: f32,
    pub heal_duration: f32,
    pub heal_radius: f32,
}

impl BuffFioraR {
    pub fn is_active(&self) -> bool {
        self.active_timer.is_finished()
    }
}

impl Default for BuffFioraR {
    fn default() -> Self {
        Self::new(1, 8.0, 0.5, 0.03, 50.0, 5.0, 550.0)
    }
}

impl BuffFioraR {
    pub fn new(
        level: usize,
        duration: f32,
        active_duration: f32,
        vital_pct: f32,
        heal_per_second: f32,
        heal_duration: f32,
        heal_radius: f32,
    ) -> Self {
        Self {
            vitals: vec![
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ],
            level,
            active_timer: Timer::from_seconds(active_duration, TimerMode::Once),
            remove_timer: Timer::from_seconds(duration, TimerMode::Once),
            timeout_red_triggered: false,
            vital_pct,
            heal_per_second,
            heal_duration,
            heal_radius,
        }
    }
}

#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "FioraRHeal" })]
pub struct BuffFioraRHeal {
    pub center: Vec3,
    pub team: Team,
    pub timer: Timer,
    pub tick: Timer,
    pub heal_per_second: f32,
    pub heal_radius: f32,
}

pub fn fixed_update(
    mut commands: Commands,
    mut q_buff_fiora_r: Query<(Entity, &BuffOf, &mut BuffFioraR)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, _buff_of, mut buff) in q_buff_fiora_r.iter_mut() {
        if !buff.is_active() {
            buff.active_timer.tick(time.delta());
            continue;
        }

        if !buff.timeout_red_triggered && buff.remove_timer.remaining_secs() <= VITAL_R_TIMEOUT {
            buff.timeout_red_triggered = true;
        }

        buff.remove_timer.tick(time.delta());

        if buff.remove_timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_fiora_r_heal(
    mut commands: Commands,
    mut q_heal: Query<(Entity, &mut BuffFioraRHeal)>,
    mut q_allies: Query<(&Transform, &Team, &mut Health), Without<Death>>,
    time: Res<Time<Fixed>>,
) {
    for (buff_entity, mut heal) in q_heal.iter_mut() {
        heal.timer.tick(time.delta());
        heal.tick.tick(time.delta());
        if heal.tick.just_finished() {
            for (transform, team, mut hp) in q_allies.iter_mut() {
                if team != &heal.team {
                    continue;
                }
                if transform.translation.distance(heal.center) > heal.heal_radius {
                    continue;
                }
                hp.value = (hp.value + heal.heal_per_second).min(hp.max);
            }
        }
        if heal.timer.is_finished() {
            commands.entity(buff_entity).despawn();
        }
    }
}

pub fn on_fiora_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_targets: Query<(Entity, &Transform, &Team), (With<Champion>, Without<Death>)>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let Ok(caster_tf) = q_transform.get(entity) else {
        return;
    };
    let Ok(caster_team) = q_team.get(entity) else {
        return;
    };
    let caster_xz = caster_tf.translation.xz();

    let mut best: Option<(Entity, f32)> = None;
    for (target, t_tf, t_team) in q_targets.iter() {
        if t_team == caster_team {
            continue;
        }
        let t_xz = t_tf.translation.xz();
        let range = get_skill_data_value(spell_obj, "MSRingRadius", skill.level).unwrap_or(500.0);
        if t_xz.distance(caster_xz) > range {
            continue;
        }
        let d = t_xz.distance(trigger.point);
        if best.map_or(true, |(_, bd)| d < bd) {
            best = Some((target, d));
        }
    }
    let Some((target, _)) = best else {
        return;
    };

    let duration = get_skill_data_value(spell_obj, "MarkDuration", skill.level).unwrap_or(8.0);
    let ms_percent = get_skill_data_value(spell_obj, "PercentMS", skill.level).unwrap_or(0.3);
    let vital_pct = get_skill_data_value(spell_obj, "VitalPercent", skill.level).unwrap_or(0.03);
    let heal_per_second = get_skill_data_value(spell_obj, "HealPerSecond", skill.level).unwrap_or(50.0);
    let heal_duration = get_skill_data_value(spell_obj, "HealDuration", skill.level).unwrap_or(5.0);
    let heal_radius = get_skill_data_value(spell_obj, "HealRingRadius", skill.level).unwrap_or(550.0);

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffFioraR::new(
            skill.level, duration, FIORA_R_ACTIVE_DURATION,
            vital_pct, heal_per_second, heal_duration, heal_radius,
        ));

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(ms_percent, duration));
}

/// 监听伤害事件：从匹配方向击破一个 R 要害并造成最大生命值真实伤害；
/// 四要害全破或目标死亡（已破≥1）时触发治疗光环。
pub fn on_r_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_target: Query<(&Transform, &Team, &Health)>,
    q_source: Query<(&Transform, &Team)>,
    mut q_buff_fiora_r: Query<(Entity, &BuffOf, &mut BuffFioraR)>,
) {
    let target_entity = trigger.event_target();
    if q_fiora.get(trigger.source).is_err() {
        return;
    }
    let Ok((source_tf, source_team)) = q_source.get(trigger.source) else {
        return;
    };

    let Some((buff_entity, mut buff_fiora_r)) =
        q_buff_fiora_r
            .iter_mut()
            .find_map(|(entity, buff_of, buff_fiora_r)| {
                if buff_of.0 == target_entity {
                    Some((entity, buff_fiora_r))
                } else {
                    None
                }
            })
    else {
        return;
    };

    let Ok((target_tf, target_team, hp)) = q_target.get(target_entity) else {
        return;
    };

    if target_team == source_team {
        return;
    }

    if !buff_fiora_r.is_active() {
        return;
    }

    let source_position = source_tf.translation.xz();
    let target_position = target_tf.translation.xz();

    let mut hit_direction: Option<Direction> = None;
    buff_fiora_r.vitals.retain(|direction| {
        if hit_direction.is_none() && is_in_direction(source_position, target_position, direction) {
            hit_direction = Some(direction.clone());
            false
        } else {
            true
        }
    });

    let Some(_direction) = hit_direction else {
        return;
    };

    let true_damage = hp.max * buff_fiora_r.vital_pct;
    commands
        .entity(target_entity)
        .trigger(|e| CommandDamageCreate {
            entity: e,
            source: trigger.source,
            damage_type: DamageType::True,
            amount: true_damage,
            tag: None,
        });

    let all_broken = buff_fiora_r.vitals.is_empty();
    let target_dead = hp.value <= 0.0;
    if all_broken || target_dead {
        commands
            .entity(trigger.source)
            .with_related::<BuffOf>(BuffFioraRHeal {
                center: Vec3::new(target_position.x, 0.0, target_position.y),
                team: *source_team,
                timer: Timer::from_seconds(buff_fiora_r.heal_duration, TimerMode::Once),
                tick: Timer::from_seconds(1.0, TimerMode::Repeating),
                heal_per_second: buff_fiora_r.heal_per_second,
                heal_radius: buff_fiora_r.heal_radius,
            });
        commands.entity(buff_entity).despawn();
    }
}