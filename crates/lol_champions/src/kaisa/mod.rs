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
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::kaisa::buffs::{BuffKaisaE, BuffKaisaPlasma, BuffKaisaR};

#[derive(Default)]
pub struct PluginKaisa;

impl Plugin for PluginKaisa {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kaisa_skill_cast);
        app.add_observer(on_kaisa_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kaisa"))]
#[reflect(Component)]
pub struct Kaisa;

fn on_kaisa_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kaisa.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_kaisa_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_kaisa_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_kaisa_e(&mut commands, entity),
        SkillSlot::R => cast_kaisa_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        _ => {}
    }
}

fn cast_kaisa_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_Q_Cast"));

    // Q fires 6 missiles that spread to nearby enemies
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kaisa_Q_Hit")),
    );
}

fn cast_kaisa_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_W_Cast"));

    // W is a long-range missile that applies 2 plasma stacks
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 3000.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kaisa_W_Hit")),
    );
}

fn cast_kaisa_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_E_Cast"));

    // E charges movement speed then grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaisaE::new(0.8, 4.0));
}

fn cast_kaisa_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_R_Cast"));

    // R is a dash to a plasma-marked enemy with shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaisaR::new(100.0, 4.0));

    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 2000.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1500.0,
        },
    );
}

fn on_kaisa_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
) {
    let source = trigger.source;
    if q_kaisa.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply plasma stacks (Q applies 1, W applies 2)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKaisaPlasma::new(1, 5.0));
}
