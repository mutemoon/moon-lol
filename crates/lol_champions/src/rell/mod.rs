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

use crate::rell::buffs::{BuffRellE, BuffRellR, BuffRellW};

#[derive(Default)]
pub struct PluginRell;

impl Plugin for PluginRell {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rell_skill_cast);
        app.add_observer(on_rell_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rell"))]
#[reflect(Component)]
pub struct Rell;

fn on_rell_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rell.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rell_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_rell_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_rell_e(&mut commands, entity, skill.spell.clone()),
        SkillSlot::R => cast_rell_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_rell_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rell_Q_Cast"));

    // Q is shattering strike - damage and armor reduction
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 500.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rell_Q_Hit")),
    );
}

fn cast_rell_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rell_W_Cast"));

    // W is crashing blow - damage and knockup
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
        Some(hash_bin("Rell_W_Hit")),
    );
}

fn cast_rell_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rell_E_Cast"));

    // E is full bind - stun
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rell_E_Hit")),
    );
}

fn cast_rell_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rell_R_Cast"));

    // R is catharsis - AoE damage and slow
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
        Some(hash_bin("Rell_R_Hit")),
    );
}

fn on_rell_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
) {
    let source = trigger.source;
    if q_rell.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellW::new(0.5, 1.5));
    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellE::new(0.75, 1.0));
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellR::new(0.4, 2.0));
}
