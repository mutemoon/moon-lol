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

use crate::singed::buffs::BuffSingedE;

#[allow(dead_code)]
const SINGED_Q_KEY: &str = "Characters/Singed/Spells/SingedQ/SingedQ";
const SINGED_W_KEY: &str = "Characters/Singed/Spells/SingedW/SingedW";
const SINGED_E_KEY: &str = "Characters/Singed/Spells/SingedE/SingedE";
#[allow(dead_code)]
const SINGED_R_KEY: &str = "Characters/Singed/Spells/SingedR/SingedR";

#[derive(Default)]
pub struct PluginSinged;

impl Plugin for PluginSinged {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_singed_skill_cast);
        app.add_observer(on_singed_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Singed"))]
#[reflect(Component)]
pub struct Singed;

fn on_singed_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_singed_q(&mut commands, entity),
        SkillSlot::W => cast_singed_w(&mut commands, entity),
        SkillSlot::E => cast_singed_e(&mut commands, entity),
        SkillSlot::R => cast_singed_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_singed_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_Q_Cast"));

    // Q is poison trail - damage over time
}

fn cast_singed_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_W_Cast"));

    // W is mega adhesive - slow
    skill_damage(
        commands,
        entity,
        SINGED_W_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Singed_W_Hit")),
    );
}

fn cast_singed_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_E_Cast"));

    // E is fling - damage
    skill_damage(
        commands,
        entity,
        SINGED_E_KEY,
        DamageShape::Nearest {
            max_distance: 400.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Singed_E_Hit")),
    );
}

fn cast_singed_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Singed_R_Cast"));

    // R is insanity - movespeed buff
}

fn on_singed_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
) {
    let source = trigger.source;
    if q_singed.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSingedE::new(0.6, 3.0));
}

fn add_skills(
    mut commands: Commands,
    q_singed: Query<Entity, (With<Singed>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_singed.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Singed/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Singed/Spells/SingedPassive/SingedPassive",
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
