use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginGnar;

impl Plugin for PluginGnar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_gnar_q);
        app.add_observer(on_gnar_w);
        app.add_observer(on_gnar_e);
        app.add_observer(on_gnar_r);
        app.add_observer(on_gnar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Gnar"))]
#[reflect(Component)]
pub struct Gnar;

/// Gnar transforms into Mega Gnar at 100 rage
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub enum GnarForm {
    #[default]
    Mini,
    Mega,
}

fn on_gnar_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gnar.get(entity).is_err() {
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
    // Q 回旋镖：Sector 模拟直线飞行
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_gnar_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gnar.get(entity).is_err() {
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
    // Mega 形态 W：AOE 伤害 + 眩晕
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 250.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_gnar_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gnar.get(entity).is_err() {
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
    // E 是跳跃，可以二段跳
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 300.0 },
        speed: 600.0,
    });
}

fn on_gnar_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gnar.get(entity).is_err() {
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
    // R is only available in Mega form - throws enemies and stuns
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

/// 监听 Gnar 造成的伤害，所有伤害命中施加减速
fn on_gnar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
) {
    let source = trigger.source;
    if q_gnar.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // 所有伤害命中施加减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.5));
}
