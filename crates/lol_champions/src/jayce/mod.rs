pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::jayce::buffs::BuffJaycePassive;

const JAYCE_Q_KEY: &str = "Characters/Jayce/Spells/JayceQ/JayceQ";
const JAYCE_W_KEY: &str = "Characters/Jayce/Spells/JayceW/JayceW";
const JAYCE_E_KEY: &str = "Characters/Jayce/Spells/JayceE/JayceE";
#[allow(dead_code)]
const JAYCE_R_KEY: &str = "Characters/Jayce/Spells/JayceR/JayceR";

#[derive(Default)]
pub struct PluginJayce;

impl Plugin for PluginJayce {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_jayce_skill_cast);
        app.add_observer(on_jayce_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jayce"))]
#[reflect(Component)]
pub struct Jayce;

fn on_jayce_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_jayce_q(&mut commands, entity),
        SkillSlot::W => cast_jayce_w(&mut commands, entity),
        SkillSlot::E => cast_jayce_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_jayce_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_jayce_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_Q_Cast"));

    // Q is a skillshot
    skill_damage(
        commands,
        entity,
        JAYCE_Q_KEY,
        DamageShape::Sector {
            radius: 1050.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jayce_Q_Hit")),
    );
}

fn cast_jayce_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_W_Cast"));

    // W is an area slow
    skill_damage(
        commands,
        entity,
        JAYCE_W_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Jayce_W_Hit")),
    );
}

fn cast_jayce_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_E_Cast"));

    // E is a knockback
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: JAYCE_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 500.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 1000.0,
        },
    );
}

fn cast_jayce_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Jayce_R_Cast"));
    // R transforms between hammer and cannon forms
}

fn on_jayce_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
) {
    let source = trigger.source;
    if q_jayce.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJaycePassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_jayce: Query<Entity, (With<Jayce>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_jayce.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Jayce/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Jayce/Spells/JaycePassive/JaycePassive",
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
