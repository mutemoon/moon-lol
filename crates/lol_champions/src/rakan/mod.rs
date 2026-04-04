pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::rakan::buffs::{BuffRakanR, BuffRakanW};

const RAKAN_Q_KEY: &str = "Characters/Rakan/Spells/RakanQ/RakanQ";
const RAKAN_W_KEY: &str = "Characters/Rakan/Spells/RakanW/RakanW";
#[allow(dead_code)]
const RAKAN_E_KEY: &str = "Characters/Rakan/Spells/RakanE/RakanE";
const RAKAN_R_KEY: &str = "Characters/Rakan/Spells/RakanR/RakanR";

#[derive(Default)]
pub struct PluginRakan;

impl Plugin for PluginRakan {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_rakan_skill_cast);
        app.add_observer(on_rakan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rakan"))]
#[reflect(Component)]
pub struct Rakan;

fn on_rakan_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rakan.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rakan_q(&mut commands, entity),
        SkillSlot::W => cast_rakan_w(&mut commands, entity),
        SkillSlot::E => cast_rakan_e(&mut commands, entity),
        SkillSlot::R => cast_rakan_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_rakan_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Rakan_Q_Cast"));

    // Q is gleaming quill - damage
    skill_damage(
        commands,
        entity,
        RAKAN_Q_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rakan_Q_Hit")),
    );
}

fn cast_rakan_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Rakan_W_Cast"));

    // W is grand entrance - knockup
    skill_damage(
        commands,
        entity,
        RAKAN_W_KEY,
        DamageShape::Circle { radius: 650.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rakan_W_Hit")),
    );
}

fn cast_rakan_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Rakan_E_Cast"));

    // E is battle dance - shield to ally
}

fn cast_rakan_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Rakan_R_Cast"));

    // R is the quickness - damage and charm
    skill_damage(
        commands,
        entity,
        RAKAN_R_KEY,
        DamageShape::Circle { radius: 150.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rakan_R_Hit")),
    );
}

fn on_rakan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
) {
    let source = trigger.source;
    if q_rakan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W knockup
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRakanW::new(1.0, 1.5));
    // R charm and slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRakanR::new(1.5, 0.75, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_rakan: Query<Entity, (With<Rakan>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_rakan.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Rakan/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Rakan/Spells/RakanPassive/RakanPassive",
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
