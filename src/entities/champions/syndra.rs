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
use crate::{BuffSyndraE, DamageType, PassiveSkillOf};

const SYNDRA_Q_KEY: &str = "Characters/Syndra/Spells/SyndraQ/SyndraQ";
const SYNDRA_W_KEY: &str = "Characters/Syndra/Spells/SyndraW/SyndraW";
const SYNDRA_E_KEY: &str = "Characters/Syndra/Spells/SyndraE/SyndraE";
const SYNDRA_R_KEY: &str = "Characters/Syndra/Spells/SyndraR/SyndraR";

#[derive(Default)]
pub struct PluginSyndra;

impl Plugin for PluginSyndra {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_syndra_skill_cast);
        app.add_observer(on_syndra_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Syndra"))]
#[reflect(Component)]
pub struct Syndra;

fn on_syndra_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_syndra: Query<(), With<Syndra>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_syndra.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_syndra_q(&mut commands, entity),
        SkillSlot::W => cast_syndra_w(&mut commands, entity),
        SkillSlot::E => cast_syndra_e(&mut commands, entity),
        SkillSlot::R => cast_syndra_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_syndra_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Syndra_Q_Cast"));

    skill_damage(
        commands,
        entity,
        SYNDRA_Q_KEY,
        DamageShape::Circle { radius: 800.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_Q_Hit")),
    );
}

fn cast_syndra_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Syndra_W_Cast"));

    skill_damage(
        commands,
        entity,
        SYNDRA_W_KEY,
        DamageShape::Circle { radius: 950.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_W_Hit")),
    );
}

fn cast_syndra_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Syndra_E_Cast"));

    skill_damage(
        commands,
        entity,
        SYNDRA_E_KEY,
        DamageShape::Sector { radius: 800.0, angle: 45.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_E_Hit")),
    );
}

fn cast_syndra_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Syndra_R_Cast"));

    skill_damage(
        commands,
        entity,
        SYNDRA_R_KEY,
        DamageShape::Nearest { max_distance: 675.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_R_Hit")),
    );
}

fn on_syndra_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_syndra: Query<(), With<Syndra>>,
) {
    let source = trigger.source;
    if q_syndra.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands.entity(target).with_related::<BuffOf>(BuffSyndraE::new(0.75, 1.0));
}

fn add_skills(
    mut commands: Commands,
    q_syndra: Query<Entity, (With<Syndra>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_syndra.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Syndra/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Syndra/Spells/SyndraPassive/SyndraPassive",
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
