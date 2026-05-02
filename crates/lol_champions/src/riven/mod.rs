pub mod passive;
pub mod q;

#[cfg(test)]
mod tests;
// NOTE: render_tests.rs requires moon_lol::PluginCore which creates a circular
// dependency (lol_champions -> lol_core/lol_render -> moon_lol -> lol_champions).
// Render tests for Riven remain in tests/riven.rs for now.

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, play_skill_animation,
    skill_damage, skill_dash, spawn_skill_particle,
};

use crate::riven::passive::BuffRivenPassive;

const RIVEN_Q_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginRiven;

impl Plugin for PluginRiven {
    fn build(&self, app: &mut App) {
        app.add_observer(on_riven_skill_cast);
        app.add_observer(passive::on_damage_create_trigger_bonus);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Riven"))]
#[reflect(Component)]
pub struct Riven;

fn on_riven_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_riven_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
            skill.spell.clone(),
        ),
        SkillSlot::W => cast_riven_w(&mut commands, entity, skill.spell.clone()),
        SkillSlot::E => cast_riven_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        SkillSlot::R => cast_riven_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_riven_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    let stage = recast.map(|window| window.stage).unwrap_or(1);

    let (animation_hash, particle_hash) = match stage {
        1 => ("Spell1A".to_string(), hash_bin("Riven_Q_01_Detonate")),
        2 => ("Spell1B".to_string(), hash_bin("Riven_Q_02_Detonate")),
        _ => ("Spell1C".to_string(), hash_bin("Riven_Q_03_Detonate")),
    };

    play_skill_animation(commands, entity, animation_hash);
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Fixed(250.0),
            damage: Some(DashDamage {
                radius_end: 250.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("FirstSlashDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1000.0,
        },
    );
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRivenPassive);
    spawn_skill_particle(commands, entity, particle_hash);

    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    } else {
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            stage + 1,
            3,
            RIVEN_Q_RECAST_WINDOW,
        ));
    }
}

fn cast_riven_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    spawn_skill_particle(commands, entity, hash_bin("Riven_W_Cast"));
    play_skill_animation(commands, entity, "spell2".to_string());
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        None,
    );
}

fn cast_riven_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    spawn_skill_particle(commands, entity, hash_bin("Riven_E_Mis"));
    play_skill_animation(commands, entity, "spell3".to_string());
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Fixed(250.0),
            damage: None,
            speed: 1000.0,
        },
    );
}

fn cast_riven_r(commands: &mut Commands, entity: Entity) {
    spawn_skill_particle(commands, entity, hash_bin("Riven_R_Indicator_Ring"));
    spawn_skill_particle(commands, entity, hash_bin("Riven_R_ALL_Warning"));
}
