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

use crate::karma::buffs::{BuffKarmaE, BuffKarmaGatheringFire, BuffKarmaQ};

#[derive(Default)]
pub struct PluginKarma;

impl Plugin for PluginKarma {
    fn build(&self, app: &mut App) {
        app.add_observer(on_karma_skill_cast);
        app.add_observer(on_karma_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Karma"))]
#[reflect(Component)]
pub struct Karma;

fn on_karma_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_karma.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_karma_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_karma_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_karma_e(&mut commands, entity),
        SkillSlot::R => cast_karma_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_karma_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Karma_Q_Cast"),
    });

    // Q is a skillshot that damages and slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 950.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Karma_Q_Hit")),
        }],
    });
}

fn cast_karma_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Karma_W_Cast"),
    });

    // W roots after delay
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 675.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Karma_W_Hit")),
        }],
    });
}

fn cast_karma_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Karma_E_Cast"),
    });

    // E provides shield and movement speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKarmaE::new(80.0, 0.4, 2.0));
}

fn cast_karma_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Karma_R_Cast"),
    });

    // R empowers next skill (handled by gathering fire passive)
}

fn on_karma_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
) {
    let source = trigger.source;
    if q_karma.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKarmaQ::new(0.4, 1.5));

    // Passive reduces R cooldown on hit
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKarmaGatheringFire::new(2.0));
}
