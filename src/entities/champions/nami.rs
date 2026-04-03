use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::nami_buffs::{BuffNamiE, BuffNamiQ};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const NAMI_Q_KEY: &str = "Characters/Nami/Spells/NamiQ/NamiQ";
const NAMI_W_KEY: &str = "Characters/Nami/Spells/NamiW/NamiW";
#[allow(dead_code)]
const NAMI_E_KEY: &str = "Characters/Nami/Spells/NamiE/NamiE";
const NAMI_R_KEY: &str = "Characters/Nami/Spells/NamiR/NamiR";

#[derive(Default)]
pub struct PluginNami;

impl Plugin for PluginNami {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_nami_skill_cast);
        app.add_observer(on_nami_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nami"))]
#[reflect(Component)]
pub struct Nami;

fn on_nami_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nami.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_nami_q(&mut commands, entity),
        SkillSlot::W => cast_nami_w(&mut commands, entity),
        SkillSlot::E => cast_nami_e(&mut commands, entity),
        SkillSlot::R => cast_nami_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_nami_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Nami_Q_Cast"));

    // Q is a bubble that roots
    skill_damage(
        commands,
        entity,
        NAMI_Q_KEY,
        DamageShape::Circle { radius: 850.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nami_Q_Hit")),
    );
}

fn cast_nami_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Nami_W_Cast"));

    // W bounces between allies and enemies
    skill_damage(
        commands,
        entity,
        NAMI_W_KEY,
        DamageShape::Circle { radius: 725.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nami_W_Hit")),
    );
}

fn cast_nami_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Nami_E_Cast"));

    // E buffs allied attacks with bonus damage and slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNamiE::new(30.0, 0.3, 6.0));
}

fn cast_nami_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Nami_R_Cast"));

    // R is a giant wave that knocks up
    skill_damage(
        commands,
        entity,
        NAMI_R_KEY,
        DamageShape::Sector {
            radius: 2750.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nami_R_Hit")),
    );
}

fn on_nami_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
) {
    let source = trigger.source;
    if q_nami.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNamiQ::new(1.5, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_nami: Query<Entity, (With<Nami>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_nami.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Nami/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Nami/Spells/NamiPassive/NamiPassive",
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
