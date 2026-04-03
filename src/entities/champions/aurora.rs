use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill,
    SkillOf, SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffAuroraPassive, BuffAuroraR, BuffOf, DamageType, DebuffSlow, PassiveSkillOf};

const AURORA_Q_KEY: &str = "Characters/Aurora/Spells/AuroraQ/AuroraQ";
const AURORA_W_KEY: &str = "Characters/Aurora/Spells/AuroraW/AuroraW";
const AURORA_E_KEY: &str = "Characters/Aurora/Spells/AuroraE/AuroraE";
const AURORA_R_KEY: &str = "Characters/Aurora/Spells/AuroraR/AuroraR";

#[derive(Default)]
pub struct PluginAurora;

impl Plugin for PluginAurora {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_aurora_skill_cast);
        app.add_observer(on_aurora_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aurora"))]
#[reflect(Component)]
pub struct Aurora;

fn on_aurora_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_aurora.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_aurora_q(&mut commands, entity),
        SkillSlot::W => cast_aurora_w(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::E => cast_aurora_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_aurora_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_aurora_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_Q_Cast"));

    // Q is a projectile
    skill_damage(
        commands,
        entity,
        AURORA_Q_KEY,
        DamageShape::Sector { radius: 850.0, angle: 30.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Aurora_Q_Hit")),
    );

    // Apply slow
    commands.entity(entity).with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}

fn cast_aurora_w(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_W_Cast"));

    // W creates a portal - dash to it
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: AURORA_W_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 600.0 },
            damage: Some(crate::DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        },
    );
}

fn cast_aurora_e(commands: &mut Commands, _q_transform: &Query<&Transform>, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_E_Cast"));

    // E creates a path
    skill_damage(
        commands,
        entity,
        AURORA_E_KEY,
        DamageShape::Sector { radius: 700.0, angle: 30.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Aurora_E_Hit")),
    );
}

fn cast_aurora_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_R_Cast"));

    // R is area damage and freeze
    skill_damage(
        commands,
        entity,
        AURORA_R_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Aurora_R_Hit")),
    );

    commands.entity(entity).with_related::<BuffOf>(BuffAuroraR::new());
}

fn on_aurora_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
) {
    let source = trigger.source;
    if q_aurora.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Passive slow
    commands.entity(target).with_related::<BuffOf>(BuffAuroraPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_aurora: Query<Entity, (With<Aurora>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_aurora.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Aurora/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Aurora/Spells/AuroraPassive/AuroraPassive",
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
