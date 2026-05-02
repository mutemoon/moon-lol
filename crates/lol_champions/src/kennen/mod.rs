pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::kennen::buffs::{BuffKennenE, BuffKennenMarkOfStorm, BuffKennenR};

#[derive(Default)]
pub struct PluginKennen;

impl Plugin for PluginKennen {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kennen_skill_cast);
        app.add_observer(on_kennen_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kennen"))]
#[reflect(Component)]
pub struct Kennen;

fn on_kennen_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kennen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_kennen_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_kennen_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_kennen_e(&mut commands, entity),
        SkillSlot::R => cast_kennen_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_kennen_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kennen_Q_Cast"),
    });

    // Q is a shuriken that applies mark
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Kennen_Q_Hit")),
        }],
    });
}

fn cast_kennen_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kennen_W_Cast"),
    });

    // W deals damage to marked enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 775.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Kennen_W_Hit")),
        }],
    });
}

fn cast_kennen_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kennen_E_Cast"),
    });

    // E grants movespeed and immunity
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKennenE::new(1.0, 0.6, 2.0));
}

fn cast_kennen_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kennen_R_Cast"),
    });

    // R summons storm that damages and applies marks
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Kennen_R_Hit")),
        }],
    });

    // R grants armor/mr
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKennenR::new(40.0, 40.0, 3.0));
}

fn on_kennen_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
) {
    let source = trigger.source;
    if q_kennen.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply mark of the storm (3 marks = stun)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKennenMarkOfStorm::new(1, 8.0));
}
