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

use crate::missfortune::buffs::BuffMissFortuneW;

const MISSFORTUNE_Q_KEY: &str = "Characters/MissFortune/Spells/MissFortuneQ/MissFortuneQ";
#[allow(dead_code)]
const MISSFORTUNE_W_KEY: &str = "Characters/MissFortune/Spells/MissFortuneW/MissFortuneW";
const MISSFORTUNE_E_KEY: &str = "Characters/MissFortune/Spells/MissFortuneE/MissFortuneE";
const MISSFORTUNE_R_KEY: &str = "Characters/MissFortune/Spells/MissFortuneR/MissFortuneR";

#[derive(Default)]
pub struct PluginMissFortune;

impl Plugin for PluginMissFortune {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_missfortune_skill_cast);
        app.add_observer(on_missfortune_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("MissFortune"))]
#[reflect(Component)]
pub struct MissFortune;

fn on_missfortune_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_missfortune.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_missfortune_q(&mut commands, entity),
        SkillSlot::W => cast_missfortune_w(&mut commands, entity),
        SkillSlot::E => cast_missfortune_e(&mut commands, entity),
        SkillSlot::R => cast_missfortune_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_missfortune_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_Q_Cast"));

    // Q bounces to second target
    skill_damage(
        commands,
        entity,
        MISSFORTUNE_Q_KEY,
        DamageShape::Sector {
            radius: 550.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("MissFortune_Q_Hit")),
    );
}

fn cast_missfortune_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_W_Cast"));

    // W grants movespeed and attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMissFortuneW::new(0.6, 1.0, 4.0));
}

fn cast_missfortune_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_E_Cast"));

    // E is a zone that slows
    skill_damage(
        commands,
        entity,
        MISSFORTUNE_E_KEY,
        DamageShape::Circle { radius: 1000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("MissFortune_E_Hit")),
    );
}

fn cast_missfortune_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("MissFortune_R_Cast"));

    // R is a cone of bullets
    skill_damage(
        commands,
        entity,
        MISSFORTUNE_R_KEY,
        DamageShape::Sector {
            radius: 1450.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("MissFortune_R_Hit")),
    );
}

fn on_missfortune_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
) {
    let source = trigger.source;
    if q_missfortune.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_missfortune: Query<Entity, (With<MissFortune>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_missfortune.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/MissFortune/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/MissFortune/Spells/MissFortunePassive/MissFortunePassive",
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
