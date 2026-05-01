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

use crate::syndra::buffs::BuffSyndraE;

#[derive(Default)]
pub struct PluginSyndra;

impl Plugin for PluginSyndra {
    fn build(&self, app: &mut App) {
        app.add_observer(on_syndra_skill_cast);
        app.add_observer(on_syndra_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Syndra"))]
#[reflect(Component)]
pub struct Syndra;

fn on_syndra_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_syndra: Query<(), With<Syndra>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_syndra.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_syndra_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_syndra_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_syndra_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_syndra_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_syndra_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Syndra_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 800.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_Q_Hit")),
    );
}

fn cast_syndra_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Syndra_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 950.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_W_Hit")),
    );
}

fn cast_syndra_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Syndra_E_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 800.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_E_Hit")),
    );
}

fn cast_syndra_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Syndra_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 675.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Syndra_R_Hit")),
    );
}

fn on_syndra_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_syndra: Query<(), With<Syndra>>,
) {
    let source = trigger.source;
    if q_syndra.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSyndraE::new(0.75, 1.0));
}
