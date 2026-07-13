use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashDamageIntent, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::{BuffMoveSpeed, BuffSelfHeal};
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

const AATROX_Q_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginAatrox;

impl Plugin for PluginAatrox {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aatrox_q);
        app.add_observer(on_aatrox_w);
        app.add_observer(on_aatrox_e);
        app.add_observer(on_aatrox_r);
        app.add_observer(on_aatrox_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aatrox"))]
#[reflect(Component)]
pub struct Aatrox;

fn on_aatrox_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    cast_aatrox_q(
        &mut commands,
        &q_transform,
        entity,
        trigger.skill_entity,
        trigger.point,
        cooldown,
        recast,
        skill.spell.clone(),
    );
}

fn on_aatrox_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_aatrox_w(&mut commands, entity, skill.spell.clone());
}

fn on_aatrox_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    cast_aatrox_e(
        &mut commands,
        &q_transform,
        entity,
        trigger.point,
        skill.spell.clone(),
    );
}

fn on_aatrox_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    cast_aatrox_r(&mut commands, entity, skill.spell.clone());
}

fn cast_aatrox_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    // Q has 3 stages with different animations
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    let animation_hash = match stage {
        1 => "spell1a".to_string(),
        2 => "spell1b".to_string(),
        _ => "spell1c".to_string(),
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: animation_hash,
        repeat: false,
        duration: None,
    });
    // Q is a 3-hit combo, each hit has different damage shape
    // Using circular damage similar to Riven
    commands.entity(entity).insert(DashDamageIntent {
        damage: DashDamage {
            radius_end: 200.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            },
        },
        skill: skill_spell,
    });
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Fixed(200.0),
        speed: 800.0,
    });

    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        // After 3rd Q, start cooldown
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Aatrox Q", stage
        );
    } else {
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            stage + 1,
            3,
            AATROX_Q_RECAST_WINDOW,
        ));
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}",
            entity, "Aatrox Q", stage
        );
    }
}

fn cast_aatrox_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a chain that pulls enemies back after delay
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn cast_aatrox_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    _skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a dash that heals based on damage dealt
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer { max: 250.0 },
        speed: 900.0,
    });
    // Self-heal based on damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSelfHeal::new(30.0));
}

fn cast_aatrox_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a self-cast that makes Aatrox unstoppable and deals damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
    // Movement speed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.5, 8.0));
}

/// 监听 Aatrox 造成的伤害，W 命中施加减速
fn on_aatrox_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
) {
    if q_aatrox.get(trigger.source).is_err() {
        return;
    }
    let target = trigger.event_target();
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.25, 1.5));
}
