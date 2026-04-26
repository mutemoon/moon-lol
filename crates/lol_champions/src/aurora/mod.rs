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

use crate::aurora::buffs::{BuffAuroraPassive, BuffAuroraR};

#[derive(Default)]
pub struct PluginAurora;

impl Plugin for PluginAurora {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aurora_skill_cast);
        app.add_observer(on_aurora_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aurora"))]
#[reflect(Component)]
pub struct Aurora;

fn on_aurora_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_aurora.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_aurora_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_aurora_w(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::E => cast_aurora_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_aurora_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_aurora_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_Q_Cast"));

    // Q is a projectile
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 850.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Aurora_Q_Hit")),
    );

    // Apply slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}

fn cast_aurora_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_W_Cast"));

    // W creates a portal - dash to it
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 600.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        },
    );
}

fn cast_aurora_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_E_Cast"));

    // E creates a path
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 700.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Aurora_E_Hit")),
    );
}

fn cast_aurora_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Aurora_R_Cast"));

    // R is area damage and freeze
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
        Some(hash_bin("Aurora_R_Hit")),
    );

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAuroraR::new());
}

fn on_aurora_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
) {
    let source = trigger.source;
    if q_aurora.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Passive slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAuroraPassive::new());
}
