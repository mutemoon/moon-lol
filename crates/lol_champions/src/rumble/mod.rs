pub mod buffs;

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

use crate::rumble::buffs::BuffRumbleW;

const RUMBLE_Q_KEY: &str = "Characters/Rumble/Spells/RumbleQ/RumbleQ";
#[allow(dead_code)]
const RUMBLE_W_KEY: &str = "Characters/Rumble/Spells/RumbleW/RumbleW";
const RUMBLE_E_KEY: &str = "Characters/Rumble/Spells/RumbleE/RumbleE";
const RUMBLE_R_KEY: &str = "Characters/Rumble/Spells/RumbleR/RumbleR";

#[derive(Default)]
pub struct PluginRumble;

impl Plugin for PluginRumble {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_rumble_skill_cast);
        app.add_observer(on_rumble_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rumble"))]
#[reflect(Component)]
pub struct Rumble;

fn on_rumble_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rumble.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rumble_q(&mut commands, entity),
        SkillSlot::W => cast_rumble_w(&mut commands, entity),
        SkillSlot::E => cast_rumble_e(&mut commands, entity),
        SkillSlot::R => cast_rumble_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_rumble_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Rumble_Q_Cast"));

    // Q is electro harpoon - damage over time
    skill_damage(
        commands,
        entity,
        RUMBLE_Q_KEY,
        DamageShape::Sector {
            radius: 600.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rumble_Q_Hit")),
    );
}

fn cast_rumble_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Rumble_W_Cast"));

    // W is scrap shield - shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRumbleW::new(50.0, 1.5));
}

fn cast_rumble_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Rumble_E_Cast"));

    // E is electro harpoon - damage
    skill_damage(
        commands,
        entity,
        RUMBLE_E_KEY,
        DamageShape::Sector {
            radius: 850.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rumble_E_Hit")),
    );
}

fn cast_rumble_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Rumble_R_Cast"));

    // R is electro fire - large AoE damage
    skill_damage(
        commands,
        entity,
        RUMBLE_R_KEY,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rumble_R_Hit")),
    );
}

fn on_rumble_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
) {
    let source = trigger.source;
    if q_rumble.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRumbleW::new(50.0, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_rumble: Query<Entity, (With<Rumble>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_rumble.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Rumble/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Rumble/Spells/RumblePassive/RumblePassive",
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
