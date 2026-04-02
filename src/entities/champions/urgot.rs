use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventSkillCast, Skill, SkillOf, SkillSlot, Skills, TargetDamage,
    TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffOf, BuffShieldWhite, BuffUrgotW, DamageType, PassiveSkillOf};

const URGOT_E_KEY: &str = "Characters/Urgot/Spells/UrgotE/UrgotE";
const URGOT_R_KEY: &str = "Characters/Urgot/Spells/UrgotR/UrgotR";

// Urgot W parameters
const URGOT_W_ATTACK_INTERVAL: f32 = 0.5; // Attack every 0.5 seconds
const URGOT_W_MOVE_SPEED_REDUCTION: f32 = 0.25; // 25% move speed reduction
const URGOT_W_MAX_RANGE: f32 = 300.0; // Max attack range

#[derive(Default)]
pub struct PluginUrgot;

impl Plugin for PluginUrgot {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_urgot_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Urgot"))]
#[reflect(Component)]
pub struct Urgot;

fn on_urgot_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_urgot.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_urgot_q(&mut commands, entity, trigger.point),
        SkillSlot::W => cast_urgot_w(&mut commands, entity),
        SkillSlot::E => cast_urgot_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_urgot_r(&mut commands, entity, trigger.point),
        _ => {}
    }
}

fn cast_urgot_q(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_Q_Cast"));
    // Q is a skillshot that fires in a line
    // TODO: Could implement as a missile or static damage zone
}

fn cast_urgot_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_W_Cast"));

    // W is a toggle that makes Urgot fire at nearby enemies with reduced move speed
    commands.entity(entity).with_related::<BuffOf>(BuffUrgotW::new(
        URGOT_W_ATTACK_INTERVAL,
        URGOT_W_MOVE_SPEED_REDUCTION,
        URGOT_W_MAX_RANGE,
    ));

    debug!("{:?} 释放了 {} 技能，自动攻击周围敌人，移速降低 {}%", entity, "Urgot W", (URGOT_W_MOVE_SPEED_REDUCTION * 100.0) as i32);
}

fn cast_urgot_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_E_Cast"));
    // E is a dash that provides shield and knocks back enemies
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: URGOT_E_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 300.0 },
            damage: None, // E doesn't deal damage directly but knockback
            speed: 700.0,
        },
    );
    commands.entity(entity).with_related::<BuffOf>(BuffShieldWhite::new(100.0));
    // TODO: Add knockback to enemies
}

fn cast_urgot_r(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_R_Cast"));
    // R is a long-range targeted ability that executes and marks enemy
    skill_damage(
        commands,
        entity,
        URGOT_R_KEY,
        DamageShape::Nearest { max_distance: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Urgot_R_Hit")),
    );
    // TODO: Add execute mark debuff
}

fn add_skills(
    mut commands: Commands,
    q_urgot: Query<Entity, (With<Urgot>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_urgot.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Urgot/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Urgot/Spells/UrgotPassiveAbility/UrgotPassive",
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
