use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::olaf_buffs::{BuffOlafR, BuffOlafW};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::DamageType;
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const OLAF_E_KEY: &str = "Characters/Olaf/Spells/OlafRecklessStrike/OlafRecklessStrike";

// Olaf W parameters
const OLAF_W_ATTACK_SPEED_BONUS: f32 = 0.4; // 40% attack speed
const OLAF_W_SHIELD: f32 = 80.0; // shield amount
const OLAF_W_DURATION: f32 = 5.0; // 5 seconds

// Olaf R parameters
const OLAF_R_DURATION: f32 = 6.0; // 6 seconds CC immunity

#[derive(Default)]
pub struct PluginOlaf;

impl Plugin for PluginOlaf {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_olaf_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Olaf"))]
#[reflect(Component)]
pub struct Olaf;

fn on_olaf_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_olaf: Query<(), With<Olaf>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_olaf.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_olaf_q(&mut commands, entity, trigger.point),
        SkillSlot::W => cast_olaf_w(&mut commands, entity),
        SkillSlot::E => cast_olaf_e(&mut commands, entity, trigger.point),
        SkillSlot::R => cast_olaf_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_olaf_q(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Olaf_Q_Cast"));
    // Q is a linear axe throw - could add dash damage or just particle
}

fn cast_olaf_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Olaf_W_Cast"));

    // W provides attack speed buff and shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOlafW::new(
            OLAF_W_ATTACK_SPEED_BONUS,
            OLAF_W_SHIELD,
            OLAF_W_DURATION,
        ));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 攻速加成和 {} 护盾",
        entity,
        "Olaf W",
        (OLAF_W_ATTACK_SPEED_BONUS * 100.0) as i32,
        OLAF_W_SHIELD as i32
    );
}

fn cast_olaf_e(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Olaf_E_Cast"));
    skill_damage(
        commands,
        entity,
        OLAF_E_KEY,
        DamageShape::Nearest {
            max_distance: 200.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::True,
        }],
        Some(hash_bin("Olaf_E_Hit")),
    );
    // E also deals self-damage
}

fn cast_olaf_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Olaf_R_Cast"));

    // R provides CC immunity
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOlafR::new(OLAF_R_DURATION));

    debug!(
        "{:?} 释放了 {} 技能，免疫控制效果持续 {} 秒",
        entity, "Olaf R", OLAF_R_DURATION
    );
}

fn add_skills(
    mut commands: Commands,
    q_olaf: Query<Entity, (With<Olaf>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_olaf.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Olaf/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Olaf/Spells/OlafPassiveAbility/OlafPassive",
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
