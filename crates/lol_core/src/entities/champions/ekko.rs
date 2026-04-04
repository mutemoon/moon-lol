use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::action::dash::{ActionDash, DashDamage, DashMoveType};
use crate::base::buff::BuffOf;
use crate::buffs::cc_debuffs::DebuffSlow;
use crate::buffs::ekko_buffs::BuffEkkoPassive;
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const EKKO_Q_KEY: &str = "Characters/Ekko/Spells/EkkoQ/EkkoQ";
const EKKO_W_KEY: &str = "Characters/Ekko/Spells/EkkoW/EkkoW";
const EKKO_E_KEY: &str = "Characters/Ekko/Spells/EkkoE/EkkoE";
const EKKO_R_KEY: &str = "Characters/Ekko/Spells/EkkoR/EkkoR";

#[derive(Default)]
pub struct PluginEkko;

impl Plugin for PluginEkko {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_ekko_skill_cast);
        app.add_observer(on_ekko_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ekko"))]
#[reflect(Component)]
pub struct Ekko;

fn on_ekko_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ekko.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_ekko_q(&mut commands, entity),
        SkillSlot::W => cast_ekko_w(&mut commands, entity),
        SkillSlot::E => cast_ekko_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_ekko_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_ekko_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_Q_Cast"));

    // Q is a projectile that slows
    skill_damage(
        commands,
        entity,
        EKKO_Q_KEY,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ekko_Q_Hit")),
    );
}

fn cast_ekko_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_W_Cast"));

    // W creates a zone that slows
    skill_damage(
        commands,
        entity,
        EKKO_W_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ekko_W_Hit")),
    );
}

fn cast_ekko_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_E_Cast"));

    // E is a dash that enhances next attack
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: EKKO_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 325.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 1000.0,
        },
    );
}

fn cast_ekko_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_R_Cast"));

    // R is an AoE damage around previous position
    skill_damage(
        commands,
        entity,
        EKKO_R_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ekko_R_Hit")),
    );
}

fn on_ekko_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
) {
    let source = trigger.source;
    if q_ekko.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffEkkoPassive::new());
    // Q and W slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_ekko: Query<Entity, (With<Ekko>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_ekko.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Ekko/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Ekko/Spells/EkkoPassive/EkkoPassive",
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
