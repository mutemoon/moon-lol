pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffSelfHeal;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, play_skill_animation,
    reset_skill_attack, skill_damage, skill_dash, spawn_skill_particle,
};

use crate::renekton::buffs::BuffRenektonR;

const RENECKTON_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginRenekton;

impl Plugin for PluginRenekton {
    fn build(&self, app: &mut App) {
        app.add_observer(on_renekton_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Renekton"))]
#[reflect(Component)]
pub struct Renekton;

fn on_renekton_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    mut q_ability_resource: Query<&mut AbilityResource>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_renekton_q(
            &mut commands,
            entity,
            &mut q_ability_resource,
            skill.key_spell_object.clone(),
        ),
        SkillSlot::W => cast_renekton_w(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::E => cast_renekton_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
            skill.key_spell_object.clone(),
        ),
        SkillSlot::R => cast_renekton_r(&mut commands, entity, skill.key_spell_object.clone()),
        _ => {}
    }
}

fn cast_renekton_q(
    commands: &mut Commands,
    entity: Entity,
    q_ability_resource: &mut Query<&mut AbilityResource>,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Renekton_Q_Cast"));
    // Q is a cleave that deals damage in a circle
    let rage = q_ability_resource
        .get(entity)
        .map(|r| r.value)
        .unwrap_or(0.0);
    if rage >= 50.0 {
        // 消耗 50 怒气，强化版伤害和治疗
        if let Ok(mut resource) = q_ability_resource.get_mut(entity) {
            resource.value -= 50.0;
        }
        skill_damage(
            commands,
            entity,
            skill_spell,
            DamageShape::Circle { radius: 300.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("Renekton_Q_Hit")),
        );
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffSelfHeal::new(80.0)); // 翻倍治疗
    } else {
        skill_damage(
            commands,
            entity,
            skill_spell,
            DamageShape::Circle { radius: 250.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("Renekton_Q_Hit")),
        );
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffSelfHeal::new(40.0));
    }
}

fn cast_renekton_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Renekton_W_Cast"));
    // W is an empowered auto attack that stuns
    reset_skill_attack(commands, entity);
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 150.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Renekton_W_Hit")),
    );
}

fn cast_renekton_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: Dash forward
        spawn_skill_particle(commands, entity, hash_bin("Renekton_E_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &ActionDash {
                skill: skill_spell,
                move_type: DashMoveType::Pointer { max: 200.0 },
                damage: Some(DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 700.0,
            },
        );
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            RENECKTON_E_RECAST_WINDOW,
        ));
    } else {
        // Second cast: Dash again
        spawn_skill_particle(commands, entity, hash_bin("Renekton_E2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &ActionDash {
                skill: skill_spell,
                move_type: DashMoveType::Pointer { max: 200.0 },
                damage: Some(DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 700.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
    }
}

fn cast_renekton_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Renekton_R_Cast"));
    // R is a transformation that deals damage around and generates rage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Renekton_R_Hit")),
    );
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRenektonR::new(0.0, 5.0, 15.0));
}
