//! E - 完美合奏 (Flawless Duet)

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_value,
};
use lol_core::team::Team;

use crate::irelia::IRELIA_E2_DAMAGE_TAG;

pub const IRELIA_E_RECAST_WINDOW: f32 = 4.0;
pub const IRELIA_E_RADIUS: f32 = 200.0;

pub fn on_irelia_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<crate::irelia::Irelia>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(2, 2, IRELIA_E_RECAST_WINDOW));
        return;
    }

    let amount = get_skill_value(spell, "total_damage", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    for (target, tf, t) in q_enemies.iter() {
        if *t == *team {
            continue;
        }
        if tf.translation.xz().distance(trigger.point) > IRELIA_E_RADIUS {
            continue;
        }
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Magic,
            amount,
            tag: Some(IRELIA_E2_DAMAGE_TAG),
        });
    }

    commands
        .entity(trigger.skill_entity)
        .remove::<SkillRecastWindow>();
    commands.entity(trigger.skill_entity).insert(CoolDown {
        duration: cooldown.duration,
        timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
    });
}
