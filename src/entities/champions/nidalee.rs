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
use crate::{BuffNidaleeE, BuffNidaleeQ, DamageType, PassiveSkillOf};

const NIDALEE_Q_KEY: &str = "Characters/Nidalee/Spells/NidaleeQ/NidaleeQ";
const NIDALEE_W_KEY: &str = "Characters/Nidalee/Spells/NidaleeW/NidaleeW";
const NIDALEE_E_KEY: &str = "Characters/Nidalee/Spells/NidaleeE/NidaleeE";
#[allow(dead_code)]
const NIDALEE_R_KEY: &str = "Characters/Nidalee/Spells/NidaleeR/NidaleeR";

#[derive(Default)]
pub struct PluginNidalee;

impl Plugin for PluginNidalee {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_nidalee_skill_cast);
        app.add_observer(on_nidalee_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nidalee"))]
#[reflect(Component)]
pub struct Nidalee;

fn on_nidalee_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nidalee.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_nidalee_q(&mut commands, entity),
        SkillSlot::W => cast_nidalee_w(&mut commands, entity),
        SkillSlot::E => cast_nidalee_e(&mut commands, entity),
        SkillSlot::R => cast_nidalee_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_nidalee_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Nidalee_Q_Cast"));

    // Q is a spear (human form)
    skill_damage(
        commands,
        entity,
        NIDALEE_Q_KEY,
        DamageShape::Sector { radius: 1500.0, angle: 10.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nidalee_Q_Hit")),
    );
}

fn cast_nidalee_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Nidalee_W_Cast"));

    // W is a trap (human form) or pounce (cougar form)
    skill_damage(
        commands,
        entity,
        NIDALEE_W_KEY,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nidalee_W_Hit")),
    );
}

fn cast_nidalee_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Nidalee_E_Cast"));

    // E is a heal (human form) or swipe (cougar form)
    commands.entity(entity).with_related::<BuffOf>(BuffNidaleeE::new(100.0, 0.7, 7.0));

    skill_damage(
        commands,
        entity,
        NIDALEE_E_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nidalee_E_Hit")),
    );
}

fn cast_nidalee_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Nidalee_R_Cast"));

    // R transforms between human and cougar forms
}

fn on_nidalee_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
) {
    let source = trigger.source;
    if q_nidalee.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q marks target
    commands.entity(target).with_related::<BuffOf>(BuffNidaleeQ::new(100.0, 4.0));
}

fn add_skills(
    mut commands: Commands,
    q_nidalee: Query<Entity, (With<Nidalee>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_nidalee.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Nidalee/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Nidalee/Spells/NidaleePassive/NidaleePassive",
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
