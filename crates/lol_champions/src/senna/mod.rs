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

use crate::senna::buffs::BuffSennaW;

const SENNA_Q_KEY: &str = "Characters/Senna/Spells/SennaQ/SennaQ";
const SENNA_W_KEY: &str = "Characters/Senna/Spells/SennaW/SennaW";
#[allow(dead_code)]
const SENNA_E_KEY: &str = "Characters/Senna/Spells/SennaE/SennaE";
const SENNA_R_KEY: &str = "Characters/Senna/Spells/SennaR/SennaR";

#[derive(Default)]
pub struct PluginSenna;

impl Plugin for PluginSenna {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_senna_skill_cast);
        app.add_observer(on_senna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Senna"))]
#[reflect(Component)]
pub struct Senna;

fn on_senna_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_senna.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_senna_q(&mut commands, entity),
        SkillSlot::W => cast_senna_w(&mut commands, entity),
        SkillSlot::E => cast_senna_e(&mut commands, entity),
        SkillSlot::R => cast_senna_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_senna_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Senna_Q_Cast"));

    // Q is duskblade of shadow - damage
    skill_damage(
        commands,
        entity,
        SENNA_Q_KEY,
        DamageShape::Sector {
            radius: 600.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Senna_Q_Hit")),
    );
}

fn cast_senna_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Senna_W_Cast"));

    // W is last embrace - root
    skill_damage(
        commands,
        entity,
        SENNA_W_KEY,
        DamageShape::Circle { radius: 1000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Senna_W_Hit")),
    );
}

fn cast_senna_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Senna_E_Cast"));

    // E is curtain of darkness - camouflage
}

fn cast_senna_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Senna_R_Cast"));

    // R is pierce the veil - AoE damage and shield
    skill_damage(
        commands,
        entity,
        SENNA_R_KEY,
        DamageShape::Sector {
            radius: 2500.0,
            angle: 50.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Senna_R_Hit")),
    );
}

fn on_senna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
) {
    let source = trigger.source;
    if q_senna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSennaW::new(1.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_senna: Query<Entity, (With<Senna>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_senna.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Senna/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Senna/Spells/SennaPassive/SennaPassive",
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
