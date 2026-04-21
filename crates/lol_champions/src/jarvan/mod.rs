pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
};

const JARVAN_Q_KEY: &str = "Characters/JarvanIV/Spells/JarvanIVQ/JarvanIVQ";
const JARVAN_W_KEY: &str = "Characters/JarvanIV/Spells/JarvanIVW/JarvanIVW";
#[allow(dead_code)]
const JARVAN_E_KEY: &str = "Characters/JarvanIV/Spells/JarvanIVE/JarvanIVE";
const JARVAN_R_KEY: &str = "Characters/JarvanIV/Spells/JarvanIVR/JarvanIVR";

#[derive(Default)]
pub struct PluginJarvan;

impl Plugin for PluginJarvan {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_jarvan_skill_cast);
        app.add_observer(on_jarvan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("JarvanIV"))]
#[reflect(Component)]
pub struct JarvanIV;

fn on_jarvan_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jarvan.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_jarvan_q(&mut commands, entity),
        SkillSlot::W => cast_jarvan_w(&mut commands, entity),
        SkillSlot::E => cast_jarvan_e(&mut commands, entity),
        SkillSlot::R => cast_jarvan_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_jarvan_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Jarvan_Q_Cast"));

    // Q is a line damage and armor shred
    skill_damage(
        commands,
        entity,
        JARVAN_Q_KEY,
        DamageShape::Sector {
            radius: 785.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jarvan_Q_Hit")),
    );
}

fn cast_jarvan_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Jarvan_W_Cast"));

    // W is an AoE slow
    skill_damage(
        commands,
        entity,
        JARVAN_W_KEY,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Jarvan_W_Hit")),
    );
}

fn cast_jarvan_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Jarvan_E_Cast"));

    // E grants attack speed aura
}

fn cast_jarvan_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Jarvan_R_Cast"));

    // R is a targeted dash that creates arena
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: JARVAN_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 650.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 800.0,
        },
    );
}

fn on_jarvan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
) {
    let source = trigger.source;
    if q_jarvan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_jarvan: Query<Entity, (With<JarvanIV>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_jarvan.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/JarvanIV/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/JarvanIV/Spells/JarvanIVPassive/JarvanIVPassive",
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
