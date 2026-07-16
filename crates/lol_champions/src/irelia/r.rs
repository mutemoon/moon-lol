//! R - 先锋之刃 (Vanguard's Edge)

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::irelia::IRELIA_R_DAMAGE_TAG;

pub const IRELIA_R_RADIUS: f32 = 350.0;

pub fn on_irelia_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<crate::irelia::Irelia>>,
    q_skill: Query<&Skill>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    let amount = get_skill_value(spell, "missile_damage", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    for (target, tf, t) in q_enemies.iter() {
        if *t == *team {
            continue;
        }
        if tf.translation.xz().distance(trigger.point) > IRELIA_R_RADIUS {
            continue;
        }
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Magic,
            amount,
            tag: Some(IRELIA_R_DAMAGE_TAG),
        });
    }
}