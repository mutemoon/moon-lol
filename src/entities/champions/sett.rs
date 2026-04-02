use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventSkillCast, Skill, SkillOf, SkillSlot, Skills, TargetDamage,
    TargetFilter,
};
use crate::entities::champion::Champion;
use crate::PassiveSkillOf;
use crate::DamageType;

const SETT_W_KEY: &str = "Characters/Sett/Spells/SettW/SettW";
const SETT_E_KEY: &str = "Characters/Sett/Spells/SettE/SettE";
const SETT_R_KEY: &str = "Characters/Sett/Spells/SettR/SettR";

#[derive(Default)]
pub struct PluginSett;

impl Plugin for PluginSett {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_sett_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sett"))]
#[reflect(Component)]
pub struct Sett;

fn on_sett_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_sett_q(&mut commands, entity),
        SkillSlot::W => cast_sett_w(&mut commands, entity),
        SkillSlot::E => cast_sett_e(&mut commands, entity),
        SkillSlot::R => cast_sett_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_sett_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Sett_Q_Cast"));
    // Q is a buff that enhances next 2 attacks with bonus damage
    // TODO: Add stacking buff for next 2 attacks
}

fn cast_sett_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Sett_W_Cast"));
    // W deals true damage in a cone and grants shield based on damage taken
    skill_damage(
        commands,
        entity,
        SETT_W_KEY,
        DamageShape::Sector { radius: 350.0, angle: 75.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::True,
        }],
        Some(hash_bin("Sett_W_Hit")),
    );
    // TODO: Track damage taken as Grit resource, convert to shield
}

fn cast_sett_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Sett_E_Cast"));
    // E is a两边拉扯 that damages and stuns enemies caught by both sides
    skill_damage(
        commands,
        entity,
        SETT_E_KEY,
        DamageShape::Sector { radius: 300.0, angle: 90.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Sett_E_Hit")),
    );
    // TODO: Pull enemies together and stun
}

fn cast_sett_r(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Sett_R_Cast"));
    // R is a dash that carries enemy to target location and deals damage
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: SETT_R_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 400.0 },
            damage: Some(crate::DashDamage {
                radius_end: 200.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 700.0,
        },
    );
    // TODO: Carry target champion and slam them on ground
}

fn add_skills(
    mut commands: Commands,
    q_sett: Query<Entity, (With<Sett>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_sett.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Sett/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Sett/Spells/SettPassiveAbility/SettPassive",
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
