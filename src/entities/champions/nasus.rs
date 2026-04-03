use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::nasus_buffs::BuffNasusW;
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

#[allow(dead_code)]
const NASUS_Q_KEY: &str = "Characters/Nasus/Spells/NasusQ/NasusQ";
const NASUS_W_KEY: &str = "Characters/Nasus/Spells/NasusW/NasusW";
const NASUS_E_KEY: &str = "Characters/Nasus/Spells/NasusE/NasusE";
const NASUS_R_KEY: &str = "Characters/Nasus/Spells/NasusR/NasusR";

#[derive(Default)]
pub struct PluginNasus;

impl Plugin for PluginNasus {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_nasus_skill_cast);
        app.add_observer(on_nasus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nasus"))]
#[reflect(Component)]
pub struct Nasus;

fn on_nasus_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nasus.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_nasus_q(&mut commands, entity),
        SkillSlot::W => cast_nasus_w(&mut commands, entity),
        SkillSlot::E => cast_nasus_e(&mut commands, entity),
        SkillSlot::R => cast_nasus_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_nasus_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Nasus_Q_Cast"));

    // Q is a siphoning strike
}

fn cast_nasus_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Nasus_W_Cast"));

    // W is a slow
    skill_damage(
        commands,
        entity,
        NASUS_W_KEY,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nasus_W_Hit")),
    );
}

fn cast_nasus_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Nasus_E_Cast"));

    // E is an area damage and armor reduction
    skill_damage(
        commands,
        entity,
        NASUS_E_KEY,
        DamageShape::Circle { radius: 650.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nasus_E_Hit")),
    );
}

fn cast_nasus_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Nasus_R_Cast"));

    // R transforms Nasus
    skill_damage(
        commands,
        entity,
        NASUS_R_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nasus_R_Hit")),
    );
}

fn on_nasus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
) {
    let source = trigger.source;
    if q_nasus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNasusW::new(0.5, 5.0));
}

fn add_skills(
    mut commands: Commands,
    q_nasus: Query<Entity, (With<Nasus>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_nasus.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Nasus/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Nasus/Spells/NasusPassive/NasusPassive",
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
