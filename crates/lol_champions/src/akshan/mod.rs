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

use crate::akshan::buffs::BuffAkshanPassive;

#[derive(Default)]
pub struct PluginAkshan;

impl Plugin for PluginAkshan {
    fn build(&self, app: &mut App) {
        app.add_observer(on_akshan_skill_cast);
        app.add_observer(on_akshan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Akshan"))]
#[reflect(Component)]
pub struct Akshan;

fn on_akshan_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_akshan.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_akshan_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_akshan_w(&mut commands, entity),
        SkillSlot::E => cast_akshan_e(&mut commands, entity),
        SkillSlot::R => cast_akshan_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_akshan_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Akshan_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 850.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Akshan_Q_Hit")),
    );
}

fn cast_akshan_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Akshan_W_Cast"));
}

fn cast_akshan_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Akshan_E_Cast"));
}

fn cast_akshan_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Akshan_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 2500.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Akshan_R_Hit")),
    );
}

fn on_akshan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
) {
    let source = trigger.source;
    if q_akshan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAkshanPassive::new(1, 15.0, 3.0));
}
