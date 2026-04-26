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

use crate::sivir::buffs::BuffSivirW;

#[derive(Default)]
pub struct PluginSivir;

impl Plugin for PluginSivir {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sivir_skill_cast);
        app.add_observer(on_sivir_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sivir"))]
#[reflect(Component)]
pub struct Sivir;

fn on_sivir_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_sivir.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_sivir_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_sivir_w(&mut commands, entity),
        SkillSlot::E => cast_sivir_e(&mut commands, entity),
        SkillSlot::R => cast_sivir_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_sivir_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_Q_Cast"));

    // Q is boomerang blade - damage
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
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Sivir_Q_Hit")),
    );
}

fn cast_sivir_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_W_Cast"));

    // W is ricochet - attackspeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSivirW::new(0.5, 5.0));
}

fn cast_sivir_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_E_Cast"));

    // E is spell shield - magic shield
}

fn cast_sivir_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_R_Cast"));

    // R is on the hunt - movespeed buff
}

fn on_sivir_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
) {
    let source = trigger.source;
    if q_sivir.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // W gives attackspeed to caster
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffSivirW::new(0.5, 5.0));
}
