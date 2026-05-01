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

use crate::janna::buffs::BuffJannaPassive;

#[derive(Default)]
pub struct PluginJanna;

impl Plugin for PluginJanna {
    fn build(&self, app: &mut App) {
        app.add_observer(on_janna_skill_cast);
        app.add_observer(on_janna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Janna"))]
#[reflect(Component)]
pub struct Janna;

fn on_janna_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_janna: Query<(), With<Janna>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_janna.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_janna_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_janna_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_janna_e(&mut commands, entity),
        SkillSlot::R => cast_janna_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_janna_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Janna_Q_Cast"));

    // Q is a tornado
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1760.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Janna_Q_Hit")),
    );
}

fn cast_janna_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Janna_W_Cast"));

    // W is targeted damage and slow
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 550.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Janna_W_Hit")),
    );
}

fn cast_janna_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Janna_E_Cast"));
    // E is a shield
}

fn cast_janna_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Janna_R_Cast"));

    // R is AoE knockback
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Janna_R_Hit")),
    );
}

fn on_janna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_janna: Query<(), With<Janna>>,
) {
    let source = trigger.source;
    if q_janna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJannaPassive::new());
    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
