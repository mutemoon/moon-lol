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
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
};

use crate::kindred::buffs::{BuffKindredE, BuffKindredW};

const KINDRED_Q_KEY: &str = "Characters/Kindred/Spells/KindredQ/KindredQ";
const KINDRED_W_KEY: &str = "Characters/Kindred/Spells/KindredW/KindredW";
const KINDRED_E_KEY: &str = "Characters/Kindred/Spells/KindredE/KindredE";
const KINDRED_R_KEY: &str = "Characters/Kindred/Spells/KindredR/KindredR";

#[derive(Default)]
pub struct PluginKindred;

impl Plugin for PluginKindred {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kindred_skill_cast);
        app.add_observer(on_kindred_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kindred"))]
#[reflect(Component)]
pub struct Kindred;

fn on_kindred_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kindred.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kindred_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_kindred_w(&mut commands, entity),
        SkillSlot::E => cast_kindred_e(&mut commands, entity),
        SkillSlot::R => cast_kindred_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_kindred_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_Q_Cast"));

    // Q is a dash that shoots arrows
    skill_damage(
        commands,
        entity,
        KINDRED_Q_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kindred_Q_Hit")),
    );
}

fn cast_kindred_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_W_Cast"));

    // W marks an area where Wolf attacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKindredW::new(50.0, 8.5));

    skill_damage(
        commands,
        entity,
        KINDRED_W_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kindred_W_Hit")),
    );
}

fn cast_kindred_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_E_Cast"));

    // E marks and slows, 3 marks = Wolf attacks
    skill_damage(
        commands,
        entity,
        KINDRED_E_KEY,
        DamageShape::Sector {
            radius: 500.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kindred_E_Hit")),
    );
}

fn cast_kindred_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_R_Cast"));

    // R creates a protective zone
    skill_damage(
        commands,
        entity,
        KINDRED_R_KEY,
        DamageShape::Circle { radius: 535.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kindred_R_Hit")),
    );
}

fn on_kindred_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
) {
    let source = trigger.source;
    if q_kindred.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply slow and mark
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKindredE::new(1, 0.3, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_kindred: Query<Entity, (With<Kindred>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kindred.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kindred/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kindred/Spells/KindredPassive/KindredPassive",
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
