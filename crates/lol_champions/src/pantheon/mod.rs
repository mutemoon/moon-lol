pub mod buffs;

use bevy::prelude::{Handle, *};
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::pantheon::buffs::BuffPantheonE;

#[derive(Default)]
pub struct PluginPantheon;

impl Plugin for PluginPantheon {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pantheon_skill_cast);
        app.add_observer(on_pantheon_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Pantheon"))]
#[reflect(Component)]
pub struct Pantheon;

fn on_pantheon_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pantheon.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_pantheon_q(&mut commands, entity, trigger.point, skill_spell),
        SkillSlot::W => cast_pantheon_w(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::E => cast_pantheon_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_pantheon_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        _ => {}
    }
}

fn cast_pantheon_q(
    commands: &mut Commands,
    entity: Entity,
    _point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_Q_Cast"));
    // Q is a spear throw that can be held for more damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 400.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pantheon_Q_Hit")),
    );
}

fn cast_pantheon_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_W_Cast"));
    // W is a dash to target that stuns
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
            speed: 1000.0,
        },
    );
}

fn cast_pantheon_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_E_Cast"));
    // E is a shield block that deals damage in a cone when released
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 300.0,
            angle: 90.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pantheon_E_Hit")),
    );
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffPantheonE::new(Vec2::ZERO, 1.5));
}

fn cast_pantheon_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Pantheon_R_Cast"));
    // R is a long-range leap that damages enemies in area
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 2000.0 },
            damage: Some(DashDamage {
                radius_end: 200.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1500.0,
        },
    );
}

/// 监听 Pantheon 造成的伤害，W 命中时眩晕
fn on_pantheon_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
) {
    let source = trigger.source;
    if q_pantheon.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W 命中时眩晕
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}
