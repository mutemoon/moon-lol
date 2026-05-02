pub mod buffs;

use bevy::asset::Handle;
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

use crate::morgana::buffs::{BuffMorganaE, BuffMorganaQ};

#[derive(Default)]
pub struct PluginMorgana;

impl Plugin for PluginMorgana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_morgana_skill_cast);
        app.add_observer(on_morgana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Morgana"))]
#[reflect(Component)]
pub struct Morgana;

fn on_morgana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morgana: Query<(), With<Morgana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_morgana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_morgana_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_morgana_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_morgana_e(&mut commands, entity),
        SkillSlot::R => cast_morgana_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_morgana_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Morgana_Q_Cast"),
    });

    // Q binds enemy
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1300.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Morgana_Q_Hit")),
        }],
    });
}

fn cast_morgana_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Morgana_W_Cast"),
    });

    // W is a DoT zone
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Morgana_W_Hit")),
        }],
    });
}

fn cast_morgana_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Morgana_E_Cast"),
    });

    // E is a shield that blocks CC
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMorganaE::new(150.0, true, 5.0));
}

fn cast_morgana_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Morgana_R_Cast"),
    });

    // R chains nearby enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 625.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Morgana_R_Hit")),
        }],
    });
}

fn on_morgana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morgana: Query<(), With<Morgana>>,
) {
    let source = trigger.source;
    if q_morgana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMorganaQ::new(2.0, 2.0));
}
