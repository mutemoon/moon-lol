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

use crate::samira::buffs::BuffSamiraE;

#[derive(Default)]
pub struct PluginSamira;

impl Plugin for PluginSamira {
    fn build(&self, app: &mut App) {
        app.add_observer(on_samira_skill_cast);
        app.add_observer(on_samira_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Samira"))]
#[reflect(Component)]
pub struct Samira;

fn on_samira_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_samira.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_samira_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_samira_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_samira_e(&mut commands, entity),
        SkillSlot::R => cast_samira_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_samira_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Samira_Q_Cast"));

    // Q is flonen - damage
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
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Samira_Q_Hit")),
    );
}

fn cast_samira_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Samira_W_Cast"));

    // W is blade storm - AoE damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Samira_W_Hit")),
    );
}

fn cast_samira_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Samira_E_Cast"));

    // E is blade rush - dash
}

fn cast_samira_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Samira_R_Cast"));

    // R is infernum - large AoE damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 900.0,
            angle: 50.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Samira_R_Hit")),
    );
}

fn on_samira_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
) {
    let source = trigger.source;
    if q_samira.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSamiraE::new(0.75, 1.0));
}
