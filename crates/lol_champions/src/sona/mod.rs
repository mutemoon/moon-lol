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

use crate::sona::buffs::{BuffSonaE, BuffSonaW};

#[derive(Default)]
pub struct PluginSona;

impl Plugin for PluginSona {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sona_skill_cast);
        app.add_observer(on_sona_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sona"))]
#[reflect(Component)]
pub struct Sona;

fn on_sona_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_sona.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_sona_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_sona_w(&mut commands, entity),
        SkillSlot::E => cast_sona_e(&mut commands, entity),
        SkillSlot::R => cast_sona_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_sona_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sona_Q_Cast"),
    });

    // Q is hymn of valor - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Sona_Q_Hit")),
        }],
    });
}

fn cast_sona_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sona_W_Cast"),
    });

    // W is aria of perseverance - shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSonaW::new(40.0, 1.5));
}

fn cast_sona_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sona_E_Cast"),
    });

    // E is song of celerity - movespeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSonaE::new(0.2, 2.5));
}

fn cast_sona_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sona_R_Cast"),
    });

    // R is cure - AoE stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 40.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Sona_R_Hit")),
        }],
    });
}

fn on_sona_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
) {
    let source = trigger.source;
    if q_sona.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // R stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSonaE::new(0.2, 2.5));
}
