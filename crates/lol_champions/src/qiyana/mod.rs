pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::qiyana::buffs::BuffQiyanaW;

const QIYANA_Q_KEY: &str = "Characters/Qiyana/Spells/QiyanaQ/QiyanaQ";
#[allow(dead_code)]
const QIYANA_W_KEY: &str = "Characters/Qiyana/Spells/QiyanaW/QiyanaW";
#[allow(dead_code)]
const QIYANA_E_KEY: &str = "Characters/Qiyana/Spells/QiyanaE/QiyanaE";
const QIYANA_R_KEY: &str = "Characters/Qiyana/Spells/QiyanaR/QiyanaR";

#[derive(Default)]
pub struct PluginQiyana;

impl Plugin for PluginQiyana {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_qiyana_skill_cast);
        app.add_observer(on_qiyana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Qiyana"))]
#[reflect(Component)]
pub struct Qiyana;

fn on_qiyana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_qiyana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_qiyana_q(&mut commands, entity),
        SkillSlot::W => cast_qiyana_w(&mut commands, entity),
        SkillSlot::E => cast_qiyana_e(&mut commands, entity),
        SkillSlot::R => cast_qiyana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_qiyana_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Qiyana_Q_Cast"));

    // Q is edge of Ixtal - damage
    skill_damage(
        commands,
        entity,
        QIYANA_Q_KEY,
        DamageShape::Sector {
            radius: 525.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Qiyana_Q_Hit")),
    );
}

fn cast_qiyana_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Qiyana_W_Cast"));

    // W is elemental wrath - dash and element buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffQiyanaW::new(0.2, 5.0));
}

fn cast_qiyana_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Qiyana_E_Cast"));

    // E is terrashape - dash through terrain
}

fn cast_qiyana_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Qiyana_R_Cast"));

    // R is audacity/supreme display - large AoE knockup
    skill_damage(
        commands,
        entity,
        QIYANA_R_KEY,
        DamageShape::Circle { radius: 875.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Qiyana_R_Hit")),
    );
}

fn on_qiyana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
) {
    let source = trigger.source;
    if q_qiyana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W grass gives movespeed
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffQiyanaW::new(0.2, 5.0));
}

fn add_skills(
    mut commands: Commands,
    q_qiyana: Query<Entity, (With<Qiyana>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_qiyana.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Qiyana/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Qiyana/Spells/QiyanaPassive/QiyanaPassive",
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
