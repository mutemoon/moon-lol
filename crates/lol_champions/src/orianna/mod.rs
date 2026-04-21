pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
};

use crate::orianna::buffs::BuffOriannaE;

const ORIANNA_Q_KEY: &str = "Characters/Orianna/Spells/OriannaQ/OriannaQ";
const ORIANNA_W_KEY: &str = "Characters/Orianna/Spells/OriannaW/OriannaW";
const ORIANNA_E_KEY: &str = "Characters/Orianna/Spells/OriannaE/OriannaE";
const ORIANNA_R_KEY: &str = "Characters/Orianna/Spells/OriannaR/OriannaR";

#[derive(Default)]
pub struct PluginOrianna;

impl Plugin for PluginOrianna {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_orianna_skill_cast);
        app.add_observer(on_orianna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Orianna"))]
#[reflect(Component)]
pub struct Orianna;

fn on_orianna_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_orianna.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_orianna_q(&mut commands, entity),
        SkillSlot::W => cast_orianna_w(&mut commands, entity),
        SkillSlot::E => cast_orianna_e(&mut commands, entity),
        SkillSlot::R => cast_orianna_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_orianna_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Orianna_Q_Cast"));

    // Q commands ball to location
    skill_damage(
        commands,
        entity,
        ORIANNA_Q_KEY,
        DamageShape::Circle { radius: 825.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Orianna_Q_Hit")),
    );
}

fn cast_orianna_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Orianna_W_Cast"));

    // W deals damage and slows
    skill_damage(
        commands,
        entity,
        ORIANNA_W_KEY,
        DamageShape::Circle { radius: 225.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Orianna_W_Hit")),
    );
}

fn cast_orianna_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Orianna_E_Cast"));

    // E shields ball carrier
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOriannaE::new(100.0, 30.0, 4.0));

    skill_damage(
        commands,
        entity,
        ORIANNA_E_KEY,
        DamageShape::Circle { radius: 1120.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Orianna_E_Hit")),
    );
}

fn cast_orianna_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Orianna_R_Cast"));

    // R is a shockwave that knocks back
    skill_damage(
        commands,
        entity,
        ORIANNA_R_KEY,
        DamageShape::Circle { radius: 415.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Orianna_R_Hit")),
    );
}

fn on_orianna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
) {
    let source = trigger.source;
    if q_orianna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_orianna: Query<Entity, (With<Orianna>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_orianna.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Orianna/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Orianna/Spells/OriannaPassive/OriannaPassive",
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
