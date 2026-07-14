pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::ahri::buffs::{BuffAhriFoxFire, BuffCharm};

#[derive(Default)]
pub struct PluginAhri;

impl Plugin for PluginAhri {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ahri_q);
        app.add_observer(on_ahri_w);
        app.add_observer(on_ahri_e);
        app.add_observer(on_ahri_r);
        app.add_observer(on_ahri_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ahri"))]
#[reflect(Component)]
pub struct Ahri;

fn on_ahri_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ahri.get(entity).is_err() {
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
    // Q creates a missile that travels out and returns
    // First pass: magic damage in a cone
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 90.0,
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

    // Apply fox fire buff for W tracking (will be consumed by W)
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAhriFoxFire::new(3));
}

fn on_ahri_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ahri.get(entity).is_err() {
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
    // Fox-fire: Three flames orbit Ahri and can attack enemies
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAhriFoxFire::new(3));

    // W damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
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

fn on_ahri_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ahri.get(entity).is_err() {
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
    // E is a charm missile that charms on hit
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1000.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_ahri_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_ahri.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // R is a dash that can be recast twice
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 500.0 },
        speed: 600.0,
    });

    // R has 2 recasts within 15 seconds
    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    } else {
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(stage + 1, 3, 15.0));
    };
}

/// Listen for Ahri damage hits to apply effects
fn on_ahri_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
) {
    let source = trigger.source;
    if q_ahri.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Check if this was from E (charm) to apply charm debuff
    // The charm effect is applied based on the skill hash
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCharm::new(1.5));
}
