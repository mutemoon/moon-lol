pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::cassiopeia::buffs::BuffCassioPoison;

#[derive(Default)]
pub struct PluginCassiopeia;

impl Plugin for PluginCassiopeia {
    fn build(&self, app: &mut App) {
        app.add_observer(on_cassiopeia_q);
        app.add_observer(on_cassiopeia_w);
        app.add_observer(on_cassiopeia_e);
        app.add_observer(on_cassiopeia_r);
        app.add_observer(on_cassiopeia_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Cassiopeia"))]
#[reflect(Component)]
pub struct Cassiopeia;

fn on_cassiopeia_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_cassiopeia: Query<(), With<Cassiopeia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_cassiopeia.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    cast_cassio_q(&mut commands, entity, skill.spell.clone());
}

fn on_cassiopeia_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_cassiopeia: Query<(), With<Cassiopeia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_cassiopeia.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_cassio_w(&mut commands, entity, skill.spell.clone());
}

fn on_cassiopeia_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_cassiopeia: Query<(), With<Cassiopeia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_cassiopeia.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    cast_cassio_e(&mut commands, entity, skill.spell.clone());
}

fn on_cassiopeia_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_cassiopeia: Query<(), With<Cassiopeia>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_cassiopeia.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    cast_cassio_r(&mut commands, entity, skill.spell.clone());
}

fn cast_cassio_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is ground targeted area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
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

fn cast_cassio_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a poison cloud
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
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

fn cast_cassio_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is targeted damage to poisoned enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
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

fn cast_cassio_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a cone that stuns facing enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 80.0,
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

fn on_cassiopeia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_cassio: Query<(), With<Cassiopeia>>,
) {
    let source = trigger.source;
    if q_cassio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply poison
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCassioPoison::new());
}
