pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::kayn::buffs::BuffKaynRActive;

#[derive(Default)]
pub struct PluginKayn;

impl Plugin for PluginKayn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kayn_q);
        app.add_observer(on_kayn_w);
        app.add_observer(on_kayn_e);
        app.add_observer(on_kayn_r);
        app.add_observer(on_kayn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kayn"))]
#[reflect(Component)]
pub struct Kayn;

/// Kayn has two forms: Blue (assassin) and Red (bruiser)
/// The form is determined by which enemy champion is damaged first with R
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub enum KaynForm {
    #[default]
    None,
    Blue, // Assassin form
    Red,  // Bruiser form
}

fn on_kayn_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayn.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a dash that deals damage
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Fixed(250.0),
        speed: 700.0,
    });
}

fn on_kayn_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayn.get(entity).is_err() {
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
    // W is an upward slash that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 300.0,
                angle: 60.0,
            },
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

fn on_kayn_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayn.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let _point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a ghost-like dash that allows passing through terrain
    // Movement speed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.4, 1.5));
}

fn on_kayn_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayn.get(entity).is_err() {
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
    // R 寄生：给自身挂 BuffKaynRActive（不可选中状态）
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaynRActive::new(Entity::PLACEHOLDER, 2.5));
}

/// 监听 Kayn 造成的伤害，W 命中时减速
fn on_kayn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
) {
    let source = trigger.source;
    if q_kayn.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W 命中时减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 1.5));
}
