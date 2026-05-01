pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, reset_skill_attack, skill_damage,
    skill_dash, spawn_skill_particle,
};

use crate::sett::buffs::BuffSettQ;

#[derive(Default)]
pub struct PluginSett;

impl Plugin for PluginSett {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sett_skill_cast);
        app.add_observer(on_sett_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sett"))]
#[reflect(Component)]
pub struct Sett;

fn on_sett_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_sett_q(&mut commands, entity),
        SkillSlot::W => cast_sett_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_sett_e(&mut commands, entity, skill.spell.clone()),
        SkillSlot::R => cast_sett_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        _ => {}
    }
}

fn cast_sett_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Sett_Q_Cast"));
    // Q is a buff that enhances next 2 attacks with bonus damage and move speed
    reset_skill_attack(commands, entity);
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSettQ::new(2, 0.3, 4.0));
}

fn cast_sett_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Sett_W_Cast"));
    // W deals true damage in a cone and grants shield based on damage taken
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 350.0,
            angle: 75.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::True,
        }],
        Some(hash_bin("Sett_W_Hit")),
    );
    // Grit converts to shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}

fn cast_sett_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Sett_E_Cast"));
    // E is a两边拉扯 that damages and stuns enemies caught by both sides
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 300.0,
            angle: 90.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Sett_E_Hit")),
    );
}

fn cast_sett_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Sett_R_Cast"));
    // R is a dash that carries enemy to target location and deals damage
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 400.0 },
            damage: Some(DashDamage {
                radius_end: 200.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 700.0,
        },
    );
}

/// 监听 Sett 造成的伤害，E/R 命中时眩晕
fn on_sett_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
) {
    let source = trigger.source;
    if q_sett.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // E/R 命中时眩晕
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}
