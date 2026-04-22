pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::draven::buffs::BuffDravenPassive;

#[derive(Default)]
pub struct PluginDraven;

impl Plugin for PluginDraven {
    fn build(&self, app: &mut App) {
        app.add_observer(on_draven_skill_cast);
        app.add_observer(on_draven_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Draven"))]
#[reflect(Component)]
pub struct Draven;

fn on_draven_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_draven_q(&mut commands, entity),
        SkillSlot::W => cast_draven_w(&mut commands, entity),
        SkillSlot::E => cast_draven_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_draven_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_draven_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_Q_Cast"));

    // Q enhances next attack - handled by buff system
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDravenPassive::new());
}

fn cast_draven_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_W_Cast"));
    // W is movement speed buff - handled by buff system
}

fn cast_draven_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_E_Cast"));

    // E is a knockback skillshot
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Draven_E_Hit")),
    );
}

fn cast_draven_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_R_Cast"));

    // R is global damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 20000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Draven_R_Hit")),
    );
}

fn on_draven_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
) {
    let source = trigger.source;
    if q_draven.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 2.0));
}
