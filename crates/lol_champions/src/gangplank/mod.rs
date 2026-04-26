pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, spawn_skill_particle,
};

use crate::gangplank::buffs::BuffGangplankPassive;

#[derive(Default)]
pub struct PluginGangplank;

impl Plugin for PluginGangplank {
    fn build(&self, app: &mut App) {
        app.add_observer(on_gangplank_skill_cast);
        app.add_observer(on_gangplank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Gangplank"))]
#[reflect(Component)]
pub struct Gangplank;

fn on_gangplank_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_gangplank_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_gangplank_w(&mut commands, entity),
        SkillSlot::E => cast_gangplank_e(&mut commands, entity),
        SkillSlot::R => cast_gangplank_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_gangplank_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_Q_Cast"));

    // Q is targeted damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 625.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Gangplank_Q_Hit")),
    );
}

fn cast_gangplank_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_W_Cast"));
    // W removes CC and heals
}

fn cast_gangplank_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_E_Cast"));
    // E places barrel
}

fn cast_gangplank_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_R_Cast"));

    // R is global AoE
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 20000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Gangplank_R_Hit")),
    );
}

fn on_gangplank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
) {
    let source = trigger.source;
    if q_gangplank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGangplankPassive::new());
}
