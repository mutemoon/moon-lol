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

use crate::belveth::buffs::{BuffBelvethPassive, BuffBelvethW};

#[derive(Default)]
pub struct PluginBelveth;

impl Plugin for PluginBelveth {
    fn build(&self, app: &mut App) {
        app.add_observer(on_belveth_skill_cast);
        app.add_observer(on_belveth_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Belveth"))]
#[reflect(Component)]
pub struct Belveth;

fn on_belveth_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_belveth.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_belveth_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_belveth_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_belveth_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_belveth_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_belveth_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Belveth_Q_Hit")),
    );
}

fn cast_belveth_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 660.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Belveth_W_Hit")),
    );
}

fn cast_belveth_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_E_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Belveth_E_Hit")),
    );
}

fn cast_belveth_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_R_Cast"));
}

fn on_belveth_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
) {
    let source = trigger.source;
    if q_belveth.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBelvethPassive::new(2, 0.1, 5.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBelvethW::new(0.5, 2.0));
}
