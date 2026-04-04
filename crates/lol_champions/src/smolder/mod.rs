pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::smolder::buffs::BuffSmolderW;

const SMOLDER_Q_KEY: &str = "Characters/Smolder/Spells/SmolderQ/SmolderQ";
const SMOLDER_W_KEY: &str = "Characters/Smolder/Spells/SmolderW/SmolderW";
#[allow(dead_code)]
const SMOLDER_E_KEY: &str = "Characters/Smolder/Spells/SmolderE/SmolderE";
const SMOLDER_R_KEY: &str = "Characters/Smolder/Spells/SmolderR/SmolderR";

#[derive(Default)]
pub struct PluginSmolder;

impl Plugin for PluginSmolder {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_smolder_skill_cast);
        app.add_observer(on_smolder_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Smolder"))]
#[reflect(Component)]
pub struct Smolder;

fn on_smolder_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_smolder.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_smolder_q(&mut commands, entity),
        SkillSlot::W => cast_smolder_w(&mut commands, entity),
        SkillSlot::E => cast_smolder_e(&mut commands, entity),
        SkillSlot::R => cast_smolder_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_smolder_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Smolder_Q_Cast"));

    // Q is searing strike - damage
    skill_damage(
        commands,
        entity,
        SMOLDER_Q_KEY,
        DamageShape::Nearest {
            max_distance: 550.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Smolder_Q_Hit")),
    );
}

fn cast_smolder_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Smolder_W_Cast"));

    // W is deep fire brand - damage and slow
    skill_damage(
        commands,
        entity,
        SMOLDER_W_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Smolder_W_Hit")),
    );
}

fn cast_smolder_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Smolder_E_Cast"));

    // E is super hot - movespeed
}

fn cast_smolder_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Smolder_R_Cast"));

    // R is dragonfire storm - AoE damage
    skill_damage(
        commands,
        entity,
        SMOLDER_R_KEY,
        DamageShape::Circle { radius: 1200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Smolder_R_Hit")),
    );
}

fn on_smolder_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
) {
    let source = trigger.source;
    if q_smolder.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSmolderW::new(0.3, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_smolder: Query<Entity, (With<Smolder>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_smolder.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Smolder/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Smolder/Spells/SmolderPassive/SmolderPassive",
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
