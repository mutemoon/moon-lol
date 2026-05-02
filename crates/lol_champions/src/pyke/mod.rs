pub mod buffs;

use bevy::prelude::{Handle, *};
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

use crate::pyke::buffs::{BuffPykeE, BuffPykeQ};

#[derive(Default)]
pub struct PluginPyke;

impl Plugin for PluginPyke {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pyke_skill_cast);
        app.add_observer(on_pyke_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Pyke"))]
#[reflect(Component)]
pub struct Pyke;

fn on_pyke_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_pyke.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_pyke_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_pyke_w(&mut commands, entity),
        SkillSlot::E => cast_pyke_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_pyke_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_pyke_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Pyke_Q_Cast"),
    });

    // Q is bone skewer - damage and pull
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Pyke_Q_Hit")),
        }],
    });
}

fn cast_pyke_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Pyke_W_Cast"),
    });

    // W is ghostwater dive - invisibility and movespeed
}

fn cast_pyke_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Pyke_E_Cast"),
    });

    // E is phantom undertow - dash and stun on return
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 550.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Pyke_E_Hit")),
        }],
    });
}

fn cast_pyke_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Pyke_R_Cast"),
    });

    // R is death from below - execute damage in AoE
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 750.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Pyke_R_Hit")),
        }],
    });
}

fn on_pyke_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
) {
    let source = trigger.source;
    if q_pyke.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffPykeQ::new(0.9, 1.0));
    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffPykeE::new(1.25, 1.5));
}
