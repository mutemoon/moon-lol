use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    BuffOf, CoolDown, DamageShape, EventSkillCast, Skill, SkillOf, SkillSlot, Skills,
    TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffPantheonE, PassiveSkillOf};
use crate::DamageType;

const PANTHEON_Q_KEY: &str = "Characters/Pantheon/Spells/PantheonQ/PantheonQ";
const PANTHEON_W_KEY: &str = "Characters/Pantheon/Spells/PantheonW/PantheonW";
const PANTHEON_E_KEY: &str = "Characters/Pantheon/Spells/PantheonE/PantheonE";
const PANTHEON_R_KEY: &str = "Characters/Pantheon/Spells/PantheonR/PantheonR";

#[derive(Default)]
pub struct PluginPantheon;

impl Plugin for PluginPantheon {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_pantheon_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Pantheon"))]
#[reflect(Component)]
pub struct Pantheon;

fn on_pantheon_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pantheon.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_pantheon_q(&mut commands, entity, trigger.point),
        SkillSlot::W => cast_pantheon_w(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::E => cast_pantheon_e(&mut commands, entity),
        SkillSlot::R => cast_pantheon_r(&mut commands, entity, trigger.point),
        _ => {}
    }
}

fn cast_pantheon_q(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_Q_Cast"));
    // Q is a spear throw that can be held for more damage
    skill_damage(
        commands,
        entity,
        PANTHEON_Q_KEY,
        DamageShape::Sector { radius: 400.0, angle: 45.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pantheon_Q_Hit")),
    );
}

fn cast_pantheon_w(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_W_Cast"));
    // W is a dash to target that stuns
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: PANTHEON_W_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 200.0 },
            damage: Some(crate::DashDamage {
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
    debug!("{:?} 的技能 {} 应对目标施加 {}",
        entity, "Pantheon W", "眩晕 DebuffStun");
}

fn cast_pantheon_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_E_Cast"));
    // E is a shield block that deals damage in a cone when released
    skill_damage(
        commands,
        entity,
        PANTHEON_E_KEY,
        DamageShape::Sector { radius: 300.0, angle: 90.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pantheon_E_Hit")),
    );
    commands.entity(entity).with_related::<BuffOf>(BuffPantheonE::new(Vec2::ZERO, 1.5));
}

fn cast_pantheon_r(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_R_Cast"));
    // R is a long-range leap that damages enemies in area
    skill_damage(
        commands,
        entity,
        PANTHEON_R_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pantheon_R_Hit")),
    );
    // FUTURE: Long range targeting
}

fn add_skills(
    mut commands: Commands,
    q_pantheon: Query<Entity, (With<Pantheon>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_pantheon.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Pantheon/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Pantheon/Spells/PantheonPassiveAbility/PantheonPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            commands.entity(entity).with_related::<SkillOf>((
                Skill::new(skill_slot_from_index(index), skill),
                CoolDown::default(),
            ));
        }
    }
}
