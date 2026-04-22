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

use crate::singed::buffs::BuffSingedE;

#[derive(Default)]
pub struct PluginSinged;

impl Plugin for PluginSinged {
    fn build(&self, app: &mut App) {
        app.add_observer(on_singed_skill_cast);
        app.add_observer(on_singed_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Singed"))]
#[reflect(Component)]
pub struct Singed;

fn on_singed_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_singed_q(&mut commands, entity),
        SkillSlot::W => cast_singed_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_singed_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_singed_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_singed_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_Q_Cast"));

    // Q is poison trail - damage over time
}

fn cast_singed_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_W_Cast"));

    // W is mega adhesive - slow
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
        Some(hash_bin("Singed_W_Hit")),
    );
}

fn cast_singed_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_E_Cast"));

    // E is fling - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 400.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Singed_E_Hit")),
    );
}

fn cast_singed_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_R_Cast"));

    // R is insanity - movespeed buff
}

fn on_singed_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
) {
    let source = trigger.source;
    if q_singed.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSingedE::new(0.6, 3.0));
}
