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

use crate::neeko::buffs::BuffNeekoE;

#[derive(Default)]
pub struct PluginNeeko;

impl Plugin for PluginNeeko {
    fn build(&self, app: &mut App) {
        app.add_observer(on_neeko_skill_cast);
        app.add_observer(on_neeko_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Neeko"))]
#[reflect(Component)]
pub struct Neeko;

fn on_neeko_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_neeko.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_neeko_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_neeko_w(&mut commands, entity),
        SkillSlot::E => cast_neeko_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_neeko_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_neeko_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Neeko_Q_Cast"),
    });

    // Q is a bloom burst
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 800.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Neeko_Q_Hit")),
        }],
    });
}

fn cast_neeko_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Neeko_W_Cast"),
    });

    // W is a shapesplitter dash
}

fn cast_neeko_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Neeko_E_Cast"),
    });

    // E is a root
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1000.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Neeko_E_Hit")),
        }],
    });
}

fn cast_neeko_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Neeko_R_Cast"),
    });

    // R is an AoE knockup/stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 590.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Neeko_R_Hit")),
        }],
    });
}

fn on_neeko_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
) {
    let source = trigger.source;
    if q_neeko.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNeekoE::new(1.5, 2.0));
}
