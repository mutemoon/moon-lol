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

use crate::aphelios::buffs::{BuffApheliosCalibrum, BuffApheliosGravitum};

#[derive(Default)]
pub struct PluginAphelios;

impl Plugin for PluginAphelios {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aphelios_skill_cast);
        app.add_observer(on_aphelios_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aphelios"))]
#[reflect(Component)]
pub struct Aphelios;

fn on_aphelios_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_aphelios.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_aphelios_q(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_aphelios_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_aphelios_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Aphelios_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1450.0,
            angle: 25.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Aphelios_Q_Hit")),
    );
}

fn cast_aphelios_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Aphelios_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 1300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Aphelios_R_Hit")),
    );
}

fn on_aphelios_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
) {
    let source = trigger.source;
    if q_aphelios.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffApheliosCalibrum::new(70.0, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffApheliosGravitum::new(0.5, 2.0));
}
