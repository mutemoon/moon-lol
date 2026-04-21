pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
};

use crate::galio::buffs::{BuffGalioPassive, BuffGalioW};

const GALIO_Q_KEY: &str = "Characters/Galio/Spells/GalioQ/GalioQ";
#[allow(dead_code)]
const GALIO_W_KEY: &str = "Characters/Galio/Spells/GalioW/GalioW";
const GALIO_E_KEY: &str = "Characters/Galio/Spells/GalioE/GalioE";
const GALIO_R_KEY: &str = "Characters/Galio/Spells/GalioR/GalioR";

#[derive(Default)]
pub struct PluginGalio;

impl Plugin for PluginGalio {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_galio_skill_cast);
        app.add_observer(on_galio_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Galio"))]
#[reflect(Component)]
pub struct Galio;

fn on_galio_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_galio_q(&mut commands, entity),
        SkillSlot::W => cast_galio_w(&mut commands, entity),
        SkillSlot::E => cast_galio_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_galio_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_galio_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Galio_Q_Cast"));

    // Q is a tornado
    skill_damage(
        commands,
        entity,
        GALIO_Q_KEY,
        DamageShape::Sector {
            radius: 825.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Galio_Q_Hit")),
    );
}

fn cast_galio_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Galio_W_Cast"));

    // W provides shield and reduces damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGalioW::new());
}

fn cast_galio_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Galio_E_Cast"));

    // E is a dash that knocks up
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: GALIO_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 650.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 900.0,
        },
    );
}

fn cast_galio_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Galio_R_Cast"));

    // R is a large AoE knockback
    skill_damage(
        commands,
        entity,
        GALIO_R_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Galio_R_Hit")),
    );
}

fn on_galio_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
) {
    let source = trigger.source;
    if q_galio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGalioPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_galio: Query<Entity, (With<Galio>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_galio.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Galio/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Galio/Spells/GalioPassive/GalioPassive",
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
