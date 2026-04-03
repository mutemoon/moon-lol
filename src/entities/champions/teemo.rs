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
use crate::{BuffTeemoQ, DamageType, PassiveSkillOf};

const TEEMO_Q_KEY: &str = "Characters/Teemo/Spells/TeemoQ/TeemoQ";
#[allow(dead_code)]
const TEEMO_W_KEY: &str = "Characters/Teemo/Spells/TeemoW/TeemoW";
#[allow(dead_code)]
const TEEMO_E_KEY: &str = "Characters/Teemo/Spells/TeemoE/TeemoE";
const TEEMO_R_KEY: &str = "Characters/Teemo/Spells/TeemoR/TeemoR";

#[derive(Default)]
pub struct PluginTeemo;

impl Plugin for PluginTeemo {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_teemo_skill_cast);
        app.add_observer(on_teemo_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Teemo"))]
#[reflect(Component)]
pub struct Teemo;

fn on_teemo_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_teemo: Query<(), With<Teemo>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_teemo.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_teemo_q(&mut commands, entity),
        SkillSlot::W => cast_teemo_w(&mut commands, entity),
        SkillSlot::E => cast_teemo_e(&mut commands, entity),
        SkillSlot::R => cast_teemo_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_teemo_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_Q_Cast"));

    skill_damage(
        commands,
        entity,
        TEEMO_Q_KEY,
        DamageShape::Nearest { max_distance: 680.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Teemo_Q_Hit")),
    );
}

fn cast_teemo_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_W_Cast"));
}

fn cast_teemo_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_E_Cast"));
}

fn cast_teemo_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Teemo_R_Cast"));

    skill_damage(
        commands,
        entity,
        TEEMO_R_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Teemo_R_Hit")),
    );
}

fn on_teemo_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_teemo: Query<(), With<Teemo>>,
) {
    let source = trigger.source;
    if q_teemo.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands.entity(target).with_related::<BuffOf>(BuffTeemoQ::new(1.25, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_teemo: Query<Entity, (With<Teemo>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_teemo.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Teemo/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Teemo/Spells/TeemoPassive/TeemoPassive",
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
