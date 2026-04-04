use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::sona_buffs::{BuffSonaE, BuffSonaW};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const SONA_Q_KEY: &str = "Characters/Sona/Spells/SonaQ/SonaQ";
#[allow(dead_code)]
const SONA_W_KEY: &str = "Characters/Sona/Spells/SonaW/SonaW";
#[allow(dead_code)]
const SONA_E_KEY: &str = "Characters/Sona/Spells/SonaE/SonaE";
const SONA_R_KEY: &str = "Characters/Sona/Spells/SonaR/SonaR";

#[derive(Default)]
pub struct PluginSona;

impl Plugin for PluginSona {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_sona_skill_cast);
        app.add_observer(on_sona_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sona"))]
#[reflect(Component)]
pub struct Sona;

fn on_sona_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_sona.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_sona_q(&mut commands, entity),
        SkillSlot::W => cast_sona_w(&mut commands, entity),
        SkillSlot::E => cast_sona_e(&mut commands, entity),
        SkillSlot::R => cast_sona_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_sona_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Sona_Q_Cast"));

    // Q is hymn of valor - damage
    skill_damage(
        commands,
        entity,
        SONA_Q_KEY,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Sona_Q_Hit")),
    );
}

fn cast_sona_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Sona_W_Cast"));

    // W is aria of perseverance - shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSonaW::new(40.0, 1.5));
}

fn cast_sona_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Sona_E_Cast"));

    // E is song of celerity - movespeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSonaE::new(0.2, 2.5));
}

fn cast_sona_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Sona_R_Cast"));

    // R is cure - AoE stun
    skill_damage(
        commands,
        entity,
        SONA_R_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 40.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Sona_R_Hit")),
    );
}

fn on_sona_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
) {
    let source = trigger.source;
    if q_sona.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // R stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSonaE::new(0.2, 2.5));
}

fn add_skills(
    mut commands: Commands,
    q_sona: Query<Entity, (With<Sona>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_sona.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Sona/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Sona/Spells/SonaPassive/SonaPassive",
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
