pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::alistar::buffs::BuffAlistarR;

#[derive(Default)]
pub struct PluginAlistar;

impl Plugin for PluginAlistar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_alistar_skill_cast);
        app.add_observer(on_alistar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Alistar"))]
#[reflect(Component)]
pub struct Alistar;

fn on_alistar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_alistar_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_alistar_w(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::E => cast_alistar_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_alistar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_alistar_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Alistar_Q_Cast"));

    // Q is a knockup and stun in area
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 375.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Alistar_Q_Hit")),
    );

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}

fn cast_alistar_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Alistar_W_Cast"));

    // W is a dash that knocks back target
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 650.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        },
    );
}

fn cast_alistar_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Alistar_E_Cast"));

    // E is area damage that stuns on 5th hit
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Alistar_E_Hit")),
    );
}

fn cast_alistar_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Alistar_R_Cast"));

    // R grants damage reduction
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAlistarR::new());
}

fn on_alistar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
) {
    let source = trigger.source;
    if q_alistar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W stuns and knocks back
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.75));
}
