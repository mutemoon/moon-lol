pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
};

use crate::blitzcrank::buffs::BuffBlitzcrankW;

const BLITZCRANK_Q_KEY: &str =
    "Characters/Blitzcrank/Spells/BlitzcrankRocketGrab/BlitzcrankRocketGrab";
#[allow(dead_code)]
const BLITZCRANK_W_KEY: &str =
    "Characters/Blitzcrank/Spells/BlitzcrankOverdrive/BlitzcrankOverdrive";
const BLITZCRANK_E_KEY: &str =
    "Characters/Blitzcrank/Spells/BlitzcrankPowerFist/BlitzcrankPowerFist";
const BLITZCRANK_R_KEY: &str =
    "Characters/Blitzcrank/Spells/BlitzcrankStaticField/BlitzcrankStaticField";

#[derive(Default)]
pub struct PluginBlitzcrank;

impl Plugin for PluginBlitzcrank {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_blitzcrank_skill_cast);
        app.add_observer(on_blitzcrank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Blitzcrank"))]
#[reflect(Component)]
pub struct Blitzcrank;

fn on_blitzcrank_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_blitzcrank_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_blitzcrank_w(&mut commands, entity),
        SkillSlot::E => cast_blitzcrank_e(&mut commands, entity),
        SkillSlot::R => cast_blitzcrank_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_blitzcrank_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_Q_Cast"));

    // Q is a hook that pulls enemy
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: BLITZCRANK_Q_KEY.into(),
            move_type: DashMoveType::Pointer { max: 1115.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::Champion,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 900.0,
        },
    );
}

fn cast_blitzcrank_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_W_Cast"));

    // W grants movement and attack speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffBlitzcrankW::new());
}

fn cast_blitzcrank_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_E_Cast"));

    // E is an empowered attack that knocks up
    skill_damage(
        commands,
        entity,
        BLITZCRANK_E_KEY,
        DamageShape::Nearest {
            max_distance: 100.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Blitzcrank_E_Hit")),
    );
}

fn cast_blitzcrank_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Blitzcrank_R_Cast"));

    // R is an AoE that silences
    skill_damage(
        commands,
        entity,
        BLITZCRANK_R_KEY,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Blitzcrank_R_Hit")),
    );
}

fn on_blitzcrank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
) {
    let source = trigger.source;
    if q_blitzcrank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns on hit
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.65));
}

fn add_skills(
    mut commands: Commands,
    q_blitzcrank: Query<Entity, (With<Blitzcrank>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_blitzcrank.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Blitzcrank/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Blitzcrank/Spells/BlitzcrankPassive/BlitzcrankPassive",
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
