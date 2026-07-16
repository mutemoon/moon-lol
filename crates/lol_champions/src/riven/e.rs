use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::debug_sphere::DebugSphere;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::Damage;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};

use crate::riven::Riven;
use crate::riven::buffs::ShieldVisual;

pub fn on_riven_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    q_transform: Query<&Transform>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let shield_value = get_skill_value(spell_obj, "total_shield", skill.level, |stat| {
        if stat == 2 { damage_value } else { 0.0 }
    })
    .unwrap_or(100.0);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    // 创建护盾 buff 实体并建立关系
    let buff_entity = commands.spawn(BuffShieldWhite::new(shield_value)).id();
    commands
        .entity(entity)
        .add_related::<BuffOf>(&[buff_entity]);

    // 创建 3 个环绕球体
    let mut c0 = Entity::PLACEHOLDER;
    let mut c1 = Entity::PLACEHOLDER;
    let mut c2 = Entity::PLACEHOLDER;
    for (i, child) in [&mut c0, &mut c1, &mut c2].into_iter().enumerate() {
        let orb = commands
            .spawn((
                DebugSphere {
                    radius: 8.0,
                    color: Color::WHITE,
                },
                Transform::from_translation(Vec3::new(
                    100.0 * (i as f32 * core::f32::consts::TAU / 3.0).cos(),
                    50.0,
                    100.0 * (i as f32 * core::f32::consts::TAU / 3.0).sin(),
                )),
            ))
            .id();
        commands.entity(entity).add_child(orb);
        *child = orb;
    }
    commands.entity(entity).insert(ShieldVisual {
        children: [c0, c1, c2],
        angle: 0.0,
        buff_entity,
    });

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Fixed(250.0),
        speed: 1000.0,
    });
}