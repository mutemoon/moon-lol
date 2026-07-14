pub mod buffs;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

// TODO(armor_reduction): TryndamereW 的减甲效果待接入伤害系统（见 buffs::BuffTryndamereW）

#[derive(Default)]
pub struct PluginTryndamere;

impl Plugin for PluginTryndamere {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tryndamere_q);
        app.add_observer(on_tryndamere_w);
        app.add_observer(on_tryndamere_e);
        app.add_observer(on_tryndamere_r);
        app.add_observer(on_tryndamere_damage_hit);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Tryndamere"))]
#[reflect(Component)]
pub struct Tryndamere;

fn on_tryndamere_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tryndamere.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
}

fn on_tryndamere_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tryndamere.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 30.0,
            },
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

fn on_tryndamere_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tryndamere.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 660.0 },
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

fn on_tryndamere_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tryndamere.get(entity).is_err() {
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
}

fn on_tryndamere_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
) {
    let source = trigger.source;
    if q_tryndamere.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // 减速：用通用 DebuffSlow（独立 buff 实体，观察者桥接 MovementSlow 标记）。
    // TODO: 当前对所有 Tryndamere 伤害命中触发，应仅限 W 伤害（需 EventDamageCreate 携带技能来源）。
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 2.0));
}
