pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::annie::buffs::{BuffAnniePassive, BuffAnnieShield};

#[derive(Default)]
pub struct PluginAnnie;

impl Plugin for PluginAnnie {
    fn build(&self, app: &mut App) {
        app.add_observer(on_annie_skill_cast);
        app.add_observer(on_annie_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Annie"))]
#[reflect(Component)]
pub struct Annie;

fn on_annie_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_annie.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_annie_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_annie_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_annie_e(&mut commands, entity),
        SkillSlot::R => cast_annie_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_annie_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Annie_Q_Cast"),
    });

    // Q is targeted damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 625.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Annie_Q_Hit")),
        }],
    });

    // Increment passive stacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn cast_annie_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Annie_W_Cast"),
    });

    // W is a cone
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 50.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Annie_W_Hit")),
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn cast_annie_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Annie_E_Cast"),
    });

    // E grants shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnnieShield::new());
}

fn cast_annie_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Annie_R_Cast"),
    });

    // R summons Tibbers - area damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Annie_R_Hit")),
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn on_annie_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
) {
    let source = trigger.source;
    if q_annie.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Check if Annie has 4 passive stacks for stun
    // For now, just stun
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.5));
}
