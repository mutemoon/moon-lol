use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::katarina_buffs::{BuffKatarinaVoracity, BuffKatarinaW};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const KATARINA_Q_KEY: &str = "Characters/Katarina/Spells/KatarinaQ/KatarinaQ";
#[allow(dead_code)]
const KATARINA_W_KEY: &str = "Characters/Katarina/Spells/KatarinaW/KatarinaW";
const KATARINA_E_KEY: &str = "Characters/Katarina/Spells/KatarinaE/KatarinaE";
const KATARINA_R_KEY: &str = "Characters/Katarina/Spells/KatarinaR/KatarinaR";

#[derive(Default)]
pub struct PluginKatarina;

impl Plugin for PluginKatarina {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_katarina_skill_cast);
        app.add_observer(on_katarina_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Katarina"))]
#[reflect(Component)]
pub struct Katarina;

fn on_katarina_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_katarina.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_katarina_q(&mut commands, entity),
        SkillSlot::W => cast_katarina_w(&mut commands, entity),
        SkillSlot::E => cast_katarina_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_katarina_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_katarina_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_Q_Cast"));

    // Q bounces between enemies
    skill_damage(
        commands,
        entity,
        KATARINA_Q_KEY,
        DamageShape::Circle { radius: 625.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Katarina_Q_Hit")),
    );
}

fn cast_katarina_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_W_Cast"));

    // W throws dagger up and grants movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKatarinaW::new(0.8, 2.0));
}

fn cast_katarina_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_E_Cast"));

    // E is a dash to target
    skill_damage(
        commands,
        entity,
        KATARINA_E_KEY,
        DamageShape::Circle { radius: 100.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Katarina_E_Hit")),
    );
}

fn cast_katarina_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Katarina_R_Cast"));

    // R throws daggers in area
    skill_damage(
        commands,
        entity,
        KATARINA_R_KEY,
        DamageShape::Circle { radius: 550.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Katarina_R_Hit")),
    );
}

fn on_katarina_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
) {
    let source = trigger.source;
    if q_katarina.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive: kill reduces cooldowns
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKatarinaVoracity::new(15.0, 3.0));
}

fn add_skills(
    mut commands: Commands,
    q_katarina: Query<Entity, (With<Katarina>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_katarina.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Katarina/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Katarina/Spells/KatarinaPassive/KatarinaPassive",
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
