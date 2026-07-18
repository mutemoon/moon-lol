//! Aatrox Q - 暗裔之刃 (The Darkin Blade)
//!
//! 三段重施，每段 +25% 伤害；边缘（sweet spot）+70% + 击飞。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffKnockup;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value,
    get_skill_value,
};
use lol_core::team::Team;

use crate::aatrox::Aatrox;

/// Q 三段重施窗口（秒）
pub const AATROX_Q_RECAST_WINDOW: f32 = 4.0;
/// Q 最大半径
pub const AATROX_Q_RADIUS: f32 = 300.0;
/// Q 边缘 sweet spot 起始距离
pub const AATROX_Q_SWEET_SPOT_MIN: f32 = 200.0;
/// Q 伤害标签
pub const AATROX_Q_TAG: u32 = 11;

pub fn on_aatrox_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let caster_pos = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or(Vec2::ZERO);
    let Ok(caster_team) = q_team.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    let stage = recast.map(|w| w.stage).unwrap_or(1);
    let anim = match stage {
        2 => "Q2IntoIdle",
        3 => "Q3IntoPassiveIdle",
        _ => "Q1IntoIdle",
    };
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: anim.to_string(),
        repeat: false,
        duration: None,
    });

    let base = get_skill_value(spell_obj, "q_damage", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);
    let ramp = get_skill_data_value(spell_obj, "QRampBonus", skill.level).unwrap_or(0.25);
    let sweet = get_skill_data_value(spell_obj, "QSweetSpotBonus", skill.level).unwrap_or(0.7);
    let knockup_dur =
        get_skill_data_value(spell_obj, "QKnockupDuration", skill.level).unwrap_or(0.25);
    let extension = get_skill_data_value(spell_obj, "QExtensionTime", skill.level)
        .unwrap_or(AATROX_Q_RECAST_WINDOW);

    let ramp_mult = 1.0 + (stage as f32 - 1.0) * ramp;
    let sweet_mult = 1.0 + sweet;
    let center_dmg = base * ramp_mult;
    let sweet_dmg = center_dmg * sweet_mult;

    for (enemy, enemy_tf) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == caster_team {
            continue;
        }
        let dist = enemy_tf.translation.xz().distance(caster_pos);
        if dist > AATROX_Q_RADIUS {
            continue;
        }
        let (dmg, is_sweet) = if dist >= AATROX_Q_SWEET_SPOT_MIN {
            (sweet_dmg, true)
        } else {
            (center_dmg, false)
        };
        if dmg > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: dmg,
                tag: Some(AATROX_Q_TAG),
            });
        }
        if is_sweet {
            commands
                .entity(enemy)
                .with_related::<BuffOf>(DebuffKnockup::new(knockup_dur));
        }
    }

    if stage >= 3 {
        commands
            .entity(trigger.skill_entity)
            .remove::<SkillRecastWindow>();
        commands.entity(trigger.skill_entity).insert(CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        });
    } else {
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(stage + 1, 3, extension));
    }
}
