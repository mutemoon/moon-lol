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

use crate::ornn::buffs::BuffOrnnQ;

#[derive(Default)]
pub struct PluginOrnn;

impl Plugin for PluginOrnn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ornn_skill_cast);
        app.add_observer(on_ornn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ornn"))]
#[reflect(Component)]
pub struct Ornn;

fn on_ornn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ornn.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_ornn_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_ornn_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_ornn_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_ornn_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_ornn_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ornn_Q_Cast"));

    // Q is volcanic rupture - damage and slow
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 750.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Ornn_Q_Hit")),
    );

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOrnnQ::new(0.4, 2.0));
}

fn cast_ornn_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ornn_W_Cast"));

    // W is bellows breath - continuous damage and brittle
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 500.0,
            angle: 25.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ornn_W_Hit")),
    );
}

fn cast_ornn_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ornn_E_Cast"));

    // E is searing charge - dash that creates shockwave on terrain hit
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Ornn_E_Hit")),
    );
}

fn cast_ornn_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ornn_R_Cast"));

    // R is call of the forge god - large AoE damage and knockup
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 3000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ornn_R_Hit")),
    );
}

fn on_ornn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
) {
    let source = trigger.source;
    if q_ornn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffOrnnQ::new(0.4, 2.0));
}
