pub mod buffs;

use bevy::prelude::*;
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

use crate::braum::buffs::{BuffBraumPassive, BuffBraumW};

#[derive(Default)]
pub struct PluginBraum;

impl Plugin for PluginBraum {
    fn build(&self, app: &mut App) {
        app.add_observer(on_braum_skill_cast);
        app.add_observer(on_braum_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Braum"))]
#[reflect(Component)]
pub struct Braum;

fn on_braum_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_braum.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_braum_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_braum_w(&mut commands, entity),
        SkillSlot::E => cast_braum_e(&mut commands, entity),
        SkillSlot::R => cast_braum_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_braum_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Braum_Q_Cast"),
    });

    // Q is a skillshot that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Braum_Q_Hit")),
        }],
    });
}

fn cast_braum_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Braum_W_Cast"),
    });

    // W jumps to ally and grants armor/mr buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffBraumW::new());
}

fn cast_braum_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Braum_E_Cast"),
    });
    // E blocks projectiles - no direct damage
}

fn cast_braum_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Braum_R_Cast"),
    });

    // R is a knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Braum_R_Hit")),
        }],
    });
}

fn on_braum_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
) {
    let source = trigger.source;
    if q_braum.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.7, 2.0));
    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBraumPassive::new());
}
