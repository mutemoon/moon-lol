pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::galio::buffs::{BuffGalioPassive, BuffGalioW};

#[derive(Default)]
pub struct PluginGalio;

impl Plugin for PluginGalio {
    fn build(&self, app: &mut App) {
        app.add_observer(on_galio_skill_cast);
        app.add_observer(on_galio_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Galio"))]
#[reflect(Component)]
pub struct Galio;

fn on_galio_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_galio_q(&mut commands, entity, skill.spell.clone()),
        SkillSlot::W => cast_galio_w(&mut commands, entity),
        SkillSlot::E => cast_galio_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        SkillSlot::R => cast_galio_r(&mut commands, entity, skill.spell.clone()),
        _ => {}
    }
}

fn cast_galio_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Galio_Q_Cast"));

    // Q is a tornado
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 825.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Galio_Q_Hit")),
    );
}

fn cast_galio_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Galio_W_Cast"));

    // W provides shield and reduces damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGalioW::new());
}

fn cast_galio_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Galio_E_Cast"));

    // E is a dash that knocks up
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell.clone(),
            move_type: DashMoveType::Pointer { max: 650.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 900.0,
        },
    );
}

fn cast_galio_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Galio_R_Cast"));

    // R is a large AoE knockback
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
        Some(hash_bin("Galio_R_Hit")),
    );
}

fn on_galio_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
) {
    let source = trigger.source;
    if q_galio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGalioPassive::new());
}
