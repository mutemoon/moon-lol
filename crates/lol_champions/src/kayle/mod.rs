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

use crate::kayle::buffs::{BuffKaylePassive, BuffKayleR, BuffKayleW};

#[derive(Default)]
pub struct PluginKayle;

impl Plugin for PluginKayle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kayle_skill_cast);
        app.add_observer(on_kayle_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kayle"))]
#[reflect(Component)]
pub struct Kayle;

fn on_kayle_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kayle.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_kayle_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_kayle_w(&mut commands, entity),
        SkillSlot::E => cast_kayle_e(&mut commands, entity),
        SkillSlot::R => cast_kayle_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_kayle_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kayle_Q_Cast"),
    });

    // Q is a skillshot that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Kayle_Q_Hit")),
        }],
    });
}

fn cast_kayle_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kayle_W_Cast"),
    });

    // W heals and grants movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKayleW::new(80.0, 0.35, 2.5));
}

fn cast_kayle_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kayle_E_Cast"),
    });

    // E enhances next attack
}

fn cast_kayle_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Kayle_R_Cast"),
    });

    // R makes Kayle invulnerable and deals damage after delay
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKayleR::new(2.5));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Kayle_R_Hit")),
        }],
    });
}

fn on_kayle_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
) {
    let source = trigger.source;
    if q_kayle.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive grants attackspeed
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKaylePassive::new(0.15, 3.0));
}
