pub mod buffs;

use bevy::asset::Handle;
use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::missfortune::buffs::BuffMissFortuneW;

#[derive(Default)]
pub struct PluginMissFortune;

impl Plugin for PluginMissFortune {
    fn build(&self, app: &mut App) {
        app.add_observer(on_missfortune_skill_cast);
        app.add_observer(on_missfortune_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("MissFortune"))]
#[reflect(Component)]
pub struct MissFortune;

fn on_missfortune_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_missfortune.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_missfortune_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_missfortune_w(&mut commands, entity),
        SkillSlot::E => cast_missfortune_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_missfortune_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_missfortune_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_Q_Cast"));

    // Q bounces to second target
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 550.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("MissFortune_Q_Hit")),
    );
}

fn cast_missfortune_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_W_Cast"));

    // W grants movespeed and attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMissFortuneW::new(0.6, 1.0, 4.0));
}

fn cast_missfortune_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_E_Cast"));

    // E is a zone that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 1000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("MissFortune_E_Hit")),
    );
}

fn cast_missfortune_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_R_Cast"));

    // R is a cone of bullets
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1450.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("MissFortune_R_Hit")),
    );
}

fn on_missfortune_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
) {
    let source = trigger.source;
    if q_missfortune.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
