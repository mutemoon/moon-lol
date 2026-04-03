use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::ezreal_buffs::BuffEzrealPassive;
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::action::dash::{ActionDash, DashDamage, DashMoveType};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const EZREAL_Q_KEY: &str = "Characters/Ezreal/Spells/EzrealQ/EzrealQ";
const EZREAL_W_KEY: &str = "Characters/Ezreal/Spells/EzrealW/EzrealW";
const EZREAL_E_KEY: &str = "Characters/Ezreal/Spells/EzrealE/EzrealE";
const EZREAL_R_KEY: &str = "Characters/Ezreal/Spells/EzrealR/EzrealR";

#[derive(Default)]
pub struct PluginEzreal;

impl Plugin for PluginEzreal {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_ezreal_skill_cast);
        app.add_observer(on_ezreal_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ezreal"))]
#[reflect(Component)]
pub struct Ezreal;

fn on_ezreal_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ezreal.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_ezreal_q(&mut commands, entity),
        SkillSlot::W => cast_ezreal_w(&mut commands, entity),
        SkillSlot::E => cast_ezreal_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_ezreal_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_ezreal_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_Q_Cast"));

    // Q is a long range skillshot
    skill_damage(
        commands,
        entity,
        EZREAL_Q_KEY,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Ezreal_Q_Hit")),
    );
}

fn cast_ezreal_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_W_Cast"));

    // W marks target
    skill_damage(
        commands,
        entity,
        EZREAL_W_KEY,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ezreal_W_Hit")),
    );
}

fn cast_ezreal_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_E_Cast"));

    // E is a blink/dash
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: EZREAL_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 475.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        },
    );
}

fn cast_ezreal_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_R_Cast"));

    // R is global AoE
    skill_damage(
        commands,
        entity,
        EZREAL_R_KEY,
        DamageShape::Sector {
            radius: 20000.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ezreal_R_Hit")),
    );
}

fn on_ezreal_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
) {
    let source = trigger.source;
    if q_ezreal.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffEzrealPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_ezreal: Query<Entity, (With<Ezreal>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_ezreal.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Ezreal/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Ezreal/Spells/EzrealPassive/EzrealPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
