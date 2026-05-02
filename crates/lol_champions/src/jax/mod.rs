pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::{BuffEmpoweredAttack, BuffResist};
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::jax::buffs::BuffJaxE;

// Jax E parameters
const JAX_E_DURATION: f32 = 2.0; // 2 seconds
const JAX_E_DODGE_CHANCE: f32 = 0.7; // 70% dodge chance
const JAX_E_AOE_DODGE_CHANCE: f32 = 0.5; // 50% AoE dodge chance

#[derive(Default)]
pub struct PluginJax;

impl Plugin for PluginJax {
    fn build(&self, app: &mut App) {
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

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_jax_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::W => cast_jax_w(&mut commands, entity),
        SkillSlot::E => cast_jax_e(&mut commands, entity),
        SkillSlot::R => cast_jax_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_jax_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jax_Q_Cast"),
    });
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 300.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            },
        }),
        speed: 1000.0,
    });
}

fn cast_jax_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jax_W_Cast"),
    });
    // W resets attack timer and enhances next attack
    commands.trigger(CommandAttackReset { entity });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffEmpoweredAttack::new(50.0, 1));
}

fn cast_jax_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jax_E_Cast"),
    });

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

fn cast_jax_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jax_R_Cast"),
    });
    // R is a self-cast that deals AoE damage and grants armor/mr
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Jax_R_Hit")),
        }],
    });
    // Armor/mr buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffResist::new(30.0, 30.0, 8.0));
}
