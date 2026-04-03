use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::tahm_kench_buffs::BuffTahmKenchE;
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const TAHM_KENCH_Q_KEY: &str = "Characters/TahmKench/Spells/TahmKenchQ/TahmKenchQ";
const TAHM_KENCH_W_KEY: &str = "Characters/TahmKench/Spells/TahmKenchW/TahmKenchW";
#[allow(dead_code)]
const TAHM_KENCH_E_KEY: &str = "Characters/TahmKench/Spells/TahmKenchE/TahmKenchE";
#[allow(dead_code)]
const TAHM_KENCH_R_KEY: &str = "Characters/TahmKench/Spells/TahmKenchR/TahmKenchR";

#[derive(Default)]
pub struct PluginTahmKench;

impl Plugin for PluginTahmKench {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_tahm_kench_skill_cast);
        app.add_observer(on_tahm_kench_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("TahmKench"))]
#[reflect(Component)]
pub struct TahmKench;

fn on_tahm_kench_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tahm_kench: Query<(), With<TahmKench>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_tahm_kench.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_tahm_kench_q(&mut commands, entity),
        SkillSlot::W => cast_tahm_kench_w(&mut commands, entity),
        SkillSlot::E => cast_tahm_kench_e(&mut commands, entity),
        SkillSlot::R => cast_tahm_kench_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_tahm_kench_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("TahmKench_Q_Cast"));

    skill_damage(
        commands,
        entity,
        TAHM_KENCH_Q_KEY,
        DamageShape::Nearest {
            max_distance: 900.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("TahmKench_Q_Hit")),
    );
}

fn cast_tahm_kench_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("TahmKench_W_Cast"));

    skill_damage(
        commands,
        entity,
        TAHM_KENCH_W_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("TahmKench_W_Hit")),
    );
}

fn cast_tahm_kench_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("TahmKench_E_Cast"));

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffTahmKenchE::new(100.0, 2.0));
}

fn cast_tahm_kench_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("TahmKench_R_Cast"));
}

fn on_tahm_kench_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tahm_kench: Query<(), With<TahmKench>>,
) {
    let source = trigger.source;
    if q_tahm_kench.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTahmKenchE::new(100.0, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_tahm_kench: Query<Entity, (With<TahmKench>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_tahm_kench.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/TahmKench/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/TahmKench/Spells/TahmKenchPassive/TahmKenchPassive",
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
