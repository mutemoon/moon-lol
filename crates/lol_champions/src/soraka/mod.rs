pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
};

use crate::soraka::buffs::BuffSorakaE;

const SORAKA_Q_KEY: &str = "Characters/Soraka/Spells/SorakaQ/SorakaQ";
#[allow(dead_code)]
const SORAKA_W_KEY: &str = "Characters/Soraka/Spells/SorakaW/SorakaW";
const SORAKA_E_KEY: &str = "Characters/Soraka/Spells/SorakaE/SorakaE";
#[allow(dead_code)]
const SORAKA_R_KEY: &str = "Characters/Soraka/Spells/SorakaR/SorakaR";

#[derive(Default)]
pub struct PluginSoraka;

impl Plugin for PluginSoraka {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_soraka_skill_cast);
        app.add_observer(on_soraka_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Soraka"))]
#[reflect(Component)]
pub struct Soraka;

fn on_soraka_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_soraka.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_soraka_q(&mut commands, entity),
        SkillSlot::W => cast_soraka_w(&mut commands, entity),
        SkillSlot::E => cast_soraka_e(&mut commands, entity),
        SkillSlot::R => cast_soraka_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_soraka_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_Q_Cast"));

    // Q is starlon fallback - damage
    skill_damage(
        commands,
        entity,
        SORAKA_Q_KEY,
        DamageShape::Circle { radius: 575.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Soraka_Q_Hit")),
    );
}

fn cast_soraka_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_W_Cast"));

    // W is infuse magic - heal
}

fn cast_soraka_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_E_Cast"));

    // E is barrier of mind - silence
    skill_damage(
        commands,
        entity,
        SORAKA_E_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Soraka_E_Hit")),
    );
}

fn cast_soraka_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Soraka_R_Cast"));

    // R is wishes - global heal
}

fn on_soraka_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
) {
    let source = trigger.source;
    if q_soraka.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E silences
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSorakaE::new(0.5, 1.0));
}

fn add_skills(
    mut commands: Commands,
    q_soraka: Query<Entity, (With<Soraka>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_soraka.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Soraka/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Soraka/Spells/SorakaPassive/SorakaPassive",
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
