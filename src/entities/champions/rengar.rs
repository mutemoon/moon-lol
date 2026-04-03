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
use crate::{BuffRengarE, BuffRengarR, DamageType, PassiveSkillOf};

#[allow(dead_code)]
const RENGAR_Q_KEY: &str = "Characters/Rengar/Spells/RengarQ/RengarQ";
const RENGAR_W_KEY: &str = "Characters/Rengar/Spells/RengarW/RengarW";
const RENGAR_E_KEY: &str = "Characters/Rengar/Spells/RengarE/RengarE";
#[allow(dead_code)]
const RENGAR_R_KEY: &str = "Characters/Rengar/Spells/RengarR/RengarR";

#[derive(Default)]
pub struct PluginRengar;

impl Plugin for PluginRengar {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_rengar_skill_cast);
        app.add_observer(on_rengar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rengar"))]
#[reflect(Component)]
pub struct Rengar;

fn on_rengar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rengar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rengar_q(&mut commands, entity),
        SkillSlot::W => cast_rengar_w(&mut commands, entity),
        SkillSlot::E => cast_rengar_e(&mut commands, entity),
        SkillSlot::R => cast_rengar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_rengar_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Rengar_Q_Cast"));

    // Q is savagery - enhanced attack
}

fn cast_rengar_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Rengar_W_Cast"));

    // W is battle roar - AoE damage and heal
    skill_damage(
        commands,
        entity,
        RENGAR_W_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rengar_W_Hit")),
    );
}

fn cast_rengar_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Rengar_E_Cast"));

    // E is bola strike - damage and slow
    skill_damage(
        commands,
        entity,
        RENGAR_E_KEY,
        DamageShape::Sector { radius: 1000.0, angle: 15.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Rengar_E_Hit")),
    );
}

fn cast_rengar_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Rengar_R_Cast"));

    // R is thrill of the hunt - camouflage and movespeed
}

fn on_rengar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
) {
    let source = trigger.source;
    if q_rengar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands.entity(target).with_related::<BuffOf>(BuffRengarE::new(0.4, 2.25));
    // R gives movespeed
    commands.entity(target).with_related::<BuffOf>(BuffRengarR::new(0.5, 14.0));
}

fn add_skills(
    mut commands: Commands,
    q_rengar: Query<Entity, (With<Rengar>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_rengar.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Rengar/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Rengar/Spells/RengarPassive/RengarPassive",
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
