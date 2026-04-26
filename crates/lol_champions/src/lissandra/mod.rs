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

use crate::lissandra::buffs::{BuffLissandraQ, BuffLissandraR, BuffLissandraW};

#[derive(Default)]
pub struct PluginLissandra;

impl Plugin for PluginLissandra {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lissandra_skill_cast);
        app.add_observer(on_lissandra_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lissandra"))]
#[reflect(Component)]
pub struct Lissandra;

fn on_lissandra_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lissandra.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_lissandra_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_lissandra_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_lissandra_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_lissandra_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_lissandra_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_Q_Cast"));

    // Q is a piercing ice shard that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 825.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_Q_Hit")),
    );
}

fn cast_lissandra_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_W_Cast"));

    // W is a circle root
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 275.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_W_Hit")),
    );
}

fn cast_lissandra_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_E_Cast"));

    // E is a dash-like skill
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1025.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_E_Hit")),
    );
}

fn cast_lissandra_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_R_Cast"));

    // R can self-cast for shield or enemy cast for damage+root
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLissandraR::new(true, 100.0, 2.5));

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
        Some(hash_bin("Lissandra_R_Hit")),
    );
}

fn on_lissandra_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
) {
    let source = trigger.source;
    if q_lissandra.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLissandraQ::new(0.3, 3.0));

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLissandraW::new(1.5, 3.0));
}
