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

use crate::teemo::buffs::BuffTeemoQ;

#[derive(Default)]
pub struct PluginTeemo;

impl Plugin for PluginTeemo {
    fn build(&self, app: &mut App) {
        app.add_observer(on_teemo_skill_cast);
        app.add_observer(on_teemo_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Teemo"))]
#[reflect(Component)]
pub struct Teemo;

fn on_teemo_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_teemo: Query<(), With<Teemo>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_teemo.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_teemo_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_teemo_w(&mut commands, entity),
        SkillSlot::E => cast_teemo_e(&mut commands, entity),
        SkillSlot::R => cast_teemo_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_teemo_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 680.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Teemo_Q_Hit")),
    );
}

fn cast_teemo_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_W_Cast"));
}

fn cast_teemo_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_E_Cast"));
}

fn cast_teemo_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Teemo_R_Hit")),
    );
}

fn on_teemo_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_teemo: Query<(), With<Teemo>>,
) {
    let source = trigger.source;
    if q_teemo.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTeemoQ::new(1.25, 1.5));
}
