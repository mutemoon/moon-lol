pub mod buffs;

use bevy::asset::Handle;
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

use crate::maokai::buffs::BuffMaokaiW;

#[derive(Default)]
pub struct PluginMaokai;

impl Plugin for PluginMaokai {
    fn build(&self, app: &mut App) {
        app.add_observer(on_maokai_skill_cast);
        app.add_observer(on_maokai_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Maokai"))]
#[reflect(Component)]
pub struct Maokai;

fn on_maokai_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_maokai.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_maokai_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_maokai_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_maokai_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_maokai_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_maokai_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Maokai_Q_Cast"));

    // Q is a knockback
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 350.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Maokai_Q_Hit")),
    );
}

fn cast_maokai_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Maokai_W_Cast"));

    // W is a dash that roots
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 525.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Maokai_W_Hit")),
    );
}

fn cast_maokai_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Maokai_E_Cast"));

    // E throws sapling that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 1100.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Maokai_E_Hit")),
    );
}

fn cast_maokai_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Maokai_R_Cast"));

    // R is a global knockup
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 3000.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Maokai_R_Hit")),
    );
}

fn on_maokai_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
) {
    let source = trigger.source;
    if q_maokai.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMaokaiW::new(2.0, 2.0));

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 3.0));
}
