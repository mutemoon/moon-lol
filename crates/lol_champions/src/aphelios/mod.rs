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

use crate::aphelios::buffs::{BuffApheliosCalibrum, BuffApheliosGravitum};

const APHELIOS_Q_KEY: &str = "Characters/Aphelios/Spells/ApheliosQ/ApheliosQ";
const APHELIOS_R_KEY: &str = "Characters/Aphelios/Spells/ApheliosR/ApheliosR";

#[derive(Default)]
pub struct PluginAphelios;

impl Plugin for PluginAphelios {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_aphelios_skill_cast);
        app.add_observer(on_aphelios_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aphelios"))]
#[reflect(Component)]
pub struct Aphelios;

fn on_aphelios_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_aphelios.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_aphelios_q(&mut commands, entity),
        SkillSlot::R => cast_aphelios_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_aphelios_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Aphelios_Q_Cast"));

    skill_damage(
        commands,
        entity,
        APHELIOS_Q_KEY,
        DamageShape::Sector {
            radius: 1450.0,
            angle: 25.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Aphelios_Q_Hit")),
    );
}

fn cast_aphelios_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Aphelios_R_Cast"));

    skill_damage(
        commands,
        entity,
        APHELIOS_R_KEY,
        DamageShape::Circle { radius: 1300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Aphelios_R_Hit")),
    );
}

fn on_aphelios_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
) {
    let source = trigger.source;
    if q_aphelios.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffApheliosCalibrum::new(70.0, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffApheliosGravitum::new(0.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_aphelios: Query<Entity, (With<Aphelios>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_aphelios.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Aphelios/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Aphelios/Spells/ApheliosPassive/ApheliosPassive",
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
