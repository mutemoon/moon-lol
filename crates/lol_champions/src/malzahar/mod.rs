pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSilence;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::malzahar::buffs::BuffMalzaharE;

const MALZAHAR_Q_KEY: &str = "Characters/Malzahar/Spells/MalzaharQ/MalzaharQ";
const MALZAHAR_W_KEY: &str = "Characters/Malzahar/Spells/MalzaharW/MalzaharW";
const MALZAHAR_E_KEY: &str = "Characters/Malzahar/Spells/MalzaharE/MalzaharE";
const MALZAHAR_R_KEY: &str = "Characters/Malzahar/Spells/MalzaharR/MalzaharR";

#[derive(Default)]
pub struct PluginMalzahar;

impl Plugin for PluginMalzahar {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_malzahar_skill_cast);
        app.add_observer(on_malzahar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Malzahar"))]
#[reflect(Component)]
pub struct Malzahar;

fn on_malzahar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_malzahar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_malzahar_q(&mut commands, entity),
        SkillSlot::W => cast_malzahar_w(&mut commands, entity),
        SkillSlot::E => cast_malzahar_e(&mut commands, entity),
        SkillSlot::R => cast_malzahar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_malzahar_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Malzahar_Q_Cast"));

    // Q opens void gates and silences
    skill_damage(
        commands,
        entity,
        MALZAHAR_Q_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Malzahar_Q_Hit")),
    );
}

fn cast_malzahar_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Malzahar_W_Cast"));

    // W summons voidlings
    skill_damage(
        commands,
        entity,
        MALZAHAR_W_KEY,
        DamageShape::Circle { radius: 150.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Malzahar_W_Hit")),
    );
}

fn cast_malzahar_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Malzahar_E_Cast"));

    // E infects target
    skill_damage(
        commands,
        entity,
        MALZAHAR_E_KEY,
        DamageShape::Circle { radius: 650.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Malzahar_E_Hit")),
    );
}

fn cast_malzahar_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Malzahar_R_Cast"));

    // R suppresses target
    skill_damage(
        commands,
        entity,
        MALZAHAR_R_KEY,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Malzahar_R_Hit")),
    );
}

fn on_malzahar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
) {
    let source = trigger.source;
    if q_malzahar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q silences
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSilence::new(1.5));

    // E applies infection
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMalzaharE::new(50.0, 4.0));
}

fn add_skills(
    mut commands: Commands,
    q_malzahar: Query<Entity, (With<Malzahar>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_malzahar.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Malzahar/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Malzahar/Spells/MalzaharPassive/MalzaharPassive",
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
