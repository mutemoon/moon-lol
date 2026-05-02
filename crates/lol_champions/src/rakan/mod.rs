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

use crate::rakan::buffs::{BuffRakanR, BuffRakanW};

#[derive(Default)]
pub struct PluginRakan;

impl Plugin for PluginRakan {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rakan_skill_cast);
        app.add_observer(on_rakan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rakan"))]
#[reflect(Component)]
pub struct Rakan;

fn on_rakan_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rakan.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rakan_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_rakan_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_rakan_e(&mut commands, entity, skill.spell.clone()),
        SkillSlot::R => cast_rakan_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_rakan_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rakan_Q_Cast"),
    });

    // Q is gleaming quill - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Rakan_Q_Hit")),
        }],
    });
}

fn cast_rakan_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rakan_W_Cast"),
    });

    // W is grand entrance - knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 650.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Rakan_W_Hit")),
        }],
    });
}

fn cast_rakan_e(commands: &mut Commands, entity: Entity, _skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rakan_E_Cast"),
    });

    // E is battle dance - shield to ally
}

fn cast_rakan_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Rakan_R_Cast"),
    });

    // R is the quickness - damage and charm
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 150.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Rakan_R_Hit")),
        }],
    });
}

fn on_rakan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
) {
    let source = trigger.source;
    if q_rakan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W knockup
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRakanW::new(1.0, 1.5));
    // R charm and slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRakanR::new(1.5, 0.75, 2.0));
}
