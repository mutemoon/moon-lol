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

use crate::gangplank::buffs::BuffGangplankPassive;

#[derive(Default)]
pub struct PluginGangplank;

impl Plugin for PluginGangplank {
    fn build(&self, app: &mut App) {
        app.add_observer(on_gangplank_skill_cast);
        app.add_observer(on_gangplank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Gangplank"))]
#[reflect(Component)]
pub struct Gangplank;

fn on_gangplank_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_gangplank_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_gangplank_w(&mut commands, entity),
        SkillSlot::E => cast_gangplank_e(&mut commands, entity),
        SkillSlot::R => cast_gangplank_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_gangplank_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Gangplank_Q_Cast"),
    });

    // Q is targeted damage
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
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Gangplank_Q_Hit")),
        }],
    });
}

fn cast_gangplank_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Gangplank_W_Cast"),
    });
    // W removes CC and heals
}

fn cast_gangplank_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Gangplank_E_Cast"),
    });
    // E places barrel
}

fn cast_gangplank_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Gangplank_R_Cast"),
    });

    // R is global AoE
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 20000.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Gangplank_R_Hit")),
        }],
    });
}

fn on_gangplank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
) {
    let source = trigger.source;
    if q_gangplank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGangplankPassive::new());
}
