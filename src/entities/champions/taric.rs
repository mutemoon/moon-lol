use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::taric_buffs::BuffTaricE;
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

#[allow(dead_code)]
const TARIC_Q_KEY: &str = "Characters/Taric/Spells/TaricQ/TaricQ";
#[allow(dead_code)]
const TARIC_W_KEY: &str = "Characters/Taric/Spells/TaricW/TaricW";
const TARIC_E_KEY: &str = "Characters/Taric/Spells/TaricE/TaricE";
#[allow(dead_code)]
const TARIC_R_KEY: &str = "Characters/Taric/Spells/TaricR/TaricR";

#[derive(Default)]
pub struct PluginTaric;

impl Plugin for PluginTaric {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_taric_skill_cast);
        app.add_observer(on_taric_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Taric"))]
#[reflect(Component)]
pub struct Taric;

fn on_taric_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taric: Query<(), With<Taric>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_taric.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_taric_q(&mut commands, entity),
        SkillSlot::W => cast_taric_w(&mut commands, entity),
        SkillSlot::E => cast_taric_e(&mut commands, entity),
        SkillSlot::R => cast_taric_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_taric_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Taric_Q_Cast"));
}

fn cast_taric_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Taric_W_Cast"));
}

fn cast_taric_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Taric_E_Cast"));

    skill_damage(
        commands,
        entity,
        TARIC_E_KEY,
        DamageShape::Sector {
            radius: 600.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Taric_E_Hit")),
    );
}

fn cast_taric_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Taric_R_Cast"));
}

fn on_taric_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_taric: Query<(), With<Taric>>,
) {
    let source = trigger.source;
    if q_taric.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTaricE::new(1.0, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_taric: Query<Entity, (With<Taric>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_taric.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Taric/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Taric/Spells/TaricPassive/TaricPassive",
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
