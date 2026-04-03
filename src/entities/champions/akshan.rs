use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, BuffOf,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill, SkillOf, SkillSlot, Skills,
    TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffAkshanPassive, DamageType, PassiveSkillOf};

const AKSHAN_Q_KEY: &str = "Characters/Akshan/Spells/AkshanQ/AkshanQ";
const AKSHAN_W_KEY: &str = "Characters/Akshan/Spells/AkshanW/AkshanW";
const AKSHAN_E_KEY: &str = "Characters/Akshan/Spells/AkshanE/AkshanE";
const AKSHAN_R_KEY: &str = "Characters/Akshan/Spells/AkshanR/AkshanR";

#[derive(Default)]
pub struct PluginAkshan;

impl Plugin for PluginAkshan {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_akshan_skill_cast);
        app.add_observer(on_akshan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Akshan"))]
#[reflect(Component)]
pub struct Akshan;

fn on_akshan_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_akshan.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_akshan_q(&mut commands, entity),
        SkillSlot::W => cast_akshan_w(&mut commands, entity),
        SkillSlot::E => cast_akshan_e(&mut commands, entity),
        SkillSlot::R => cast_akshan_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_akshan_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Akshan_Q_Cast"));

    skill_damage(
        commands,
        entity,
        AKSHAN_Q_KEY,
        DamageShape::Sector { radius: 850.0, angle: 20.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Akshan_Q_Hit")),
    );
}

fn cast_akshan_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Akshan_W_Cast"));
}

fn cast_akshan_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Akshan_E_Cast"));
}

fn cast_akshan_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Akshan_R_Cast"));

    skill_damage(
        commands,
        entity,
        AKSHAN_R_KEY,
        DamageShape::Nearest { max_distance: 2500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Akshan_R_Hit")),
    );
}

fn on_akshan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
) {
    let source = trigger.source;
    if q_akshan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands.entity(target).with_related::<BuffOf>(BuffAkshanPassive::new(1, 15.0, 3.0));
}

fn add_skills(
    mut commands: Commands,
    q_akshan: Query<Entity, (With<Akshan>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_akshan.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Akshan/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Akshan/Spells/AkshanPassive/AkshanPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
