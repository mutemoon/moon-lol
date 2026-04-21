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
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
};

use crate::pyke::buffs::{BuffPykeE, BuffPykeQ};

const PYKE_Q_KEY: &str = "Characters/Pyke/Spells/PykeQ/PykeQ";
#[allow(dead_code)]
const PYKE_W_KEY: &str = "Characters/Pyke/Spells/PykeW/PykeW";
const PYKE_E_KEY: &str = "Characters/Pyke/Spells/PykeE/PykeE";
const PYKE_R_KEY: &str = "Characters/Pyke/Spells/PykeR/PykeR";

#[derive(Default)]
pub struct PluginPyke;

impl Plugin for PluginPyke {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_pyke_skill_cast);
        app.add_observer(on_pyke_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Pyke"))]
#[reflect(Component)]
pub struct Pyke;

fn on_pyke_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_pyke.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_pyke_q(&mut commands, entity),
        SkillSlot::W => cast_pyke_w(&mut commands, entity),
        SkillSlot::E => cast_pyke_e(&mut commands, entity),
        SkillSlot::R => cast_pyke_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_pyke_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Pyke_Q_Cast"));

    // Q is bone skewer - damage and pull
    skill_damage(
        commands,
        entity,
        PYKE_Q_KEY,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pyke_Q_Hit")),
    );
}

fn cast_pyke_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Pyke_W_Cast"));

    // W is ghostwater dive - invisibility and movespeed
}

fn cast_pyke_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Pyke_E_Cast"));

    // E is phantom undertow - dash and stun on return
    skill_damage(
        commands,
        entity,
        PYKE_E_KEY,
        DamageShape::Sector {
            radius: 550.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pyke_E_Hit")),
    );
}

fn cast_pyke_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Pyke_R_Cast"));

    // R is death from below - execute damage in AoE
    skill_damage(
        commands,
        entity,
        PYKE_R_KEY,
        DamageShape::Circle { radius: 750.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Pyke_R_Hit")),
    );
}

fn on_pyke_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
) {
    let source = trigger.source;
    if q_pyke.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffPykeQ::new(0.9, 1.0));
    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffPykeE::new(1.25, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_pyke: Query<Entity, (With<Pyke>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_pyke.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Pyke/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Pyke/Spells/PykePassive/PykePassive",
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
