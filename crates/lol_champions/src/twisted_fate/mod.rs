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

use crate::twisted_fate::buffs::{BuffTwistedFateWSlow, BuffTwistedFateWStun};

#[derive(Default)]
pub struct PluginTwistedFate;

impl Plugin for PluginTwistedFate {
    fn build(&self, app: &mut App) {
        app.add_observer(on_twisted_fate_skill_cast);
        app.add_observer(on_twisted_fate_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("TwistedFate"))]
#[reflect(Component)]
pub struct TwistedFate;

fn on_twisted_fate_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_twisted_fate.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_twisted_fate_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_twisted_fate_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_twisted_fate_e(&mut commands, entity),
        SkillSlot::R => cast_twisted_fate_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_twisted_fate_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TwistedFate_Q_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1450.0,
                angle: 25.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("TwistedFate_Q_Hit")),
        }],
    });
}

fn cast_twisted_fate_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TwistedFate_W_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 325.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("TwistedFate_W_Hit")),
        }],
    });
}

fn cast_twisted_fate_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TwistedFate_E_Cast"),
    });
}

fn cast_twisted_fate_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TwistedFate_R_Cast"),
    });
}

fn on_twisted_fate_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
) {
    let source = trigger.source;
    if q_twisted_fate.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwistedFateWSlow::new(0.35, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwistedFateWStun::new(1.5));
}
