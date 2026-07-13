pub mod buffs;

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
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::anivia::buffs::BuffAniviaR;

const ANIVIA_Q_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginAnivia;

impl Plugin for PluginAnivia {
    fn build(&self, app: &mut App) {
        app.add_observer(on_anivia_q);
        app.add_observer(on_anivia_w);
        app.add_observer(on_anivia_e);
        app.add_observer(on_anivia_r);
        app.add_observer(on_anivia_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Anivia"))]
#[reflect(Component)]
pub struct Anivia;

fn on_anivia_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_anivia.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let skill_spell = skill.spell.clone();
    let skill_entity = trigger.skill_entity;
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: launch the crystal
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, ANIVIA_Q_RECAST_WINDOW));
    } else {
        // Second cast: detonate for extra damage and stun
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 150.0 },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Magic,
                }],
            }],
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        });
    };
}

fn on_anivia_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_anivia.get(entity).is_err() {
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
    // W creates a wall that blocks movement;
}

fn on_anivia_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_anivia.get(entity).is_err() {
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
    // E deals extra damage to frozen targets
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_anivia_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_anivia.get(entity).is_err() {
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
    // R is a continuous storm
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 750.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAniviaR::new());
}

fn on_anivia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
) {
    let source = trigger.source;
    if q_anivia.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q and R slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.2, 2.0));
}
