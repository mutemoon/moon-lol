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

use crate::smolder::buffs::BuffSmolderW;

#[derive(Default)]
pub struct PluginSmolder;

impl Plugin for PluginSmolder {
    fn build(&self, app: &mut App) {
        app.add_observer(on_smolder_skill_cast);
        app.add_observer(on_smolder_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Smolder"))]
#[reflect(Component)]
pub struct Smolder;

fn on_smolder_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_smolder.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_smolder_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_smolder_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_smolder_e(&mut commands, entity),
        SkillSlot::R => cast_smolder_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_smolder_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Smolder_Q_Cast"));

    // Q is searing strike - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 550.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Smolder_Q_Hit")),
    );
}

fn cast_smolder_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Smolder_W_Cast"));

    // W is deep fire brand - damage and slow
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Smolder_W_Hit")),
    );
}

fn cast_smolder_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Smolder_E_Cast"));

    // E is super hot - movespeed
}

fn cast_smolder_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Smolder_R_Cast"));

    // R is dragonfire storm - AoE damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 1200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Smolder_R_Hit")),
    );
}

fn on_smolder_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
) {
    let source = trigger.source;
    if q_smolder.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSmolderW::new(0.3, 1.5));
}
