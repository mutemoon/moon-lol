pub mod buffs;

use bevy::asset::Handle;
use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSilence;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::malzahar::buffs::BuffMalzaharE;

#[derive(Default)]
pub struct PluginMalzahar;

impl Plugin for PluginMalzahar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_malzahar_skill_cast);
        app.add_observer(on_malzahar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Malzahar"))]
#[reflect(Component)]
pub struct Malzahar;

fn on_malzahar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_malzahar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_malzahar_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_malzahar_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_malzahar_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_malzahar_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_malzahar_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Malzahar_Q_Cast"),
    });

    // Q opens void gates and silences
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Malzahar_Q_Hit")),
        }],
    });
}

fn cast_malzahar_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Malzahar_W_Cast"),
    });

    // W summons voidlings
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 150.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Malzahar_W_Hit")),
        }],
    });
}

fn cast_malzahar_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Malzahar_E_Cast"),
    });

    // E infects target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 650.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Malzahar_E_Hit")),
        }],
    });
}

fn cast_malzahar_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Malzahar_R_Cast"),
    });

    // R suppresses target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Malzahar_R_Hit")),
        }],
    });
}

fn on_malzahar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
) {
    let source = trigger.source;
    if q_malzahar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q silences
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSilence::new(1.5));

    // E applies infection
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMalzaharE::new(50.0, 4.0));
}
