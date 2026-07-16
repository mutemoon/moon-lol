//! Camille R（海克斯最后通牒 / Hextech Ultimatum）。
//!
//! R 标记目标，普攻命中被标记目标时造成额外魔法伤害
//! （`RPercentCurrentHPDamage`% 当前生命值），持续 `RDuration`。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};
use lol_core::team::Team;

use crate::camille::Camille;

/// R 标记：记录额外伤害百分比与持续时间。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleRMark" })]
pub struct BuffCamilleRMark {
    pub percent: f32,
    pub timer: Timer,
}

impl BuffCamilleRMark {
    pub fn new(percent: f32, duration: f32) -> Self {
        Self {
            percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 标记目标（R 施法时调用）。
pub fn apply_camille_r_mark(commands: &mut Commands, target: Entity, percent: f32, duration: f32) {
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCamilleRMark::new(percent, duration));
}

pub fn on_camille_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<&Team, With<Camille>>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(camille_team) = q_camille.get(entity) else {
        return;
    };

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;
    let percent = get_skill_data_value(spell_obj, "RPercentCurrentHPDamage", level).unwrap_or(2.0);
    let duration = get_skill_data_value(spell_obj, "RDuration", level).unwrap_or(1.75);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    let nearest = q_enemies
        .iter()
        .filter(|(_, _, team)| **team != *camille_team)
        .min_by(|a, b| {
            let da = (a.1.translation.xz() - trigger.point).length_squared();
            let db = (b.1.translation.xz() - trigger.point).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _, _)| e);
    if let Some(target) = nearest {
        apply_camille_r_mark(&mut commands, target, percent, duration);
    }

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer { max: 350.0 },
        speed: 800.0,
    });
}

/// 普攻命中被标记目标：造成额外魔法伤害（% 当前生命值）。
pub fn on_camille_r_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_buffs: Query<&Buffs>,
    q_mark: Query<&BuffCamilleRMark>,
    q_health: Query<&Health>,
) {
    let attacker = trigger.event_target();
    if q_camille.get(attacker).is_err() {
        return;
    }
    let target = trigger.target;
    let Ok(buffs) = q_buffs.get(target) else {
        return;
    };
    let Some(percent) = buffs
        .iter()
        .find_map(|b| q_mark.get(b).ok().map(|m| m.percent))
    else {
        return;
    };
    let Ok(hp) = q_health.get(target) else {
        return;
    };
    let bonus = hp.value * percent / 100.0;
    if bonus <= 0.0 {
        return;
    }
    commands.entity(target).trigger(|e| CommandDamageCreate {
        entity: e,
        source: attacker,
        damage_type: DamageType::Magic,
        amount: bonus,
        tag: None,
    });
}

/// R 标记计时：到期回收标记。
pub fn update_camille_r_mark(
    mut commands: Commands,
    mut q: Query<(Entity, &mut BuffCamilleRMark)>,
    time: Res<Time<Fixed>>,
) {
    for (e, mut mark) in q.iter_mut() {
        mark.timer.tick(time.delta());
        if mark.timer.is_finished() {
            commands.entity(e).despawn();
        }
    }
}