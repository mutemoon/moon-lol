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

use crate::singed::buffs::BuffSingedE;

#[derive(Default)]
pub struct PluginSinged;

impl Plugin for PluginSinged {
    fn build(&self, app: &mut App) {
        app.add_observer(on_singed_skill_cast);
        app.add_observer(on_singed_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Singed"))]
#[reflect(Component)]
pub struct Singed;

fn on_singed_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_singed_q(&mut commands, entity),
        SkillSlot::W => cast_singed_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_singed_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_singed_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_singed_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Singed_Q_Cast"),
    });

    // Q is poison trail - damage over time
}

fn cast_singed_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Singed_W_Cast"),
    });

    // W is mega adhesive - slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Singed_W_Hit")),
        }],
    });
}

fn cast_singed_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Singed_E_Cast"),
    });

    // E is fling - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 400.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Singed_E_Hit")),
        }],
    });
}

fn cast_singed_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Singed_R_Cast"),
    });

    // R is insanity - movespeed buff
}

fn on_singed_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
) {
    let source = trigger.source;
    if q_singed.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSingedE::new(0.6, 3.0));
}
