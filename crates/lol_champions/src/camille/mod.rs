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
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

const CAMILLE_Q_RECAST_WINDOW: f32 = 3.0;
const CAMILLE_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginCamille;

impl Plugin for PluginCamille {
    fn build(&self, app: &mut App) {
        app.add_observer(on_camille_q);
        app.add_observer(on_camille_w);
        app.add_observer(on_camille_e);
        app.add_observer(on_camille_r);
        app.add_observer(on_camille_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Camille"))]
#[reflect(Component)]
pub struct Camille;

fn on_camille_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    cast_camille_q(
        &mut commands,
        entity,
        trigger.skill_entity,
        cooldown,
        recast,
        skill.spell.clone(),
    );
}

fn on_camille_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_camille_w(&mut commands, entity, skill.spell.clone());
}

fn on_camille_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    cast_camille_e(
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

fn on_camille_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    cast_camille_r(
        &mut commands,
        &q_transform,
        entity,
        trigger.point,
        skill.spell.clone(),
    );
}

fn cast_camille_q(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Prepares the hookshot
        // Q1 doesn't deal damage, just marks for second cast
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_Q_RECAST_WINDOW));
    } else {
        // Second cast: Deals bonus damage and resets attack
        commands.trigger(CommandAttackReset { entity });
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Nearest {
                    max_distance: 150.0,
                },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                }],
            }],
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

fn cast_camille_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a swept cone that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 300.0,
                angle: 90.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn cast_camille_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Hookshot - launches toward terrain
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash toward hooked terrain
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
            speed: 900.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

fn cast_camille_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a hookshot-like leap that marks and traps target champion
    commands.entity(entity).insert(DashDamageIntent {
        damage: DashDamage {
            radius_end: 150.0,
            damage: TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            },
        },
        skill: skill_spell,
    });
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer { max: 350.0 },
        speed: 800.0,
    });
}

/// 监听 Camille 造成的伤害，给目标施加减速
fn on_camille_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
) {
    let source = trigger.source;
    if q_camille.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 2.0));
}
