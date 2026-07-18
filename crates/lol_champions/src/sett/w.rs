//! Sett W - 强腕重击 (Haymaker)
//!
//! 被动储存"灰心"（受到伤害累积）；主动把灰心转化为白盾 + 锥形伤害。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, delay_from_cast_frame};

use crate::sett::Sett;
use crate::sett::buffs::{SettGrit, SettWShield};

pub fn on_sett_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_grit: Query<&SettGrit>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
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

    // 读取已储存的灰心，主动施放时全部转化为护盾并清零
    let grit_stored = q_grit.get(entity).map(|g| g.stored).unwrap_or(0.0);
    if grit_stored > 0.0 {
        commands.entity(entity).queue(move |mut e: EntityWorldMut| {
            if let Some(mut grit) = e.get_mut::<SettGrit>() {
                grit.stored = 0.0;
            }
        });
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    // 灰心 -> 白盾子 buff
    if grit_stored > 0.0 {
        let shield_entity = commands
            .spawn((BuffShieldWhite::new(grit_stored), SettWShield::new()))
            .id();
        commands
            .entity(entity)
            .add_related::<BuffOf>(&[shield_entity]);
    }

    // 锥形：中心窄扇形（30°）真实伤害，两侧（全 75° 扇形排除中心）物理伤害。
    let full_sector = DamageShape::Sector {
        radius: 350.0,
        angle: 75.0,
    };
    let center_sector = DamageShape::Sector {
        radius: 350.0,
        angle: 30.0,
    };
    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill.spell.clone(),
        skill_level: skill.level,
        delay: delay_from_cast_frame(spell_obj),
        point: trigger.point,
        origin: AoEOrigin::Caster,
        effects: vec![
            ActionDamageEffect {
                shape: full_sector,
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "damage_calc".to_string(),
                    damage_type: DamageType::Physical,
                    ..Default::default()
                }],
                exclude: vec![center_sector.clone()],
                ..Default::default()
            },
            ActionDamageEffect {
                shape: center_sector,
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "damage_calc".to_string(),
                    damage_type: DamageType::True,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        indicator: AoEIndicator {
            color: Color::srgba(1.0, 0.84, 0.0, 0.35),
            pulse: true,
            grow_from_zero: false,
            impact_burst_scale: 1.4,
            fade_duration: 0.3,
        },
    });
}

/// W 护盾到期移除（0.75s；被打空时由通用 update_shield_white 提前 despawn）。
pub fn update_sett_w_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut q_shield: Query<(Entity, &mut SettWShield)>,
) {
    for (entity, mut shield) in q_shield.iter_mut() {
        shield.timer.tick(time.delta());
        if shield.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
