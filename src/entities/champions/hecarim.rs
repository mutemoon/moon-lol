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
use crate::{BuffHecarimQ, BuffMoveSpeed, BuffOf, DamageType, PassiveSkillOf};

const HECARIM_Q_KEY: &str = "Characters/Hecarim/Spells/HecarimBlade/HecarimBlade";
const HECARIM_W_KEY: &str = "Characters/Hecarim/Spells/HecarimRampart/HecarimRampart";
const HECARIM_R_KEY: &str = "Characters/Hecarim/Spells/HecarimR/HecarimR";

// Hecarim Q parameters
const HECARIM_Q_MAX_STACKS: u8 = 4;
const HECARIM_Q_COOLDOWN_REDUCTION: f32 = 0.5; // 0.5s per stack
const HECARIM_Q_DAMAGE_BONUS: f32 = 0.1; // 10% bonus per stack

#[derive(Default)]
pub struct PluginHecarim;

impl Plugin for PluginHecarim {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_hecarim_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Hecarim"))]
#[reflect(Component)]
pub struct Hecarim;

fn on_hecarim_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_hecarim: Query<(), With<Hecarim>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_hecarim.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_hecarim_q(&mut commands, entity),
        SkillSlot::W => cast_hecarim_w(&mut commands, entity),
        SkillSlot::E => cast_hecarim_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_hecarim_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_hecarim_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Hecarim_Q_Cast"));
    skill_damage(
        commands,
        entity,
        HECARIM_Q_KEY,
        DamageShape::Circle { radius: 200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Hecarim_Q_Hit")),
    );
    // Q stacks - adds stacking buff for cooldown reduction and bonus damage
    commands.entity(entity).with_related::<BuffOf>(BuffHecarimQ::new(
        HECARIM_Q_MAX_STACKS,
        HECARIM_Q_COOLDOWN_REDUCTION,
        HECARIM_Q_DAMAGE_BONUS,
    ));
    debug!("{:?} 释放了 {} 技能，获得层数", entity, "Hecarim Q");
}

fn cast_hecarim_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Hecarim_W_Cast"));
    // W is AoE damage in area + healing based on damage dealt
    skill_damage(
        commands,
        entity,
        HECARIM_W_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Hecarim_W_Hit")),
    );
    // FUTURE: Add AoE healing buff
}

fn cast_hecarim_e(commands: &mut Commands, _q_transform: &Query<&Transform>, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Hecarim_E_Cast"));
    // E is movement speed boost + knockback on contact
    // Movement speed buff with knockback on collision
    commands.entity(entity).with_related::<BuffOf>(BuffMoveSpeed::new(0.75, 4.0));
}

fn cast_hecarim_r(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Hecarim_R_Cast"));
    // R is a long dash with fear
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: HECARIM_R_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 800.0 },
            damage: Some(crate::DashDamage {
                radius_end: 200.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1500.0,
        },
    );
    debug!("{:?} 的技能 {} 应对目标施加 {}",
        entity, "Hecarim R", "恐惧 DebuffFear");
}

fn add_skills(
    mut commands: Commands,
    q_hecarim: Query<Entity, (With<Hecarim>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_hecarim.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Hecarim/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Hecarim/Spells/HecarimPassiveAbility/HecarimPassive",
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
