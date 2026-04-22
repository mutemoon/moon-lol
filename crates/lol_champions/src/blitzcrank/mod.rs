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

use crate::blitzcrank::buffs::BuffBlitzcrankW;

#[derive(Default)]
pub struct PluginBlitzcrank;

impl Plugin for PluginBlitzcrank {
    fn build(&self, app: &mut App) {
        app.add_observer(on_blitzcrank_skill_cast);
        app.add_observer(on_blitzcrank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Blitzcrank"))]
#[reflect(Component)]
pub struct Blitzcrank;

fn on_blitzcrank_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_blitzcrank_q(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::W => cast_blitzcrank_w(&mut commands, entity),
        SkillSlot::E => cast_blitzcrank_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_blitzcrank_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_blitzcrank_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_Q_Cast"));

    // Q is a hook that pulls enemy
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 1115.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::Champion,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 900.0,
        },
    );
}

fn cast_blitzcrank_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_W_Cast"));

    // W grants movement and attack speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffBlitzcrankW::new());
}

fn cast_blitzcrank_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_E_Cast"));

    // E is an empowered attack that knocks up
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 100.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Blitzcrank_E_Hit")),
    );
}

fn cast_blitzcrank_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_R_Cast"));

    // R is an AoE that silences
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Blitzcrank_R_Hit")),
    );
}

fn on_blitzcrank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
) {
    let source = trigger.source;
    if q_blitzcrank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns on hit
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.65));
}
