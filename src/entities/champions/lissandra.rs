use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::lissandra_buffs::{BuffLissandraQ, BuffLissandraR, BuffLissandraW};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const LISSANDRA_Q_KEY: &str = "Characters/Lissandra/Spells/LissandraQ/LissandraQ";
const LISSANDRA_W_KEY: &str = "Characters/Lissandra/Spells/LissandraW/LissandraW";
const LISSANDRA_E_KEY: &str = "Characters/Lissandra/Spells/LissandraE/LissandraE";
const LISSANDRA_R_KEY: &str = "Characters/Lissandra/Spells/LissandraR/LissandraR";

#[derive(Default)]
pub struct PluginLissandra;

impl Plugin for PluginLissandra {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_lissandra_skill_cast);
        app.add_observer(on_lissandra_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lissandra"))]
#[reflect(Component)]
pub struct Lissandra;

fn on_lissandra_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lissandra.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_lissandra_q(&mut commands, entity),
        SkillSlot::W => cast_lissandra_w(&mut commands, entity),
        SkillSlot::E => cast_lissandra_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_lissandra_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_lissandra_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_Q_Cast"));

    // Q is a piercing ice shard that slows
    skill_damage(
        commands,
        entity,
        LISSANDRA_Q_KEY,
        DamageShape::Sector {
            radius: 825.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_Q_Hit")),
    );
}

fn cast_lissandra_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_W_Cast"));

    // W is a circle root
    skill_damage(
        commands,
        entity,
        LISSANDRA_W_KEY,
        DamageShape::Circle { radius: 275.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_W_Hit")),
    );
}

fn cast_lissandra_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_E_Cast"));

    // E is a dash-like skill
    skill_damage(
        commands,
        entity,
        LISSANDRA_E_KEY,
        DamageShape::Sector {
            radius: 1025.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_E_Hit")),
    );
}

fn cast_lissandra_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Lissandra_R_Cast"));

    // R can self-cast for shield or enemy cast for damage+root
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLissandraR::new(true, 100.0, 2.5));

    skill_damage(
        commands,
        entity,
        LISSANDRA_R_KEY,
        DamageShape::Circle { radius: 550.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lissandra_R_Hit")),
    );
}

fn on_lissandra_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lissandra: Query<(), With<Lissandra>>,
) {
    let source = trigger.source;
    if q_lissandra.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLissandraQ::new(0.3, 3.0));

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLissandraW::new(1.5, 3.0));
}

fn add_skills(
    mut commands: Commands,
    q_lissandra: Query<Entity, (With<Lissandra>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_lissandra.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Lissandra/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Lissandra/Spells/LissandraPassive/LissandraPassive",
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
