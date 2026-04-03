use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::morgana_buffs::{BuffMorganaE, BuffMorganaQ};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const MORGANA_Q_KEY: &str = "Characters/Morgana/Spells/MorganaQ/MorganaQ";
const MORGANA_W_KEY: &str = "Characters/Morgana/Spells/MorganaW/MorganaW";
#[allow(dead_code)]
const MORGANA_E_KEY: &str = "Characters/Morgana/Spells/MorganaE/MorganaE";
const MORGANA_R_KEY: &str = "Characters/Morgana/Spells/MorganaR/MorganaR";

#[derive(Default)]
pub struct PluginMorgana;

impl Plugin for PluginMorgana {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_morgana_skill_cast);
        app.add_observer(on_morgana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Morgana"))]
#[reflect(Component)]
pub struct Morgana;

fn on_morgana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morgana: Query<(), With<Morgana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_morgana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_morgana_q(&mut commands, entity),
        SkillSlot::W => cast_morgana_w(&mut commands, entity),
        SkillSlot::E => cast_morgana_e(&mut commands, entity),
        SkillSlot::R => cast_morgana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_morgana_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Morgana_Q_Cast"));

    // Q binds enemy
    skill_damage(
        commands,
        entity,
        MORGANA_Q_KEY,
        DamageShape::Sector {
            radius: 1300.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Morgana_Q_Hit")),
    );
}

fn cast_morgana_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Morgana_W_Cast"));

    // W is a DoT zone
    skill_damage(
        commands,
        entity,
        MORGANA_W_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Morgana_W_Hit")),
    );
}

fn cast_morgana_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Morgana_E_Cast"));

    // E is a shield that blocks CC
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMorganaE::new(150.0, true, 5.0));
}

fn cast_morgana_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Morgana_R_Cast"));

    // R chains nearby enemies
    skill_damage(
        commands,
        entity,
        MORGANA_R_KEY,
        DamageShape::Circle { radius: 625.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Morgana_R_Hit")),
    );
}

fn on_morgana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morgana: Query<(), With<Morgana>>,
) {
    let source = trigger.source;
    if q_morgana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMorganaQ::new(2.0, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_morgana: Query<Entity, (With<Morgana>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_morgana.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Morgana/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Morgana/Spells/MorganaPassive/MorganaPassive",
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
