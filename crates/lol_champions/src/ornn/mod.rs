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

use crate::ornn::buffs::BuffOrnnQ;

#[derive(Default)]
pub struct PluginOrnn;

impl Plugin for PluginOrnn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ornn_skill_cast);
        app.add_observer(on_ornn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ornn"))]
#[reflect(Component)]
pub struct Ornn;

fn on_ornn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ornn.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_ornn_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_ornn_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_ornn_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_ornn_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_ornn_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Ornn_Q_Cast"),
    });

    // Q is volcanic rupture - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 750.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Ornn_Q_Hit")),
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOrnnQ::new(0.4, 2.0));
}

fn cast_ornn_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Ornn_W_Cast"),
    });

    // W is bellows breath - continuous damage and brittle
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
                angle: 25.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Ornn_W_Hit")),
        }],
    });
}

fn cast_ornn_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Ornn_E_Cast"),
    });

    // E is searing charge - dash that creates shockwave on terrain hit
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Ornn_E_Hit")),
        }],
    });
}

fn cast_ornn_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Ornn_R_Cast"),
    });

    // R is call of the forge god - large AoE damage and knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 3000.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Ornn_R_Hit")),
        }],
    });
}

fn on_ornn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
) {
    let source = trigger.source;
    if q_ornn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffOrnnQ::new(0.4, 2.0));
}
