pub mod buffs;

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

use crate::caitlyn::buffs::BuffCaitlynPassive;

const CAITLYN_Q_KEY: &str = "Characters/Caitlyn/Spells/CaitlynQ/CaitlynQ";
#[allow(dead_code)]
const CAITLYN_W_KEY: &str = "Characters/Caitlyn/Spells/CaitlynW/CaitlynW";
const CAITLYN_E_KEY: &str = "Characters/Caitlyn/Spells/CaitlynE/CaitlynE";
const CAITLYN_R_KEY: &str = "Characters/Caitlyn/Spells/CaitlynR/CaitlynR";

#[derive(Default)]
pub struct PluginCaitlyn;

impl Plugin for PluginCaitlyn {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_caitlyn_skill_cast);
        app.add_observer(on_caitlyn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Caitlyn"))]
#[reflect(Component)]
pub struct Caitlyn;

fn on_caitlyn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_caitlyn.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_caitlyn_q(&mut commands, entity),
        SkillSlot::W => cast_caitlyn_w(&mut commands, entity),
        SkillSlot::E => cast_caitlyn_e(&mut commands, entity),
        SkillSlot::R => cast_caitlyn_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_caitlyn_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Caitlyn_Q_Cast"));

    // Q is a long range piercing shot
    skill_damage(
        commands,
        entity,
        CAITLYN_Q_KEY,
        DamageShape::Sector {
            radius: 1300.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Caitlyn_Q_Hit")),
    );
}

fn cast_caitlyn_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Caitlyn_W_Cast"));
    // W places traps - no direct damage
}

fn cast_caitlyn_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Caitlyn_E_Cast"));

    // E is a net that slows
    skill_damage(
        commands,
        entity,
        CAITLYN_E_KEY,
        DamageShape::Sector {
            radius: 800.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Caitlyn_E_Hit")),
    );
}

fn cast_caitlyn_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Caitlyn_R_Cast"));

    // R is a global targeted shot
    skill_damage(
        commands,
        entity,
        CAITLYN_R_KEY,
        DamageShape::Circle { radius: 3500.0 },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Caitlyn_R_Hit")),
    );
}

fn on_caitlyn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
) {
    let source = trigger.source;
    if q_caitlyn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.0));
    // Apply headshot passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCaitlynPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_caitlyn: Query<Entity, (With<Caitlyn>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_caitlyn.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Caitlyn/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Caitlyn/Spells/CaitlynPassive/CaitlynPassive",
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
