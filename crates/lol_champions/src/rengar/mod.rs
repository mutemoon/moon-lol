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

use crate::rengar::buffs::{BuffRengarE, BuffRengarR};

#[derive(Default)]
pub struct PluginRengar;

impl Plugin for PluginRengar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rengar_skill_cast);
        app.add_observer(on_rengar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rengar"))]
#[reflect(Component)]
pub struct Rengar;

fn on_rengar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rengar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rengar_q(&mut commands, entity),
        SkillSlot::W => cast_rengar_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_rengar_e(&mut commands, entity, skill.spell.clone()),
        SkillSlot::R => cast_rengar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_rengar_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rengar_Q_Cast"));

    // Q is savagery - enhanced attack
}

fn cast_rengar_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rengar_W_Cast"));

    // W is battle roar - AoE damage and heal
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
        Some(hash_bin("Rengar_W_Hit")),
    );
}

fn cast_rengar_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rengar_E_Cast"));

    // E is bola strike - damage and slow
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1000.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Rengar_E_Hit")),
    );
}

fn cast_rengar_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Rengar_R_Cast"));

    // R is thrill of the hunt - camouflage and movespeed
}

fn on_rengar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
) {
    let source = trigger.source;
    if q_rengar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRengarE::new(0.4, 2.25));
    // R gives movespeed
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRengarR::new(0.5, 14.0));
}
