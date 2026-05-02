pub mod buffs;

use bevy::prelude::{GlobalTransform, *};
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffFear;
use lol_core::buffs::common_buffs::{BuffMoveSpeed, BuffSelfHeal};
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::movement::{CommandMovement, EventMovementEnd, MovementAction, MovementWay};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};
use lol_core::team::Team;

use crate::hecarim::buffs::{BuffHecarimQ, BuffHecarimW};

// Hecarim Q parameters
const HECARIM_Q_MAX_STACKS: u8 = 4;
const HECARIM_Q_COOLDOWN_REDUCTION: f32 = 0.5; // 0.5s per stack
const HECARIM_Q_DAMAGE_BONUS: f32 = 0.1; // 10% bonus per stack

#[derive(Default)]
pub struct PluginHecarim;

impl Plugin for PluginHecarim {
    fn build(&self, app: &mut App) {
        app.add_observer(on_hecarim_skill_cast);
        app.add_observer(on_hecarim_damage_hit);
        app.add_observer(on_hecarim_e_dash_end);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Hecarim"))]
#[reflect(Component)]
pub struct Hecarim;

fn on_hecarim_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_hecarim: Query<(), With<Hecarim>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_hecarim.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_hecarim_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_hecarim_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_hecarim_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_hecarim_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        _ => {}
    }
}

fn cast_hecarim_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Hecarim_Q_Cast"),
    });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Hecarim_Q_Hit")),
        }],
    });
    // Q stacks - adds stacking buff for cooldown reduction and bonus damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffHecarimQ::new(
            HECARIM_Q_MAX_STACKS,
            HECARIM_Q_COOLDOWN_REDUCTION,
            HECARIM_Q_DAMAGE_BONUS,
        ));
    debug!("{:?} 释放了 {} 技能，获得层数", entity, "Hecarim Q");
}

fn cast_hecarim_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Hecarim_W_Cast"),
    });
    // W is AoE damage in area + healing based on damage dealt
    // Apply BuffHecarimW that will trigger heal on damage dealt
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffHecarimW::new(4.0));
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
            particle: Some(hash_bin("Hecarim_W_Hit")),
        }],
    });
}

fn cast_hecarim_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Hecarim_E_Cast"),
    });
    // E is movement speed boost + knockback on contact
    // Movement speed buff with knockback on collision
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.75, 4.0));
}

fn cast_hecarim_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Hecarim_R_Cast"),
    });
    // R is a long dash with fear
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell.clone(),
        move_type: DashMoveType::Pointer { max: 800.0 },
        damage: Some(DashDamage {
            radius_end: 200.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            },
        }),
        speed: 1500.0,
    });
}

/// Hecarim W 持续期间造成伤害时给自身治疗，R 命中施加恐惧
fn on_hecarim_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_hecarim: Query<(), With<Hecarim>>,
    q_has_w_buff: Query<(), With<BuffHecarimW>>,
) {
    let source = trigger.source;
    if q_hecarim.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W buff 期间造成伤害时自我治疗
    if q_has_w_buff.get(source).is_ok() {
        commands
            .entity(source)
            .with_related::<BuffOf>(BuffSelfHeal::new(30.0));
    }
    // R 命中施加恐惧
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffFear::new(1.5));
}

/// Hecarim E 冲刺结束时推开最近目标
fn on_hecarim_e_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_hecarim: Query<(), With<Hecarim>>,
    q_hecarim_transform: Query<&GlobalTransform>,
    q_target: Query<(Entity, &Team, &GlobalTransform)>,
) {
    let entity = trigger.event_target();
    // Only process if this is Hecarim's dash ending
    if q_hecarim.get(entity).is_err() {
        return;
    }
    if trigger.source != "HecarimE" {
        return;
    }

    let Ok(hecarim_transform) = q_hecarim_transform.get(entity) else {
        return;
    };

    // Find nearest enemy to knock back
    let mut nearest: Option<(Entity, f32)> = None;
    let targets: Vec<(Entity, &Team, &GlobalTransform)> = q_target.iter().collect();
    for (target, team, target_transform) in targets.iter() {
        if *team == &Team::Order {
            continue;
        }
        let dist = target_transform
            .translation()
            .distance(hecarim_transform.translation());
        if nearest.map_or(true, |(_, d)| dist < d) {
            nearest = Some((*target, dist));
        }
    }

    if let Some((target_entity, _)) = nearest {
        let knockback_dir = {
            let target_transform = q_target.get(target_entity).unwrap().2;
            target_transform.translation() - hecarim_transform.translation()
        };
        if knockback_dir.length() > 0.1 {
            let knockback_pos = {
                let target_transform = q_target.get(target_entity).unwrap().2;
                target_transform.translation() + knockback_dir.normalize() * 150.0
            };
            commands.trigger(CommandMovement {
                entity: target_entity,
                priority: 100,
                action: MovementAction::Start {
                    way: MovementWay::Pathfind(Vec3::new(
                        knockback_pos.x,
                        knockback_pos.y,
                        knockback_pos.z,
                    )),
                    speed: Some(800.0),
                    source: "HecarimE".to_string(),
                },
            });
        }
    }
}
