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
use crate::{BuffThreshE, BuffThreshQ, DamageType, PassiveSkillOf};

const THRESH_Q_KEY: &str = "Characters/Thresh/Spells/ThreshQ/ThreshQ";
const THRESH_W_KEY: &str = "Characters/Thresh/Spells/ThreshW/ThreshW";
const THRESH_E_KEY: &str = "Characters/Thresh/Spells/ThreshE/ThreshE";
const THRESH_R_KEY: &str = "Characters/Thresh/Spells/ThreshR/ThreshR";

#[derive(Default)]
pub struct PluginThresh;

impl Plugin for PluginThresh {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_thresh_skill_cast);
        app.add_observer(on_thresh_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Thresh"))]
#[reflect(Component)]
pub struct Thresh;

fn on_thresh_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_thresh: Query<(), With<Thresh>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_thresh.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_thresh_q(&mut commands, entity),
        SkillSlot::W => cast_thresh_w(&mut commands, entity),
        SkillSlot::E => cast_thresh_e(&mut commands, entity),
        SkillSlot::R => cast_thresh_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_thresh_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Thresh_Q_Cast"));

    skill_damage(
        commands,
        entity,
        THRESH_Q_KEY,
        DamageShape::Sector { radius: 1100.0, angle: 15.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Thresh_Q_Hit")),
    );
}

fn cast_thresh_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Thresh_W_Cast"));
}

fn cast_thresh_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Thresh_E_Cast"));

    skill_damage(
        commands,
        entity,
        THRESH_E_KEY,
        DamageShape::Sector { radius: 500.0, angle: 30.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Thresh_E_Hit")),
    );
}

fn cast_thresh_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Thresh_R_Cast"));

    skill_damage(
        commands,
        entity,
        THRESH_R_KEY,
        DamageShape::Circle { radius: 450.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Thresh_R_Hit")),
    );
}

fn on_thresh_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_thresh: Query<(), With<Thresh>>,
) {
    let source = trigger.source;
    if q_thresh.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands.entity(target).with_related::<BuffOf>(BuffThreshQ::new(1.0, 1.5));
    commands.entity(target).with_related::<BuffOf>(BuffThreshE::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_thresh: Query<Entity, (With<Thresh>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_thresh.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Thresh/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Thresh/Spells/ThreshPassive/ThreshPassive",
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
