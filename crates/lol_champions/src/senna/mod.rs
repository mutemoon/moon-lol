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
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::senna::buffs::BuffSennaW;

#[derive(Default)]
pub struct PluginSenna;

impl Plugin for PluginSenna {
    fn build(&self, app: &mut App) {
        app.add_observer(on_senna_skill_cast);
        app.add_observer(on_senna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Senna"))]
#[reflect(Component)]
pub struct Senna;

fn on_senna_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_senna.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_senna_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_senna_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_senna_e(&mut commands, entity),
        SkillSlot::R => cast_senna_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_senna_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Senna_Q_Cast"),
    });

    // Q is duskblade of shadow - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Senna_Q_Hit")),
        }],
    });
}

fn cast_senna_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Senna_W_Cast"),
    });

    // W is last embrace - root
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1000.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Senna_W_Hit")),
        }],
    });
}

fn cast_senna_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Senna_E_Cast"),
    });

    // E is curtain of darkness - camouflage
}

fn cast_senna_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Senna_R_Cast"),
    });

    // R is pierce the veil - AoE damage and shield
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 2500.0,
                angle: 50.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Senna_R_Hit")),
        }],
    });
}

fn on_senna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
) {
    let source = trigger.source;
    if q_senna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSennaW::new(1.5, 2.0));
}
