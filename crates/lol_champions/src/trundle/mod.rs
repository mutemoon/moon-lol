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

use crate::trundle::buffs::BuffTrundleQ;

#[derive(Default)]
pub struct PluginTrundle;

impl Plugin for PluginTrundle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_trundle_skill_cast);
        app.add_observer(on_trundle_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Trundle"))]
#[reflect(Component)]
pub struct Trundle;

fn on_trundle_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_trundle.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_trundle_q(&mut commands, entity),
        SkillSlot::W => cast_trundle_w(&mut commands, entity),
        SkillSlot::E => cast_trundle_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_trundle_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_trundle_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_Q_Cast"));
}

fn cast_trundle_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_W_Cast"));
}

fn cast_trundle_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_E_Cast"));

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
        Some(hash_bin("Trundle_E_Hit")),
    );
}

fn cast_trundle_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 650.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Trundle_R_Hit")),
    );
}

fn on_trundle_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
) {
    let source = trigger.source;
    if q_trundle.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTrundleQ::new(0.4, 2.0));
}
