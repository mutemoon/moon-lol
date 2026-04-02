use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    DamageShape, EventSkillCast, Skill, SkillOf, SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffGarenQ, BuffGarenQAttack, BuffGarenW, BuffOf, PassiveSkillOf};

const GAREN_E_KEY: &str = "Characters/Garen/Spells/GarenSpin/GarenSpin";
const GAREN_R_KEY: &str = "Characters/Garen/Spells/GarenExecute/GarenExecute";

// Garen Q parameters
const GAREN_Q_MOVE_SPEED_BONUS: f32 = 0.3; // 30% move speed bonus
const GAREN_Q_DURATION: f32 = 1.5; // 1.5 seconds
const GAREN_Q_SILENCE_DURATION: f32 = 1.0; // 1 second silence

// Garen W parameters
const GAREN_W_TENACITY: f32 = 0.3; // 30% tenacity
const GAREN_W_DAMAGE_REDUCTION: f32 = 0.3; // 30% damage reduction
const GAREN_W_SHIELD: f32 = 100.0; // shield amount
const GAREN_W_DURATION: f32 = 2.0; // 2 seconds duration

#[derive(Default)]
pub struct PluginGaren;

impl Plugin for PluginGaren {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_garen_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Garen"))]
#[reflect(Component)]
pub struct Garen;

fn on_garen_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_garen_q(&mut commands, entity),
        SkillSlot::W => cast_garen_w(&mut commands, entity),
        SkillSlot::E => cast_garen_e(&mut commands, entity),
        SkillSlot::R => cast_garen_r(&mut commands, entity, trigger.point),
        _ => {}
    }
}

fn cast_garen_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Garen_Q_Cast"));

    // Q provides movement speed buff and enhanced next attack
    commands.entity(entity).with_related::<BuffOf>(BuffGarenQ::new(
        GAREN_Q_MOVE_SPEED_BONUS,
        GAREN_Q_DURATION,
    ));

    // Add the enhanced attack buff (silence on hit)
    commands.entity(entity).with_related::<BuffOf>(BuffGarenQAttack::new(
        GAREN_Q_SILENCE_DURATION,
    ));

    debug!("{:?} 释放了 {} 技能，获得 {}% 移速加成和沉默效果", entity, "Garen Q", (GAREN_Q_MOVE_SPEED_BONUS * 100.0) as i32);
}

fn cast_garen_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Garen_W_Cast"));

    // W provides tenacity, damage reduction, and a shield
    commands.entity(entity).with_related::<BuffOf>(BuffGarenW::new(
        GAREN_W_TENACITY,
        GAREN_W_DAMAGE_REDUCTION,
        GAREN_W_SHIELD,
        GAREN_W_DURATION,
    ));

    debug!("{:?} 释放了 {} 技能，获得 {}% 韧性、{}% 伤害减免和 {} 护盾", entity, "Garen W",
        (GAREN_W_TENACITY * 100.0) as i32, (GAREN_W_DAMAGE_REDUCTION * 100.0) as i32, GAREN_W_SHIELD as i32);
}

fn cast_garen_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Garen_E_Cast"));
    skill_damage(
        commands,
        entity,
        GAREN_E_KEY,
        DamageShape::Circle { radius: 200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: crate::DamageType::Physical,
        }],
        Some(hash_bin("Garen_E_Hit")),
    );
}

fn cast_garen_r(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Garen_R_Cast"));
    // R is a targeted execute ability - use Nearest shape for single target
    skill_damage(
        commands,
        entity,
        GAREN_R_KEY,
        DamageShape::Nearest { max_distance: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: crate::DamageType::Physical,
        }],
        Some(hash_bin("Garen_R_Hit")),
    );
}

fn add_skills(
    mut commands: Commands,
    q_garen: Query<Entity, (With<Garen>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_garen.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Garen/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Garen/Spells/GarenPassiveAbility/GarenPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            commands.entity(entity).with_related::<SkillOf>((
                Skill::new(skill_slot_from_index(index), skill),
                CoolDown::default(),
            ));
        }
    }
}
