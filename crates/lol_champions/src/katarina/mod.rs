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

use crate::katarina::buffs::{BuffKatarinaVoracity, BuffKatarinaW};

#[derive(Default)]
pub struct PluginKatarina;

impl Plugin for PluginKatarina {
    fn build(&self, app: &mut App) {
        app.add_observer(on_katarina_skill_cast);
        app.add_observer(on_katarina_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Katarina"))]
#[reflect(Component)]
pub struct Katarina;

fn on_katarina_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_katarina.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_katarina_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_katarina_w(&mut commands, entity),
        SkillSlot::E => cast_katarina_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_katarina_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_katarina_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_Q_Cast"));

    // Q bounces between enemies
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
        Some(hash_bin("Katarina_Q_Hit")),
    );
}

fn cast_katarina_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_W_Cast"));

    // W throws dagger up and grants movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKatarinaW::new(0.8, 2.0));
}

fn cast_katarina_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_E_Cast"));

    // E is a dash to target
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 100.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Katarina_E_Hit")),
    );
}

fn cast_katarina_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_R_Cast"));

    // R throws daggers in area
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 550.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Katarina_R_Hit")),
    );
}

fn on_katarina_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
) {
    let source = trigger.source;
    if q_katarina.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive: kill reduces cooldowns
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKatarinaVoracity::new(15.0, 3.0));
}
