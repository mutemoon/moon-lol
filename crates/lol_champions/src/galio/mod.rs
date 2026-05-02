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
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::galio::buffs::{BuffGalioPassive, BuffGalioW};

#[derive(Default)]
pub struct PluginGalio;

impl Plugin for PluginGalio {
    fn build(&self, app: &mut App) {
        app.add_observer(on_galio_skill_cast);
        app.add_observer(on_galio_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Galio"))]
#[reflect(Component)]
pub struct Galio;

fn on_galio_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_galio_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_galio_w(&mut commands, entity),
        SkillSlot::E => cast_galio_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        SkillSlot::R => cast_galio_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_galio_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Galio_Q_Cast"),
    });

    // Q is a tornado
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 825.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Galio_Q_Hit")),
        }],
    });
}

fn cast_galio_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Galio_W_Cast"),
    });

    // W provides shield and reduces damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGalioW::new());
}

fn cast_galio_e(
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
        hash: hash_bin("Galio_E_Cast"),
    });

    // E is a dash that knocks up
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell.clone(),
        move_type: DashMoveType::Pointer { max: 650.0 },
        damage: Some(DashDamage {
            radius_end: 150.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            },
        }),
        speed: 900.0,
    });
}

fn cast_galio_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Galio_R_Cast"),
    });

    // R is a large AoE knockback
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Galio_R_Hit")),
        }],
    });
}

fn on_galio_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
) {
    let source = trigger.source;
    if q_galio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGalioPassive::new());
}
