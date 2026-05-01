pub mod buffs;

use bevy::asset::Handle;
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

use crate::morgana::buffs::{BuffMorganaE, BuffMorganaQ};

#[derive(Default)]
pub struct PluginMorgana;

impl Plugin for PluginMorgana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_morgana_skill_cast);
        app.add_observer(on_morgana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Morgana"))]
#[reflect(Component)]
pub struct Morgana;

fn on_morgana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morgana: Query<(), With<Morgana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_morgana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_morgana_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_morgana_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_morgana_e(&mut commands, entity),
        SkillSlot::R => cast_morgana_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_morgana_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Morgana_Q_Cast"));

    // Q binds enemy
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1300.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Morgana_Q_Hit")),
    );
}

fn cast_morgana_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Morgana_W_Cast"));

    // W is a DoT zone
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
        Some(hash_bin("Morgana_W_Hit")),
    );
}

fn cast_morgana_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Morgana_E_Cast"));

    // E is a shield that blocks CC
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMorganaE::new(150.0, true, 5.0));
}

fn cast_morgana_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Morgana_R_Cast"));

    // R chains nearby enemies
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 625.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Morgana_R_Hit")),
    );
}

fn on_morgana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morgana: Query<(), With<Morgana>>,
) {
    let source = trigger.source;
    if q_morgana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMorganaQ::new(2.0, 2.0));
}
