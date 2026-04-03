use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::rammus_buffs::{BuffRammusE, BuffRammusQ, BuffRammusR};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const RAMMUS_Q_KEY: &str = "Characters/Rammus/Spells/RammusQ/RammusQ";
#[allow(dead_code)]
const RAMMUS_W_KEY: &str = "Characters/Rammus/Spells/RammusW/RammusW";
const RAMMUS_E_KEY: &str = "Characters/Rammus/Spells/RammusE/RammusE";
const RAMMUS_R_KEY: &str = "Characters/Rammus/Spells/RammusR/RammusR";

#[derive(Default)]
pub struct PluginRammus;

impl Plugin for PluginRammus {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_rammus_skill_cast);
        app.add_observer(on_rammus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rammus"))]
#[reflect(Component)]
pub struct Rammus;

fn on_rammus_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rammus.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rammus_q(&mut commands, entity),
        SkillSlot::W => cast_rammus_w(&mut commands, entity),
        SkillSlot::E => cast_rammus_e(&mut commands, entity),
        SkillSlot::R => cast_rammus_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_rammus_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Rammus_Q_Cast"));

    // Q is powerball - damage and knockup
    skill_damage(
        commands,
        entity,
        RAMMUS_Q_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rammus_Q_Hit")),
    );
}

fn cast_rammus_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Rammus_W_Cast"));

    // W is defensive ball curl - damage reflection
}

fn cast_rammus_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Rammus_E_Cast"));

    // E is frencying taunt - taunt enemies
    skill_damage(
        commands,
        entity,
        RAMMUS_E_KEY,
        DamageShape::Circle { radius: 325.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rammus_E_Hit")),
    );
}

fn cast_rammus_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Rammus_R_Cast"));

    // R is soaring slam - AoE damage and slow
    skill_damage(
        commands,
        entity,
        RAMMUS_R_KEY,
        DamageShape::Circle { radius: 800.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rammus_R_Hit")),
    );
}

fn on_rammus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
) {
    let source = trigger.source;
    if q_rammus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRammusQ::new(0.8, 1.0));
    // E taunts
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRammusE::new(2.0, 2.5));
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRammusR::new(0.5, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_rammus: Query<Entity, (With<Rammus>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_rammus.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Rammus/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Rammus/Spells/RammusPassive/RammusPassive",
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
