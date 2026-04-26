pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::jayce::buffs::BuffJaycePassive;

#[derive(Default)]
pub struct PluginJayce;

impl Plugin for PluginJayce {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jayce_skill_cast);
        app.add_observer(on_jayce_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jayce"))]
#[reflect(Component)]
pub struct Jayce;

fn on_jayce_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_jayce_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_jayce_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_jayce_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_jayce_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_jayce_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_Q_Cast"));

    // Q is a skillshot
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1050.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jayce_Q_Hit")),
    );
}

fn cast_jayce_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_W_Cast"));

    // W is an area slow
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Jayce_W_Hit")),
    );
}

fn cast_jayce_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_E_Cast"));

    // E is a knockback
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 500.0 },
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

fn cast_jayce_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_R_Cast"));
    // R transforms between hammer and cannon forms
}

fn on_jayce_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
) {
    let source = trigger.source;
    if q_jayce.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJaycePassive::new());
}
