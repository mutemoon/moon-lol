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

use crate::illaoi::buffs::BuffIllaoiPassive;

#[derive(Default)]
pub struct PluginIllaoi;

impl Plugin for PluginIllaoi {
    fn build(&self, app: &mut App) {
        app.add_observer(on_illaoi_skill_cast);
        app.add_observer(on_illaoi_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Illaoi"))]
#[reflect(Component)]
pub struct Illaoi;

fn on_illaoi_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_illaoi.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_illaoi_q(&mut commands, entity),
        SkillSlot::W => cast_illaoi_w(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::E => cast_illaoi_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_illaoi_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_illaoi_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Illaoi_Q_Cast"),
    });

    // Q enhances tentacle damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffIllaoiPassive::new());
}

fn cast_illaoi_w(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Illaoi_W_Cast"),
    });

    // W is a dash to target
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 225.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            },
        }),
        speed: 1000.0,
    });
}

fn cast_illaoi_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Illaoi_E_Cast"),
    });

    // E pulls soul
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 950.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Illaoi_E_Hit")),
        }],
    });
}

fn cast_illaoi_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Illaoi_R_Cast"),
    });

    // R is AoE damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Illaoi_R_Hit")),
        }],
    });
}

fn on_illaoi_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
) {
    let source = trigger.source;
    if q_illaoi.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffIllaoiPassive::new());
}
