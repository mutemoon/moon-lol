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

use crate::nasus::buffs::BuffNasusW;

#[derive(Default)]
pub struct PluginNasus;

impl Plugin for PluginNasus {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nasus_skill_cast);
        app.add_observer(on_nasus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nasus"))]
#[reflect(Component)]
pub struct Nasus;

fn on_nasus_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nasus.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_nasus_q(&mut commands, entity),
        SkillSlot::W => cast_nasus_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_nasus_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_nasus_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_nasus_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nasus_Q_Cast"),
    });

    // Q is a siphoning strike
}

fn cast_nasus_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nasus_W_Cast"),
    });

    // W is a slow
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
            particle: Some(hash_bin("Nasus_W_Hit")),
        }],
    });
}

fn cast_nasus_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nasus_E_Cast"),
    });

    // E is an area damage and armor reduction
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
            particle: Some(hash_bin("Nasus_E_Hit")),
        }],
    });
}

fn cast_nasus_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nasus_R_Cast"),
    });

    // R transforms Nasus
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nasus_R_Hit")),
        }],
    });
}

fn on_nasus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
) {
    let source = trigger.source;
    if q_nasus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNasusW::new(0.5, 5.0));
}
