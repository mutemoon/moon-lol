//! Q - 利刃冲击 (Bladesurge)

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::Buffs;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::irelia::buffs::DebuffIreliaUnsteady;
use crate::irelia::IRELIA_Q_DAMAGE_TAG;

/// Q 命中判定距离（ron `castRange`）
pub const IRELIA_Q_RANGE: f32 = 600.0;
const IRELIA_Q_DASH_MAX: f32 = 250.0;

pub fn on_irelia_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<crate::irelia::Irelia>>,
    q_skill: Query<(&Skill, &CoolDown)>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_unsteady: Query<&DebuffIreliaUnsteady>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok((skill, cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    let point = trigger.point;
    let nearest = q_enemies
        .iter()
        .filter(|(_, _, t)| **t != *team)
        .filter(|(_, tf, _)| tf.translation.xz().distance(point) <= IRELIA_Q_RANGE)
        .min_by(|a, b| {
            let da = a.1.translation.xz().distance(point);
            let db = b.1.translation.xz().distance(point);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });

    if let Some((target, _, _)) = nearest {
        let amount = get_skill_value(spell, "champion_damage", skill.level, |stat| {
            if stat == 2 { ad } else { 0.0 }
        })
        .unwrap_or(0.0);

        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Physical,
            amount,
            tag: Some(IRELIA_Q_DAMAGE_TAG),
        });

        let is_unsteady = q_buffs
            .get(target)
            .map(|buffs| buffs.iter().any(|b| q_unsteady.get(b).is_ok()))
            .unwrap_or(false);
        if is_unsteady {
            commands.entity(trigger.skill_entity).insert(CoolDown {
                duration: cooldown.duration,
                timer: None,
            });
        }
    }

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer {
            max: IRELIA_Q_DASH_MAX,
        },
        speed: 800.0,
    });
}