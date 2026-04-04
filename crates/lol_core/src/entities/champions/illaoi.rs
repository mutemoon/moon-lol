use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::illaoi_buffs::BuffIllaoiPassive;
use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::action::dash::{ActionDash, DashDamage, DashMoveType};
use crate::base::buff::BuffOf;
use crate::damage::{DamageType, EventDamageCreate};
use crate::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

#[allow(dead_code)]
const ILLAAI_Q_KEY: &str = "Characters/Illaoi/Spells/IllaoiQ/IllaoiQ";
const ILLAAI_W_KEY: &str = "Characters/Illaoi/Spells/IllaoiW/IllaoiW";
const ILLAAI_E_KEY: &str = "Characters/Illaoi/Spells/IllaoiE/IllaoiE";
const ILLAAI_R_KEY: &str = "Characters/Illaoi/Spells/IllaoiR/IllaoiR";

#[derive(Default)]
pub struct PluginIllaoi;

impl Plugin for PluginIllaoi {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_illaoi_skill_cast);
        app.add_observer(on_illaoi_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Illaoi"))]
#[reflect(Component)]
pub struct Illaoi;

fn on_illaoi_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_illaoi.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_illaoi_q(&mut commands, entity),
        SkillSlot::W => cast_illaoi_w(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::E => cast_illaoi_e(&mut commands, entity),
        SkillSlot::R => cast_illaoi_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_illaoi_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Illaoi_Q_Cast"));

    // Q enhances tentacle damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffIllaoiPassive::new());
}

fn cast_illaoi_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Illaoi_W_Cast"));

    // W is a dash to target
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: ILLAAI_W_KEY.into(),
            move_type: DashMoveType::Pointer { max: 225.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1000.0,
        },
    );
}

fn cast_illaoi_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Illaoi_E_Cast"));

    // E pulls soul
    skill_damage(
        commands,
        entity,
        ILLAAI_E_KEY,
        DamageShape::Sector {
            radius: 950.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Illaoi_E_Hit")),
    );
}

fn cast_illaoi_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Illaoi_R_Cast"));

    // R is AoE damage
    skill_damage(
        commands,
        entity,
        ILLAAI_R_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Illaoi_R_Hit")),
    );
}

fn on_illaoi_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
) {
    let source = trigger.source;
    if q_illaoi.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffIllaoiPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_illaoi: Query<Entity, (With<Illaoi>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_illaoi.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Illaoi/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Illaoi/Spells/IllaoiPassive/IllaoiPassive",
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
