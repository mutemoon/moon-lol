pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::seraphine::buffs::{BuffSeraphineE, BuffSeraphineW};

const SERAPHINE_Q_KEY: &str = "Characters/Seraphine/Spells/SeraphineQ/SeraphineQ";
#[allow(dead_code)]
const SERAPHINE_W_KEY: &str = "Characters/Seraphine/Spells/SeraphineW/SeraphineW";
const SERAPHINE_E_KEY: &str = "Characters/Seraphine/Spells/SeraphineE/SeraphineE";
const SERAPHINE_R_KEY: &str = "Characters/Seraphine/Spells/SeraphineR/SeraphineR";

#[derive(Default)]
pub struct PluginSeraphine;

impl Plugin for PluginSeraphine {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_seraphine_skill_cast);
        app.add_observer(on_seraphine_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Seraphine"))]
#[reflect(Component)]
pub struct Seraphine;

fn on_seraphine_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_seraphine.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_seraphine_q(&mut commands, entity),
        SkillSlot::W => cast_seraphine_w(&mut commands, entity),
        SkillSlot::E => cast_seraphine_e(&mut commands, entity),
        SkillSlot::R => cast_seraphine_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_seraphine_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Seraphine_Q_Cast"));

    // Q is high note - damage
    skill_damage(
        commands,
        entity,
        SERAPHINE_Q_KEY,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Seraphine_Q_Hit")),
    );
}

fn cast_seraphine_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Seraphine_W_Cast"));

    // W is solo - shield and slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSeraphineW::new(50.0, 2.5));
}

fn cast_seraphine_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Seraphine_E_Cast"));

    // E is beat drop - stun
    skill_damage(
        commands,
        entity,
        SERAPHINE_E_KEY,
        DamageShape::Sector {
            radius: 1300.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Seraphine_E_Hit")),
    );
}

fn cast_seraphine_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Seraphine_R_Cast"));

    // R is encore - AoE charm
    skill_damage(
        commands,
        entity,
        SERAPHINE_R_KEY,
        DamageShape::Sector {
            radius: 1500.0,
            angle: 50.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Seraphine_R_Hit")),
    );
}

fn on_seraphine_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
) {
    let source = trigger.source;
    if q_seraphine.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSeraphineE::new(0.75, 1.0));
}

fn add_skills(
    mut commands: Commands,
    q_seraphine: Query<Entity, (With<Seraphine>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_seraphine.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Seraphine/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Seraphine/Spells/SeraphinePassive/SeraphinePassive",
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
