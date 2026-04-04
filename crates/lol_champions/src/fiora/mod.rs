pub mod e;
pub mod passive;
pub mod r;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::BuffAttack;
use lol_core::base::buff::BuffOf;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    despawn_skill_particle, play_skill_animation, reset_skill_attack, skill_damage, skill_dash,
    skill_slot_from_index, spawn_skill_particle, CoolDown, EventSkillCast, PassiveSkillOf, Skill,
    SkillOf, SkillSlot, Skills,
};

use crate::fiora::e::BuffFioraE;
use crate::fiora::passive::AbilityFioraPassive;
use crate::fiora::r::BuffFioraR;

const FIORA_Q_KEY: &str = "Characters/Fiora/Spells/FioraQAbility/FioraQ";
#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.init_resource::<passive::FioraVitalLastDirection>();
        app.add_systems(
            FixedUpdate,
            (
                add_skills,
                passive::update_add_vital,
                passive::update_remove_vital,
                r::fixed_update,
            ),
        );
        app.add_observer(on_fiora_skill_cast);
        app.add_observer(passive::on_passive_damage_create);
        app.add_observer(e::on_event_attack_end);
        app.add_observer(r::on_r_damage_create);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;

fn on_fiora_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_fiora_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_fiora_w(&mut commands, entity),
        SkillSlot::E => cast_fiora_e(&mut commands, entity),
        SkillSlot::R => cast_fiora_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_fiora_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Fiora_Q_Dash_Trail_ground"));
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: FIORA_Q_KEY.into(),
            move_type: DashMoveType::Pointer { max: 300.0 },
            damage: None,
            speed: 1000.0,
        },
    );
    skill_damage(
        commands,
        entity,
        FIORA_Q_KEY,
        DamageShape::Nearest {
            max_distance: 300.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Fiora_Q_Slash_Cas")),
    );
}

fn cast_fiora_w(commands: &mut Commands, entity: Entity) {
    spawn_skill_particle(commands, entity, hash_bin("Fiora_W_Telegraph_Blue"));
    play_skill_animation(commands, entity, hash_bin("Spell2_In"));
    spawn_skill_particle(commands, entity, hash_bin("Fiora_W_Cas"));
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    despawn_skill_particle(commands, entity, hash_bin("Fiora_W_Telegraph_Blue"));
}

fn cast_fiora_e(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert((BuffAttack {
        bonus_attack_speed: 0.5,
    },));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraE::default());
    reset_skill_attack(commands, entity);
}

fn cast_fiora_r(commands: &mut Commands, entity: Entity) {
    spawn_skill_particle(commands, entity, hash_bin("Fiora_R_Indicator_Ring"));
    spawn_skill_particle(commands, entity, hash_bin("Fiora_R_ALL_Warning"));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraR::default());
}

fn add_skills(
    mut commands: Commands,
    q_fiora: Query<Entity, (With<Fiora>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_fiora.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Fiora/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Fiora/Spells/FioraPassiveAbility/FioraPassive",
            ),
            CoolDown::default(),
            AbilityFioraPassive,
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            commands.entity(entity).with_related::<SkillOf>((
                Skill::new(skill_slot_from_index(index), skill),
                CoolDown::default(),
            ));
        }
    }
}
