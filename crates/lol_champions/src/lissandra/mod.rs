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

use crate::lissandra::buffs::{BuffLissandraQ, BuffLissandraR, BuffLissandraW};

#[derive(Default)]
pub struct PluginLissandra;

impl Plugin for PluginLissandra {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lissandra_q);
        app.add_observer(on_lissandra_w);
        app.add_observer(on_lissandra_e);
        app.add_observer(on_lissandra_r);
        app.add_observer(on_lissandra_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lissandra"))]
#[reflect(Component)]
pub struct Lissandra;

fn on_lissandra_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lissandra.get(entity).is_err() {
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

    // Q is a piercing ice shard that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 825.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_lissandra_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lissandra.get(entity).is_err() {
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

    // W is a circle root
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 275.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_lissandra_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lissandra.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let _point = trigger.point;
    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    // E is a dash-like skill
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1025.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_lissandra_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lissandra.get(entity).is_err() {
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

    // R can self-cast for shield or enemy cast for damage+root
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLissandraR::new(true, 100.0, 2.5));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_lissandra_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
) {
    let source = trigger.source;
    if q_lissandra.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLissandraQ::new(0.3, 3.0));

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLissandraW::new(1.5, 3.0));
}
