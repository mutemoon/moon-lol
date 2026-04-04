use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::buffs::janna_buffs::BuffJannaPassive;

const JANNA_Q_KEY: &str = "Characters/Janna/Spells/JannaQ/JannaQ";
const JANNA_W_KEY: &str = "Characters/Janna/Spells/JannaW/JannaW";
#[allow(dead_code)]
const JANNA_E_KEY: &str = "Characters/Janna/Spells/JannaE/JannaE";
const JANNA_R_KEY: &str = "Characters/Janna/Spells/JannaR/JannaR";

#[derive(Default)]
pub struct PluginJanna;

impl Plugin for PluginJanna {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_janna_skill_cast);
        app.add_observer(on_janna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Janna"))]
#[reflect(Component)]
pub struct Janna;

fn on_janna_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_janna: Query<(), With<Janna>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_janna.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_janna_q(&mut commands, entity),
        SkillSlot::W => cast_janna_w(&mut commands, entity),
        SkillSlot::E => cast_janna_e(&mut commands, entity),
        SkillSlot::R => cast_janna_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_janna_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Janna_Q_Cast"));

    // Q is a tornado
    skill_damage(
        commands,
        entity,
        JANNA_Q_KEY,
        DamageShape::Sector {
            radius: 1760.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Janna_Q_Hit")),
    );
}

fn cast_janna_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Janna_W_Cast"));

    // W is targeted damage and slow
    skill_damage(
        commands,
        entity,
        JANNA_W_KEY,
        DamageShape::Nearest {
            max_distance: 550.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Janna_W_Hit")),
    );
}

fn cast_janna_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Janna_E_Cast"));
    // E is a shield
}

fn cast_janna_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Janna_R_Cast"));

    // R is AoE knockback
    skill_damage(
        commands,
        entity,
        JANNA_R_KEY,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Janna_R_Hit")),
    );
}

fn on_janna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_janna: Query<(), With<Janna>>,
) {
    let source = trigger.source;
    if q_janna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJannaPassive::new());
    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_janna: Query<Entity, (With<Janna>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_janna.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Janna/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Janna/Spells/JannaPassive/JannaPassive",
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
