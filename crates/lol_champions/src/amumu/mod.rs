pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::amumu::buffs::{BuffAmumuPassive, BuffAmumuR};

const AMUMU_Q_KEY: &str = "Characters/Amumu/Spells/AmumuQ/AmumuQ";
const AMUMU_W_KEY: &str = "Characters/Amumu/Spells/AmumuW/AmumuW";
const AMUMU_E_KEY: &str = "Characters/Amumu/Spells/AmumuE/AmumuE";
const AMUMU_R_KEY: &str = "Characters/Amumu/Spells/AmumuR/AmumuR";

#[derive(Default)]
pub struct PluginAmumu;

impl Plugin for PluginAmumu {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_amumu_skill_cast);
        app.add_observer(on_amumu_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Amumu"))]
#[reflect(Component)]
pub struct Amumu;

fn on_amumu_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_amumu.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_amumu_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_amumu_w(&mut commands, entity),
        SkillSlot::E => cast_amumu_e(&mut commands, entity),
        SkillSlot::R => cast_amumu_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_amumu_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Amumu_Q_Cast"));

    // Q is a targeted dash that stuns
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: AMUMU_Q_KEY.into(),
            move_type: DashMoveType::Pointer { max: 1100.0 },
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

fn cast_amumu_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Amumu_W_Cast"));

    // W is toggle damage around self
    skill_damage(
        commands,
        entity,
        AMUMU_W_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Amumu_W_Hit")),
    );
}

fn cast_amumu_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Amumu_E_Cast"));

    // E is area damage
    skill_damage(
        commands,
        entity,
        AMUMU_E_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Amumu_E_Hit")),
    );
}

fn cast_amumu_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Amumu_R_Cast"));

    // R is area stun
    skill_damage(
        commands,
        entity,
        AMUMU_R_KEY,
        DamageShape::Circle { radius: 550.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Amumu_R_Hit")),
    );

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.5));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAmumuR::new());
}

fn on_amumu_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
) {
    let source = trigger.source;
    if q_amumu.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns target
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
    // Apply passive - Cursed Touch
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAmumuPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_amumu: Query<Entity, (With<Amumu>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_amumu.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Amumu/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Amumu/Spells/AmumuPassive/AmumuPassive",
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
