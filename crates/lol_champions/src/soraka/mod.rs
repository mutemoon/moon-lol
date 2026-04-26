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

use crate::soraka::buffs::BuffSorakaE;

#[derive(Default)]
pub struct PluginSoraka;

impl Plugin for PluginSoraka {
    fn build(&self, app: &mut App) {
        app.add_observer(on_soraka_skill_cast);
        app.add_observer(on_soraka_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Soraka"))]
#[reflect(Component)]
pub struct Soraka;

fn on_soraka_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_soraka.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_soraka_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_soraka_w(&mut commands, entity),
        SkillSlot::E => cast_soraka_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_soraka_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_soraka_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_Q_Cast"));

    // Q is starlon fallback - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 575.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Soraka_Q_Hit")),
    );
}

fn cast_soraka_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_W_Cast"));

    // W is infuse magic - heal
}

fn cast_soraka_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_E_Cast"));

    // E is barrier of mind - silence
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
        Some(hash_bin("Soraka_E_Hit")),
    );
}

fn cast_soraka_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_R_Cast"));

    // R is wishes - global heal
}

fn on_soraka_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
) {
    let source = trigger.source;
    if q_soraka.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E silences
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSorakaE::new(0.5, 1.0));
}
