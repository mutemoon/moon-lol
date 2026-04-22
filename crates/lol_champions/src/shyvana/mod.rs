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

use crate::shyvana::buffs::BuffShyvanaE;

#[derive(Default)]
pub struct PluginShyvana;

impl Plugin for PluginShyvana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shyvana_skill_cast);
        app.add_observer(on_shyvana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shyvana"))]
#[reflect(Component)]
pub struct Shyvana;

fn on_shyvana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_shyvana_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_shyvana_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_shyvana_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_shyvana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_shyvana_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_Q_Cast"));

    // Q is twin bite - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Shyvana_Q_Hit")),
    );
}

fn cast_shyvana_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_W_Cast"));

    // W is flame breath - damage over time
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 600.0,
            angle: 25.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Shyvana_W_Hit")),
    );
}

fn cast_shyvana_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_E_Cast"));

    // E is dragon descent - knockup
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 450.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Shyvana_E_Hit")),
    );
}

fn cast_shyvana_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_R_Cast"));

    // R is shape shift - transformation
}

fn on_shyvana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
) {
    let source = trigger.source;
    if q_shyvana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShyvanaE::new(0.5, 1.0));
}
