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

use crate::tristana::buffs::BuffTristanaW;

#[derive(Default)]
pub struct PluginTristana;

impl Plugin for PluginTristana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tristana_skill_cast);
        app.add_observer(on_tristana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Tristana"))]
#[reflect(Component)]
pub struct Tristana;

fn on_tristana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_tristana_q(&mut commands, entity),
        SkillSlot::W => cast_tristana_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_tristana_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_tristana_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_tristana_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_Q_Cast"));
}

fn cast_tristana_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Tristana_W_Hit")),
    );
}

fn cast_tristana_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_E_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 700.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Tristana_E_Hit")),
    );
}

fn cast_tristana_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_R_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 700.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Tristana_R_Hit")),
    );
}

fn on_tristana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
) {
    let source = trigger.source;
    if q_tristana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTristanaW::new(0.5, 2.0));
}
