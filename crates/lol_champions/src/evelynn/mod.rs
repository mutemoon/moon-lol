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

const EVELYNN_Q_KEY: &str = "Characters/Evelynn/Spells/EvelynnQ/EvelynnQ";
#[allow(dead_code)]
const EVELYNN_W_KEY: &str = "Characters/Evelynn/Spells/EvelynnW/EvelynnW";
const EVELYNN_E_KEY: &str = "Characters/Evelynn/Spells/EvelynnE/EvelynnE";
const EVELYNN_R_KEY: &str = "Characters/Evelynn/Spells/EvelynnR/EvelynnR";

#[derive(Default)]
pub struct PluginEvelynn;

impl Plugin for PluginEvelynn {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_evelynn_skill_cast);
        app.add_observer(on_evelynn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Evelynn"))]
#[reflect(Component)]
pub struct Evelynn;

fn on_evelynn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_evelynn.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_evelynn_q(&mut commands, entity),
        SkillSlot::W => cast_evelynn_w(&mut commands, entity),
        SkillSlot::E => cast_evelynn_e(&mut commands, entity),
        SkillSlot::R => cast_evelynn_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_evelynn_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Evelynn_Q_Cast"));

    // Q is a skillshot
    skill_damage(
        commands,
        entity,
        EVELYNN_Q_KEY,
        DamageShape::Sector {
            radius: 800.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Evelynn_Q_Hit")),
    );
}

fn cast_evelynn_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Evelynn_W_Cast"));
    // W is a charm/slow - handled by damage observer
}

fn cast_evelynn_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Evelynn_E_Cast"));

    // E is targeted damage
    skill_damage(
        commands,
        entity,
        EVELYNN_E_KEY,
        DamageShape::Nearest {
            max_distance: 210.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Evelynn_E_Hit")),
    );
}

fn cast_evelynn_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Evelynn_R_Cast"));

    // R is AoE damage with execute
    skill_damage(
        commands,
        entity,
        EVELYNN_R_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Evelynn_R_Hit")),
    );
}

fn on_evelynn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
) {
    let source = trigger.source;
    if q_evelynn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows then charms
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.45, 2.5));
}

fn add_skills(
    mut commands: Commands,
    q_evelynn: Query<Entity, (With<Evelynn>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_evelynn.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Evelynn/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Evelynn/Spells/EvelynnPassive/EvelynnPassive",
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
