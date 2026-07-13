pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashDamageIntent, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::{BuffMoveSpeed, BuffSelfHeal};
use lol_core::buffs::on_hit::{BuffOnHitCounter, BuffOnHitStun};
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::volibear::buffs::DebuffVolibearWMark;

const VOLIBEAR_W_RECAST_WINDOW: f32 = 2.0;

#[derive(Default)]
pub struct PluginVolibear;

impl Plugin for PluginVolibear {
    fn build(&self, app: &mut App) {
        app.add_observer(on_volibear_q);
        app.add_observer(on_volibear_w);
        app.add_observer(on_volibear_e);
        app.add_observer(on_volibear_r);
        app.add_observer(on_volibear_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Volibear"))]
#[reflect(Component)]
pub struct Volibear;

fn on_volibear_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    cast_volibear_q(&mut commands, entity);
}

fn on_volibear_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_volibear_w(
        &mut commands,
        entity,
        skill.spell.clone(),
        trigger.skill_entity,
        trigger.point,
        cooldown,
        recast,
    );
}

fn on_volibear_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    cast_volibear_e(&mut commands, entity, skill.spell.clone());
}

fn on_volibear_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    cast_volibear_r(
        &mut commands,
        &q_transform,
        entity,
        skill.spell.clone(),
        trigger.point,
    );
}

fn cast_volibear_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q 组合：加速 + 下次普攻眩晕
    commands.trigger(CommandAttackReset { entity });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, 4.0))
        .with_related::<BuffOf>(BuffOnHitStun { duration: 1.0 })
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.3, 4.0));
}

fn cast_volibear_w(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_entity: Entity,
    _point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: W marks target
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            VOLIBEAR_W_RECAST_WINDOW,
        ));
    } else {
        // Second cast: W detonates mark for bonus damage + heal
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Nearest {
                    max_distance: 200.0,
                },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                }],
            }],
        });
        // W2 命中已标记目标时自我治疗
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffSelfHeal::new(50.0));
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

fn cast_volibear_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is AoE damage + slow + shield
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}

fn cast_volibear_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a leap that deals damage and marks towers as vulnerable
    commands.entity(entity).insert(DashDamageIntent {
        damage: DashDamage {
            radius_end: 150.0,
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
        move_type: DashMoveType::Pointer { max: 400.0 },
        speed: 800.0,
    });
}

/// 监听 Volibear 造成的伤害，W1 标记目标，E 命中减速
fn on_volibear_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
) {
    let source = trigger.source;
    if q_volibear.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W1 标记目标
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffVolibearWMark::new(source, 4.0));
    // E 命中减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
