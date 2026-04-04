use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::alistar_buffs::BuffAlistarR;
use crate::buffs::cc_debuffs::DebuffStun;
use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::action::dash::{ActionDash, DashDamage, DashMoveType};
use crate::base::buff::BuffOf;
use crate::damage::{DamageType, EventDamageCreate};
use crate::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const ALISTAR_Q_KEY: &str = "Characters/Alistar/Spells/AlistarQ/AlistarQ";
const ALISTAR_W_KEY: &str = "Characters/Alistar/Spells/AlistarW/AlistarW";
const ALISTAR_E_KEY: &str = "Characters/Alistar/Spells/AlistarE/AlistarE";
#[allow(dead_code)]
const ALISTAR_R_KEY: &str = "Characters/Alistar/Spells/AlistarR/AlistarR";

#[derive(Default)]
pub struct PluginAlistar;

impl Plugin for PluginAlistar {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_alistar_skill_cast);
        app.add_observer(on_alistar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Alistar"))]
#[reflect(Component)]
pub struct Alistar;

fn on_alistar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_alistar_q(&mut commands, entity),
        SkillSlot::W => cast_alistar_w(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::E => cast_alistar_e(&mut commands, entity),
        SkillSlot::R => cast_alistar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_alistar_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Alistar_Q_Cast"));

    // Q is a knockup and stun in area
    skill_damage(
        commands,
        entity,
        ALISTAR_Q_KEY,
        DamageShape::Circle { radius: 375.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Alistar_Q_Hit")),
    );

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}

fn cast_alistar_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Alistar_W_Cast"));

    // W is a dash that knocks back target
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: ALISTAR_W_KEY.into(),
            move_type: DashMoveType::Pointer { max: 650.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        },
    );
}

fn cast_alistar_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Alistar_E_Cast"));

    // E is area damage that stuns on 5th hit
    skill_damage(
        commands,
        entity,
        ALISTAR_E_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Alistar_E_Hit")),
    );
}

fn cast_alistar_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Alistar_R_Cast"));

    // R grants damage reduction
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAlistarR::new());
}

fn on_alistar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
) {
    let source = trigger.source;
    if q_alistar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W stuns and knocks back
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.75));
}

fn add_skills(
    mut commands: Commands,
    q_alistar: Query<Entity, (With<Alistar>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_alistar.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Alistar/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Alistar/Spells/AlistarPassive/AlistarPassive",
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
