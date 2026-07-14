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
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::lulu::buffs::{BuffLuluE, BuffLuluR, BuffLuluW};

#[derive(Default)]
pub struct PluginLulu;

impl Plugin for PluginLulu {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lulu_q);
        app.add_observer(on_lulu_w);
        app.add_observer(on_lulu_e);
        app.add_observer(on_lulu_r);
        app.add_observer(on_lulu_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lulu"))]
#[reflect(Component)]
pub struct Lulu;

fn on_lulu_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lulu.get(entity).is_err() {
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

    // Q is a bolt that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 950.0,
                angle: 15.0,
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

fn on_lulu_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lulu.get(entity).is_err() {
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

    // W polymorphs enemy or buffs ally
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLuluW::new(false, 0.3, 0.25, 2.5));
}

fn on_lulu_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lulu.get(entity).is_err() {
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

    // E shields ally or damages enemy
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLuluE::new(80.0, 2.5));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 650.0 },
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

fn on_lulu_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lulu.get(entity).is_err() {
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

    // R knocks up nearby enemies and grants bonus health
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLuluR::new(400.0, true, 0.5, 4.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
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

fn on_lulu_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
) {
    let source = trigger.source;
    if q_lulu.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.8, 2.0));
}
