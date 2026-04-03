use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, BuffOf,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill, SkillOf, SkillSlot, Skills,
    TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffSkarnerR, DamageType, PassiveSkillOf};

const SKARNER_Q_KEY: &str = "Characters/Skarner/Spells/SkarnerQ/SkarnerQ";
#[allow(dead_code)]
const SKARNER_W_KEY: &str = "Characters/Skarner/Spells/SkarnerW/SkarnerW";
const SKARNER_E_KEY: &str = "Characters/Skarner/Spells/SkarnerE/SkarnerE";
const SKARNER_R_KEY: &str = "Characters/Skarner/Spells/SkarnerR/SkarnerR";

#[derive(Default)]
pub struct PluginSkarner;

impl Plugin for PluginSkarner {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_skarner_skill_cast);
        app.add_observer(on_skarner_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Skarner"))]
#[reflect(Component)]
pub struct Skarner;

fn on_skarner_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_skarner.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_skarner_q(&mut commands, entity),
        SkillSlot::W => cast_skarner_w(&mut commands, entity),
        SkillSlot::E => cast_skarner_e(&mut commands, entity),
        SkillSlot::R => cast_skarner_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_skarner_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Skarner_Q_Cast"));

    // Q is crystal slash - damage
    skill_damage(
        commands,
        entity,
        SKARNER_Q_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Skarner_Q_Hit")),
    );
}

fn cast_skarner_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Skarner_W_Cast"));

    // W is crystalline exoskeleton - shield
}

fn cast_skarner_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Skarner_E_Cast"));

    // E is fracture - damage
    skill_damage(
        commands,
        entity,
        SKARNER_E_KEY,
        DamageShape::Sector { radius: 600.0, angle: 30.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Skarner_E_Hit")),
    );
}

fn cast_skarner_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Skarner_R_Cast"));

    // R is impale - stun and damage
    skill_damage(
        commands,
        entity,
        SKARNER_R_KEY,
        DamageShape::Nearest { max_distance: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Skarner_R_Hit")),
    );
}

fn on_skarner_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
) {
    let source = trigger.source;
    if q_skarner.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // R stuns
    commands.entity(target).with_related::<BuffOf>(BuffSkarnerR::new(1.0, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_skarner: Query<Entity, (With<Skarner>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_skarner.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Skarner/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Skarner/Spells/SkarnerPassive/SkarnerPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
