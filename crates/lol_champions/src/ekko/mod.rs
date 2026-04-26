pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::ekko::buffs::BuffEkkoPassive;

#[derive(Default)]
pub struct PluginEkko;

impl Plugin for PluginEkko {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ekko_skill_cast);
        app.add_observer(on_ekko_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ekko"))]
#[reflect(Component)]
pub struct Ekko;

fn on_ekko_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ekko.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_ekko_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_ekko_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_ekko_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_ekko_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_ekko_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_Q_Cast"));

    // Q is a projectile that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ekko_Q_Hit")),
    );
}

fn cast_ekko_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_W_Cast"));

    // W creates a zone that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ekko_W_Hit")),
    );
}

fn cast_ekko_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_E_Cast"));

    // E is a dash that enhances next attack
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 325.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 1000.0,
        },
    );
}

fn cast_ekko_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ekko_R_Cast"));

    // R is an AoE damage around previous position
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ekko_R_Hit")),
    );
}

fn on_ekko_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
) {
    let source = trigger.source;
    if q_ekko.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffEkkoPassive::new());
    // Q and W slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}
