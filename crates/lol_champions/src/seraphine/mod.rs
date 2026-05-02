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

use crate::seraphine::buffs::{BuffSeraphineE, BuffSeraphineW};

#[derive(Default)]
pub struct PluginSeraphine;

impl Plugin for PluginSeraphine {
    fn build(&self, app: &mut App) {
        app.add_observer(on_seraphine_skill_cast);
        app.add_observer(on_seraphine_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Seraphine"))]
#[reflect(Component)]
pub struct Seraphine;

fn on_seraphine_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_seraphine.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_seraphine_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_seraphine_w(&mut commands, entity),
        SkillSlot::E => cast_seraphine_e(&mut commands, entity, skill.spell.clone()),
        SkillSlot::R => cast_seraphine_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_seraphine_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Seraphine_Q_Cast"),
    });

    // Q is high note - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Seraphine_Q_Hit")),
        }],
    });
}

fn cast_seraphine_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Seraphine_W_Cast"),
    });

    // W is solo - shield and slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSeraphineW::new(50.0, 2.5));
}

fn cast_seraphine_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Seraphine_E_Cast"),
    });

    // E is beat drop - stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1300.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Seraphine_E_Hit")),
        }],
    });
}

fn cast_seraphine_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Seraphine_R_Cast"),
    });

    // R is encore - AoE charm
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1500.0,
                angle: 50.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Seraphine_R_Hit")),
        }],
    });
}

fn on_seraphine_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
) {
    let source = trigger.source;
    if q_seraphine.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSeraphineE::new(0.75, 1.0));
}
