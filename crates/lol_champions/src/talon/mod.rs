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

use crate::talon::buffs::BuffTalonW;

#[derive(Default)]
pub struct PluginTalon;

impl Plugin for PluginTalon {
    fn build(&self, app: &mut App) {
        app.add_observer(on_talon_skill_cast);
        app.add_observer(on_talon_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Talon"))]
#[reflect(Component)]
pub struct Talon;

fn on_talon_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_talon.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_talon_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_talon_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_talon_e(&mut commands, entity),
        SkillSlot::R => cast_talon_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_talon_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 600.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Talon_Q_Hit")),
    );
}

fn cast_talon_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 600.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Talon_W_Hit")),
    );
}

fn cast_talon_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_E_Cast"));
}

fn cast_talon_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Talon_R_Hit")),
    );
}

fn on_talon_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
) {
    let source = trigger.source;
    if q_talon.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTalonW::new(0.4, 2.0));
}
