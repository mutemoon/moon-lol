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

use crate::quinn::buffs::{BuffQuinnE, BuffQuinnW};

#[derive(Default)]
pub struct PluginQuinn;

impl Plugin for PluginQuinn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_quinn_skill_cast);
        app.add_observer(on_quinn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Quinn"))]
#[reflect(Component)]
pub struct Quinn;

fn on_quinn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_quinn.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_quinn_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_quinn_w(&mut commands, entity),
        SkillSlot::E => cast_quinn_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_quinn_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_quinn_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Quinn_Q_Cast"));

    // Q is blinding assault - damage and blind
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1050.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Quinn_Q_Hit")),
    );
}

fn cast_quinn_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Quinn_W_Cast"));

    // W is heightened senses - attackspeed and movespeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffQuinnW::new(0.8, 0.4, 2.0));
}

fn cast_quinn_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Quinn_E_Cast"));

    // E is vault - knockback and slow
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
        Some(hash_bin("Quinn_E_Hit")),
    );
}

fn cast_quinn_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Quinn_R_Cast"));

    // R is behind enemy lines - high movespeed
}

fn on_quinn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
) {
    let source = trigger.source;
    if q_quinn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffQuinnE::new(0.5, 1.5));
}
