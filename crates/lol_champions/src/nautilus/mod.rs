pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::nautilus::buffs::{BuffNautilusE, BuffNautilusW};

#[derive(Default)]
pub struct PluginNautilus;

impl Plugin for PluginNautilus {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nautilus_q);
        app.add_observer(on_nautilus_w);
        app.add_observer(on_nautilus_e);
        app.add_observer(on_nautilus_r);
        app.add_observer(on_nautilus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nautilus"))]
#[reflect(Component)]
pub struct Nautilus;

fn on_nautilus_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nautilus.get(entity).is_err() {
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
    // Q is a hook that drags
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1122.0,
                angle: 10.0,
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

fn on_nautilus_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nautilus.get(entity).is_err() {
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
    // W is a shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNautilusW::new(100.0, 6.0));
}

fn on_nautilus_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nautilus.get(entity).is_err() {
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
    // E is a three-hit wave that slows
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

fn on_nautilus_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nautilus.get(entity).is_err() {
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
    // R is a targeted knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 825.0,
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

fn on_nautilus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
) {
    let source = trigger.source;
    if q_nautilus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNautilusE::new(0.4, 1.5));
}
