use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::Damage;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};

use crate::riven::Riven;

pub fn on_riven_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
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

    // 位移瞬间的冲刺粒子 + 护盾粒子（盾破/到期时在 cleanup_shield_visuals 撤下）
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: "Riven_E_Mis".to_string(),
        rotation: None,
        resolver_entity: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: "Riven_E_Shield".to_string(),
        rotation: None,
        resolver_entity: None,
    });

    // 创建护盾 buff 实体并建立关系
    let buff_entity = commands.spawn(BuffShieldWhite::new(shield_value)).id();
    commands
        .entity(entity)
        .add_related::<BuffOf>(&[buff_entity]);

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Fixed(250.0),
        speed: 1000.0,
    });
}
