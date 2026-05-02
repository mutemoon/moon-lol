pub mod buffs;

use bevy::prelude::{Handle, *};
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::orianna::buffs::BuffOriannaE;

#[derive(Default)]
pub struct PluginOrianna;

impl Plugin for PluginOrianna {
    fn build(&self, app: &mut App) {
        app.add_observer(on_orianna_skill_cast);
        app.add_observer(on_orianna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Orianna"))]
#[reflect(Component)]
pub struct Orianna;

fn on_orianna_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_orianna.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_orianna_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_orianna_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_orianna_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_orianna_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_orianna_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Orianna_Q_Cast"),
    });

    // Q commands ball to location
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 825.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Orianna_Q_Hit")),
        }],
    });
}

fn cast_orianna_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Orianna_W_Cast"),
    });

    // W deals damage and slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 225.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Orianna_W_Hit")),
        }],
    });
}

fn cast_orianna_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Orianna_E_Cast"),
    });

    // E shields ball carrier
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOriannaE::new(100.0, 30.0, 4.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1120.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Orianna_E_Hit")),
        }],
    });
}

fn cast_orianna_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Orianna_R_Cast"),
    });

    // R is a shockwave that knocks back
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 415.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Orianna_R_Hit")),
        }],
    });
}

fn on_orianna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
) {
    let source = trigger.source;
    if q_orianna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
