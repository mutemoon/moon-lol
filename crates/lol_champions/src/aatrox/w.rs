//! Aatrox W - 冥府之链 (Infernal Chains)
//!
//! 命中伤害 + 减速 + 标记；1.5s 后引爆二次伤害 + 击飞。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffKnockup;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, get_skill_cast_radius, get_skill_data_value, get_skill_value,
};
use lol_core::team::Team;

use crate::aatrox::Aatrox;
use crate::aatrox::buffs::DebuffAatroxWMark;

/// W 伤害标签
pub const AATROX_W_TAG: u32 = 12;
/// W 标记持续时长（秒）
pub const AATROX_W_MARK_DURATION: f32 = 1.5;

pub fn on_aatrox_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
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
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
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

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    let dmg = get_skill_value(spell_obj, "w_damage", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);
    let slow_pct = get_skill_data_value(spell_obj, "WSlowPercentage", skill.level)
        .map(|v| v.abs())
        .unwrap_or(0.25);
    let slow_dur = get_skill_data_value(spell_obj, "WSlowDuration", skill.level).unwrap_or(1.5);
    let range = get_skill_cast_radius(spell_obj, skill.level).unwrap_or(825.0);

    for (enemy, enemy_tf) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == caster_team {
            continue;
        }
        let dist = enemy_tf.translation.xz().distance(caster_pos);
        if dist > range {
            continue;
        }
        if dmg > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: dmg,
                tag: Some(AATROX_W_TAG),
            });
        }
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffSlow::new(slow_pct, slow_dur));
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffAatroxWMark::new(
                entity,
                enemy,
                dmg,
                AATROX_W_MARK_DURATION,
            ));
    }
}

/// W 标记引爆：到时造成等额二次伤害 + 击飞，并移除标记。
pub fn update_aatrox_w_marks(
    time: Res<Time>,
    mut commands: Commands,
    mut q_marks: Query<(Entity, &mut DebuffAatroxWMark)>,
) {
    for (buff_entity, mut mark) in q_marks.iter_mut() {
        mark.timer.tick(time.delta());
        if mark.timer.just_finished() {
            let target = mark.target;
            let source = mark.source;
            let damage = mark.damage;
            if damage > 0.0 {
                commands.entity(target).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source,
                    damage_type: DamageType::Physical,
                    amount: damage,
                    tag: Some(AATROX_W_TAG),
                });
            }
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffKnockup::new(0.5));
            commands.entity(buff_entity).despawn();
        }
    }
}