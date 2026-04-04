use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::talon_buffs::BuffTalonW;
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const TALON_Q_KEY: &str = "Characters/Talon/Spells/TalonQ/TalonQ";
const TALON_W_KEY: &str = "Characters/Talon/Spells/TalonW/TalonW";
#[allow(dead_code)]
const TALON_E_KEY: &str = "Characters/Talon/Spells/TalonE/TalonE";
const TALON_R_KEY: &str = "Characters/Talon/Spells/TalonR/TalonR";

#[derive(Default)]
pub struct PluginTalon;

impl Plugin for PluginTalon {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_talon_skill_cast);
        app.add_observer(on_talon_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Talon"))]
#[reflect(Component)]
pub struct Talon;

fn on_talon_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_talon.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_talon_q(&mut commands, entity),
        SkillSlot::W => cast_talon_w(&mut commands, entity),
        SkillSlot::E => cast_talon_e(&mut commands, entity),
        SkillSlot::R => cast_talon_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_talon_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_Q_Cast"));

    skill_damage(
        commands,
        entity,
        TALON_Q_KEY,
        DamageShape::Nearest {
            max_distance: 600.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Talon_Q_Hit")),
    );
}

fn cast_talon_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_W_Cast"));

    skill_damage(
        commands,
        entity,
        TALON_W_KEY,
        DamageShape::Sector {
            radius: 600.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Talon_W_Hit")),
    );
}

fn cast_talon_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_E_Cast"));
}

fn cast_talon_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Talon_R_Cast"));

    skill_damage(
        commands,
        entity,
        TALON_R_KEY,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Talon_R_Hit")),
    );
}

fn on_talon_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
) {
    let source = trigger.source;
    if q_talon.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTalonW::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_talon: Query<Entity, (With<Talon>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_talon.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Talon/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Talon/Spells/TalonPassive/TalonPassive",
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
