pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::graves::buffs::BuffGravesE;

#[derive(Default)]
pub struct PluginGraves;

impl Plugin for PluginGraves {
    fn build(&self, app: &mut App) {
        app.add_observer(on_graves_skill_cast);
        app.add_observer(on_graves_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Graves"))]
#[reflect(Component)]
pub struct Graves;

fn on_graves_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_graves: Query<(), With<Graves>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_graves.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_graves_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_graves_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_graves_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        SkillSlot::R => cast_graves_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_graves_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Graves_Q_Cast"),
    });

    // Q is a line shot that explodes
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 800.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Graves_Q_Hit")),
        }],
    });
}

fn cast_graves_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Graves_W_Cast"),
    });

    // W is a smoke screen
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Graves_W_Hit")),
        }],
    });
}

fn cast_graves_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Graves_E_Cast"),
    });

    // E is a dash
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell.clone(),
        move_type: DashMoveType::Pointer { max: 375.0 },
        damage: None,
        speed: 900.0,
    });

    // Grant armor buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGravesE::new());
}

fn cast_graves_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Graves_R_Cast"),
    });

    // R is a big shot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Graves_R_Hit")),
        }],
    });
}

fn on_graves_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_graves: Query<(), With<Graves>>,
) {
    let source = trigger.source;
    if q_graves.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}
