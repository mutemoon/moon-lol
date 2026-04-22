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

use crate::renata::buffs::{BuffRenataQ, BuffRenataR, BuffRenataW};

#[derive(Default)]
pub struct PluginRenata;

impl Plugin for PluginRenata {
    fn build(&self, app: &mut App) {
        app.add_observer(on_renata_skill_cast);
        app.add_observer(on_renata_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Renata"))]
#[reflect(Component)]
pub struct Renata;

fn on_renata_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_renata.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_renata_q(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::W => cast_renata_w(&mut commands, entity),
        SkillSlot::E => cast_renata_e(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::R => cast_renata_r(&mut commands, entity, skill.key_spell_object.clone()),
        _ => {}
    }
}

fn cast_renata_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Renata_Q_Cast"));

    // Q is header lash - damage and slow
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 900.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Renata_Q_Hit")),
    );
}

fn cast_renata_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Renata_W_Cast"));

    // W is loyalty program - attackspeed buff to ally
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRenataW::new(0.5, 5.0));
}

fn cast_renata_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Renata_E_Cast"));

    // E is trusim - damage and shield
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 800.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Renata_E_Hit")),
    );
}

fn cast_renata_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Renata_R_Cast"));

    // R is hostile takeovers - AoE stun
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1500.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Renata_R_Hit")),
    );
}

fn on_renata_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
) {
    let source = trigger.source;
    if q_renata.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRenataQ::new(0.5, 1.5));
    // R stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRenataR::new(0.75, 1.0));
}
