pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::jayce::buffs::BuffJaycePassive;

#[derive(Default)]
pub struct PluginJayce;

impl Plugin for PluginJayce {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jayce_q);
        app.add_observer(on_jayce_w);
        app.add_observer(on_jayce_e);
        app.add_observer(on_jayce_r);
        app.add_observer(on_jayce_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jayce"))]
#[reflect(Component)]
pub struct Jayce;

fn on_jayce_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // 电能震荡：电球 + 弹道光效
    for key in ["Jayce_OrbLightning", "Jayce_Q_range_xp"] {
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: key.to_string(),
            rotation: None,
            resolver_entity: None,
        });
    }
    // Q is a skillshot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_jayce_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // 闪电领域：电击冲击 + 持续静电场光效
    for key in ["Jayce_StaticStormShock", "Jayce_StaticStorm_aura"] {
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: key.to_string(),
            rotation: None,
            resolver_entity: None,
        });
    }
    // W is an area slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_jayce_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // 雷霆一击：击退命中光效
    for key in ["Jayce_ThunderingBlow_tar", "Jayce_ThunderingBlow_Hit"] {
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: key.to_string(),
            rotation: None,
            resolver_entity: None,
        });
    }
    // E is a knockback
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 500.0 },
        speed: 1000.0,
    });
}

fn on_jayce_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // 形态切换：锤↔炮变形光效
    for key in ["Jayce_Model_Swap", "Jayce_Model_Swap2"] {
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: key.to_string(),
            rotation: None,
            resolver_entity: None,
        });
    }
    // R transforms between hammer and cannon forms;
}

fn on_jayce_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
) {
    let source = trigger.source;
    if q_jayce.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // 被动破甲减抗 debuff 光效（挂在目标上）
    commands.trigger(CommandSkinParticleSpawn {
        entity: target,
        hash: "Jayce_P_Debuff_Armor_MR_Shred".to_string(),
        rotation: None,
        resolver_entity: Some(source),
    });

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJaycePassive::new());
}
