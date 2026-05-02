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

use crate::shaco::buffs::BuffShacoW;

#[derive(Default)]
pub struct PluginShaco;

impl Plugin for PluginShaco {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shaco_skill_cast);
        app.add_observer(on_shaco_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shaco"))]
#[reflect(Component)]
pub struct Shaco;

fn on_shaco_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_shaco.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_shaco_q(&mut commands, entity),
        SkillSlot::W => cast_shaco_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_shaco_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_shaco_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_shaco_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shaco_Q_Cast"),
    });

    // Q is vanish - invisibility
}

fn cast_shaco_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shaco_W_Cast"),
    });

    // W is jack inp - fear
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Shaco_W_Hit")),
        }],
    });
}

fn cast_shaco_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shaco_E_Cast"),
    });

    // E is two shiv - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 625.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Shaco_E_Hit")),
        }],
    });
}

fn cast_shaco_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shaco_R_Cast"),
    });

    // R is halluate - explosion
}

fn on_shaco_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
) {
    let source = trigger.source;
    if q_shaco.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W fears
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShacoW::new(0.5, 1.0));
}
