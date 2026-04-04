use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::{BuffEmpoweredAttack, BuffResist};
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_dash, skill_slot_from_index,
    spawn_skill_particle, CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot,
    Skills,
};

use crate::buffs::jax_buffs::BuffJaxE;

const JAX_Q_KEY: &str = "Characters/Jax/Spells/JaxLeapStrike/JaxLeapStrike";
const JAX_R_KEY: &str = "Characters/Jax/Spells/JaxR/JaxR";

// Jax E parameters
const JAX_E_DURATION: f32 = 2.0; // 2 seconds
const JAX_E_DODGE_CHANCE: f32 = 0.7; // 70% dodge chance
const JAX_E_AOE_DODGE_CHANCE: f32 = 0.5; // 50% AoE dodge chance

#[derive(Default)]
pub struct PluginJax;

impl Plugin for PluginJax {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_jax_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jax"))]
#[reflect(Component)]
pub struct Jax;

fn on_jax_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jax: Query<(), With<Jax>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jax.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_jax_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_jax_w(&mut commands, entity),
        SkillSlot::E => cast_jax_e(&mut commands, entity),
        SkillSlot::R => cast_jax_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_jax_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Jax_Q_Cast"));
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: JAX_Q_KEY.into(),
            move_type: DashMoveType::Pointer { max: 300.0 },
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

fn cast_jax_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Jax_W_Cast"));
    // W resets attack timer and enhances next attack
    reset_skill_attack(commands, entity);
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffEmpoweredAttack::new(50.0, 1));
}

fn cast_jax_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Jax_E_Cast"));

    // E provides dodge buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffJaxE::new(
            JAX_E_DURATION,
            JAX_E_DODGE_CHANCE,
            JAX_E_AOE_DODGE_CHANCE,
        ));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 闪避几率",
        entity,
        "Jax E",
        (JAX_E_DODGE_CHANCE * 100.0) as i32
    );
}

fn cast_jax_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Jax_R_Cast"));
    // R is a self-cast that deals AoE damage and grants armor/mr
    skill_damage(
        commands,
        entity,
        JAX_R_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Jax_R_Hit")),
    );
    // Armor/mr buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffResist::new(30.0, 30.0, 8.0));
}

fn add_skills(
    mut commands: Commands,
    q_jax: Query<Entity, (With<Jax>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_jax.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Jax/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Jax/Spells/JaxPassiveAbility/JaxPassive",
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
