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

use crate::cassiopeia::buffs::BuffCassioPoison;

#[derive(Default)]
pub struct PluginCassiopeia;

impl Plugin for PluginCassiopeia {
    fn build(&self, app: &mut App) {
        app.add_observer(on_cassiopeia_skill_cast);
        app.add_observer(on_cassiopeia_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Cassiopeia"))]
#[reflect(Component)]
pub struct Cassiopeia;

fn on_cassiopeia_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_cassio: Query<(), With<Cassiopeia>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_cassio.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_cassio_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_cassio_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_cassio_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_cassio_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_cassio_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Cassio_Q_Cast"),
    });

    // Q is ground targeted area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Cassio_Q_Hit")),
        }],
    });
}

fn cast_cassio_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Cassio_W_Cast"),
    });

    // W is a poison cloud
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Cassio_W_Hit")),
        }],
    });
}

fn cast_cassio_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Cassio_E_Cast"),
    });

    // E is targeted damage to poisoned enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Cassio_E_Hit")),
        }],
    });
}

fn cast_cassio_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Cassio_R_Cast"),
    });

    // R is a cone that stuns facing enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 80.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Cassio_R_Hit")),
        }],
    });
}

fn on_cassiopeia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_cassio: Query<(), With<Cassiopeia>>,
) {
    let source = trigger.source;
    if q_cassio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply poison
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCassioPoison::new());
}
