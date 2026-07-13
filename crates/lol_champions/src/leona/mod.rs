pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::leona::buffs::{BuffLeonaSunlight, BuffLeonaW};

#[derive(Default)]
pub struct PluginLeona;

impl Plugin for PluginLeona {
    fn build(&self, app: &mut App) {
        app.add_observer(on_leona_q);
        app.add_observer(on_leona_w);
        app.add_observer(on_leona_e);
        app.add_observer(on_leona_r);
        app.add_observer(on_leona_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Leona"))]
#[reflect(Component)]
pub struct Leona;

fn on_leona_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leona: Query<(), With<Leona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_leona.get(entity).is_err() {
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

    // Q is an empowered auto attack that stuns
    commands.trigger(CommandAttackReset { entity });
}

fn on_leona_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leona: Query<(), With<Leona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_leona.get(entity).is_err() {
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

    // W grants damage reduction and armor/mr
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLeonaW::new(0.5, 40.0, 40.0, 3.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 450.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_leona_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leona: Query<(), With<Leona>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_leona.get(entity).is_err() {
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

    // E is a dash that roots on hit
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
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

fn on_leona_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leona: Query<(), With<Leona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_leona.get(entity).is_err() {
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

    // R is a global nuke that slows and stuns center
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_leona_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_leona: Query<(), With<Leona>>,
) {
    let source = trigger.source;
    if q_leona.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Passive applies sunlight
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLeonaSunlight::new(50.0, 2.5));

    // R applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.8, 1.75));
}
