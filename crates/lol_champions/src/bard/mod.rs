pub mod buffs;

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

#[derive(Default)]
pub struct PluginBard;

impl Plugin for PluginBard {
    fn build(&self, app: &mut App) {
        app.add_observer(on_bard_skill_cast);
        app.add_observer(on_bard_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Bard"))]
#[reflect(Component)]
pub struct Bard;

fn on_bard_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_bard.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_bard_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_bard_w(&mut commands, entity),
        SkillSlot::E => cast_bard_e(&mut commands, entity),
        SkillSlot::R => cast_bard_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_bard_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Bard_Q_Cast"));

    // Q is a binding missile
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
        Some(hash_bin("Bard_Q_Hit")),
    );
}

fn cast_bard_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Bard_W_Cast"));
    // W is a heal shrine - no direct damage
}

fn cast_bard_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Bard_E_Cast"));
    // E is a tunnel - no direct damage
}

fn cast_bard_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Bard_R_Cast"));
    // R is a global AoE stun
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 3400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Bard_R_Hit")),
    );
}

fn on_bard_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
) {
    let source = trigger.source;
    if q_bard.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 1.5));
}
