pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::kalista::buffs::{BuffKalistaE, BuffKalistaR};

const KALISTA_Q_KEY: &str = "Characters/Kalista/Spells/KalistaQ/KalistaQ";
#[allow(dead_code)]
const KALISTA_W_KEY: &str = "Characters/Kalista/Spells/KalistaW/KalistaW";
const KALISTA_E_KEY: &str = "Characters/Kalista/Spells/KalistaE/KalistaE";
const KALISTA_R_KEY: &str = "Characters/Kalista/Spells/KalistaR/KalistaR";

#[derive(Default)]
pub struct PluginKalista;

impl Plugin for PluginKalista {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kalista_skill_cast);
        app.add_observer(on_kalista_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kalista"))]
#[reflect(Component)]
pub struct Kalista;

fn on_kalista_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kalista: Query<(), With<Kalista>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kalista.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kalista_q(&mut commands, entity),
        SkillSlot::W => cast_kalista_w(&mut commands, entity),
        SkillSlot::E => cast_kalista_e(&mut commands, entity),
        SkillSlot::R => cast_kalista_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_kalista_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kalista_Q_Cast"));

    // Q is a spear that passes through enemies
    skill_damage(
        commands,
        entity,
        KALISTA_Q_KEY,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kalista_Q_Hit")),
    );
}

fn cast_kalista_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kalista_W_Cast"));

    // W is a sentinel that provides vision and deals damage on basic attacks
}

fn cast_kalista_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kalista_E_Cast"));

    // E deals damage to speared enemies and slows
    skill_damage(
        commands,
        entity,
        KALISTA_E_KEY,
        DamageShape::Circle { radius: 1100.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kalista_E_Hit")),
    );
}

fn cast_kalista_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kalista_R_Cast"));

    // R pulls oathsworn ally and grants invulnerability
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKalistaR::new(4.0));

    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: KALISTA_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 1200.0 },
            damage: None,
            speed: 1000.0,
        },
    );
}

fn on_kalista_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kalista: Query<(), With<Kalista>>,
) {
    let source = trigger.source;
    if q_kalista.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKalistaE::new(0.3, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_kalista: Query<Entity, (With<Kalista>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kalista.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kalista/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kalista/Spells/KalistaPassive/KalistaPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
