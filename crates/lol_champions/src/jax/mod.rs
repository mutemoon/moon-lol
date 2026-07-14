pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffResist;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter};
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
        app.add_observer(on_jax_q);
        app.add_observer(on_jax_w);
        app.add_observer(on_jax_e);
        app.add_observer(on_jax_r);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jax"))]
#[reflect(Component)]
pub struct Jax;

fn on_jax_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jax: Query<(), With<Jax>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jax.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 300.0 },
        speed: 1000.0,
    });
}

fn on_jax_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jax: Query<(), With<Jax>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jax.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W 组合：攻击重置 + 强化下次普攻（50 额外伤害）
    commands.trigger(CommandAttackReset { entity });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, 1.0))
        .with_related::<BuffOf>(BuffOnHitBonusDamage {
            flat: 50.0,
            ratio: 0.0,
        });
}

fn on_jax_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jax: Query<(), With<Jax>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jax.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
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

fn on_jax_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jax: Query<(), With<Jax>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jax.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a self-cast that deals AoE damage and grants armor/mr
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
    // Armor/mr buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffResist::new(30.0, 30.0, 8.0));
}
