pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::jayce::buffs::BuffJaycePassive;

#[derive(Default)]
pub struct PluginJayce;

impl Plugin for PluginJayce {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jayce_skill_cast);
        app.add_observer(on_jayce_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jayce"))]
#[reflect(Component)]
pub struct Jayce;

fn on_jayce_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_jayce_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_jayce_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_jayce_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_jayce_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_jayce_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jayce_Q_Cast"),
    });

    // Q is a skillshot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Jayce_Q_Hit")),
        }],
    });
}

fn cast_jayce_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jayce_W_Cast"),
    });

    // W is an area slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Jayce_W_Hit")),
        }],
    });
}

fn cast_jayce_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jayce_E_Cast"),
    });

    // E is a knockback
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 500.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            },
        }),
        speed: 1000.0,
    });
}

fn cast_jayce_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jayce_R_Cast"),
    });
    // R transforms between hammer and cannon forms
}

fn on_jayce_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
) {
    let source = trigger.source;
    if q_jayce.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJaycePassive::new());
}
