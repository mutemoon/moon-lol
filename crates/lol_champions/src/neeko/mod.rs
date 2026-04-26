pub mod buffs;

use bevy::prelude::{Handle, *};
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

use crate::neeko::buffs::BuffNeekoE;

#[derive(Default)]
pub struct PluginNeeko;

impl Plugin for PluginNeeko {
    fn build(&self, app: &mut App) {
        app.add_observer(on_neeko_skill_cast);
        app.add_observer(on_neeko_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Neeko"))]
#[reflect(Component)]
pub struct Neeko;

fn on_neeko_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_neeko.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_neeko_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_neeko_w(&mut commands, entity),
        SkillSlot::E => cast_neeko_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_neeko_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_neeko_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Neeko_Q_Cast"));

    // Q is a bloom burst
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
        Some(hash_bin("Neeko_Q_Hit")),
    );
}

fn cast_neeko_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Neeko_W_Cast"));

    // W is a shapesplitter dash
}

fn cast_neeko_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Neeko_E_Cast"));

    // E is a root
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1000.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Neeko_E_Hit")),
    );
}

fn cast_neeko_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Neeko_R_Cast"));

    // R is an AoE knockup/stun
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 590.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Neeko_R_Hit")),
    );
}

fn on_neeko_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
) {
    let source = trigger.source;
    if q_neeko.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNeekoE::new(1.5, 2.0));
}
