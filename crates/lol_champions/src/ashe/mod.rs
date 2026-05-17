pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::ashe::buffs::BuffAsheQ;

#[derive(Default)]
pub struct PluginAshe;

impl Plugin for PluginAshe {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ashe_skill_cast);
        app.add_observer(on_ashe_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ashe"))]
#[reflect(Component)]
pub struct Ashe;

fn on_ashe_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_ashe_q(&mut commands, entity),
        SkillSlot::W => cast_ashe_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_ashe_e(&mut commands, entity),
        SkillSlot::R => cast_ashe_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_ashe_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q grants attack speed buff and fires multiple arrows
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAsheQ::new());
}

fn cast_ashe_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a cone volley
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 40.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn cast_ashe_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is global vision - no damage
}

fn cast_ashe_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a global arrow that stuns - use large sector to simulate global range
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 20000.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_ashe_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
) {
    let source = trigger.source;
    if q_ashe.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply frost slow on all damage
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
