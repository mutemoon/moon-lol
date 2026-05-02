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

use crate::rumble::buffs::BuffRumbleW;

#[derive(Default)]
pub struct PluginRumble;

impl Plugin for PluginRumble {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rumble_skill_cast);
        app.add_observer(on_rumble_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rumble"))]
#[reflect(Component)]
pub struct Rumble;

fn on_rumble_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rumble.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rumble_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_rumble_w(&mut commands, entity),
        SkillSlot::E => cast_rumble_e(&mut commands, entity, skill.spell.clone()),
        SkillSlot::R => cast_rumble_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_rumble_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rumble_Q_Cast"),
    });

    // Q is electro harpoon - damage over time
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Rumble_Q_Hit")),
        }],
    });
}

fn cast_rumble_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rumble_W_Cast"),
    });

    // W is scrap shield - shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRumbleW::new(50.0, 1.5));
}

fn cast_rumble_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rumble_E_Cast"),
    });

    // E is electro harpoon - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Rumble_E_Hit")),
        }],
    });
}

fn cast_rumble_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rumble_R_Cast"),
    });

    // R is electro fire - large AoE damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Rumble_R_Hit")),
        }],
    });
}

fn on_rumble_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
) {
    let source = trigger.source;
    if q_rumble.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRumbleW::new(50.0, 1.5));
}
