pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::ryze::buffs::BuffRyzeW;

#[derive(Default)]
pub struct PluginRyze;

impl Plugin for PluginRyze {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ryze_skill_cast);
        app.add_observer(on_ryze_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ryze"))]
#[reflect(Component)]
pub struct Ryze;

fn on_ryze_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ryze: Query<(), With<Ryze>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ryze.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_ryze_q(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::W => cast_ryze_w(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::E => cast_ryze_e(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::R => cast_ryze_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_ryze_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ryze_Q_Cast"));

    // Q is overload - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 600.0,
            angle: 25.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ryze_Q_Hit")),
    );
}

fn cast_ryze_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ryze_W_Cast"));

    // W is rune prison - root
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ryze_W_Hit")),
    );
}

fn cast_ryze_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ryze_E_Cast"));

    // E is spell flux - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ryze_E_Hit")),
    );
}

fn cast_ryze_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ryze_R_Cast"));

    // R is realm warp - teleport
}

fn on_ryze_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ryze: Query<(), With<Ryze>>,
) {
    let source = trigger.source;
    if q_ryze.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRyzeW::new(0.7, 1.0));
}
