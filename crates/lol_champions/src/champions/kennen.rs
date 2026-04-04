use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::buffs::kennen_buffs::{BuffKennenE, BuffKennenMarkOfStorm, BuffKennenR};

const KENNEN_Q_KEY: &str = "Characters/Kennen/Spells/KennenQ/KennenQ";
const KENNEN_W_KEY: &str = "Characters/Kennen/Spells/KennenW/KennenW";
#[allow(dead_code)]
const KENNEN_E_KEY: &str = "Characters/Kennen/Spells/KennenE/KennenE";
const KENNEN_R_KEY: &str = "Characters/Kennen/Spells/KennenR/KennR";

#[derive(Default)]
pub struct PluginKennen;

impl Plugin for PluginKennen {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kennen_skill_cast);
        app.add_observer(on_kennen_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kennen"))]
#[reflect(Component)]
pub struct Kennen;

fn on_kennen_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kennen.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kennen_q(&mut commands, entity),
        SkillSlot::W => cast_kennen_w(&mut commands, entity),
        SkillSlot::E => cast_kennen_e(&mut commands, entity),
        SkillSlot::R => cast_kennen_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_kennen_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kennen_Q_Cast"));

    // Q is a shuriken that applies mark
    skill_damage(
        commands,
        entity,
        KENNEN_Q_KEY,
        DamageShape::Sector {
            radius: 1050.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kennen_Q_Hit")),
    );
}

fn cast_kennen_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kennen_W_Cast"));

    // W deals damage to marked enemies
    skill_damage(
        commands,
        entity,
        KENNEN_W_KEY,
        DamageShape::Circle { radius: 775.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kennen_W_Hit")),
    );
}

fn cast_kennen_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kennen_E_Cast"));

    // E grants movespeed and immunity
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKennenE::new(1.0, 0.6, 2.0));
}

fn cast_kennen_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kennen_R_Cast"));

    // R summons storm that damages and applies marks
    skill_damage(
        commands,
        entity,
        KENNEN_R_KEY,
        DamageShape::Circle { radius: 550.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kennen_R_Hit")),
    );

    // R grants armor/mr
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKennenR::new(40.0, 40.0, 3.0));
}

fn on_kennen_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
) {
    let source = trigger.source;
    if q_kennen.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply mark of the storm (3 marks = stun)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKennenMarkOfStorm::new(1, 8.0));
}

fn add_skills(
    mut commands: Commands,
    q_kennen: Query<Entity, (With<Kennen>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kennen.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kennen/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kennen/Spells/KennenPassive/KennenPassive",
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
