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

use crate::tryndamere::buffs::BuffTryndamereW;

#[derive(Default)]
pub struct PluginTryndamere;

impl Plugin for PluginTryndamere {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tryndamere_skill_cast);
        app.add_observer(on_tryndamere_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Tryndamere"))]
#[reflect(Component)]
pub struct Tryndamere;

fn on_tryndamere_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_tryndamere.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_tryndamere_q(&mut commands, entity),
        SkillSlot::W => cast_tryndamere_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_tryndamere_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_tryndamere_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_tryndamere_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Tryndamere_Q_Cast"));
}

fn cast_tryndamere_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Tryndamere_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 850.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Tryndamere_W_Hit")),
    );
}

fn cast_tryndamere_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Tryndamere_E_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 660.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Tryndamere_E_Hit")),
    );
}

fn cast_tryndamere_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Tryndamere_R_Cast"));
}

fn on_tryndamere_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tryndamere: Query<(), With<Tryndamere>>,
) {
    let source = trigger.source;
    if q_tryndamere.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTryndamereW::new(0.35, 20.0, 2.0));
}
