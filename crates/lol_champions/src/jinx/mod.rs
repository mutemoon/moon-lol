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
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, reset_skill_attack,
    skill_damage, spawn_skill_particle,
};

#[derive(Default)]
pub struct PluginJinx;

impl Plugin for PluginJinx {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jinx_skill_cast);
        app.add_observer(on_jinx_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jinx"))]
#[reflect(Component)]
pub struct Jinx;

fn on_jinx_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jinx.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_jinx_q(&mut commands, entity),
        SkillSlot::W => cast_jinx_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_jinx_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_jinx_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_jinx_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_Q_Cast"));
    // Q switches between minigun and rocket launcher
    // Minigun gives attackspeed stacks, rocket deals AoE
    reset_skill_attack(commands, entity);
}

fn cast_jinx_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_W_Cast"));

    // W is a skillshot that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1500.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jinx_W_Hit")),
    );
}

fn cast_jinx_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_E_Cast"));

    // E places traps that explode and knock up
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Jinx_E_Hit")),
    );
}

fn cast_jinx_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_R_Cast"));

    // R is a global rocket with damage based on distance
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 25000.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jinx_R_Hit")),
    );
}

fn on_jinx_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
) {
    let source = trigger.source;
    if q_jinx.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
