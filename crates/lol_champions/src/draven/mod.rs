pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::draven::buffs::BuffDravenPassive;

#[derive(Default)]
pub struct PluginDraven;

impl Plugin for PluginDraven {
    fn build(&self, app: &mut App) {
        app.add_observer(on_draven_skill_cast);
        app.add_observer(on_draven_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Draven"))]
#[reflect(Component)]
pub struct Draven;

fn on_draven_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_draven_q(&mut commands, entity),
        SkillSlot::W => cast_draven_w(&mut commands, entity),
        SkillSlot::E => cast_draven_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_draven_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_draven_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Draven_Q_Cast"),
    });

    // Q enhances next attack - handled by buff system
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDravenPassive::new());
}

fn cast_draven_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Draven_W_Cast"),
    });
    // W is movement speed buff - handled by buff system
}

fn cast_draven_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Draven_E_Cast"),
    });

    // E is a knockback skillshot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Draven_E_Hit")),
        }],
    });
}

fn cast_draven_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Draven_R_Cast"),
    });

    // R is global damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 20000.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Draven_R_Hit")),
        }],
    });
}

fn on_draven_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
) {
    let source = trigger.source;
    if q_draven.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 2.0));
}
