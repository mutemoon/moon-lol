use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_slot_from_index,
    spawn_skill_particle, CoolDown, DamageShape, EventSkillCast, Skill, SkillOf, SkillSlot,
    Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::PassiveSkillOf;
use crate::DamageType;

const DARIUS_Q_KEY: &str = "Characters/Darius/Spells/DariusAxeGrabCone/DariusAxeGrabCone";
const DARIUS_E_KEY: &str = "Characters/Darius/Spells/DariusAoeGrab/DariusAoeGrab";
const DARIUS_R_KEY: &str = "Characters/Darius/Spells/DariusExecute/DariusExecute";

#[derive(Default)]
pub struct PluginDarius;

impl Plugin for PluginDarius {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_darius_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Darius"))]
#[reflect(Component)]
pub struct Darius;

fn on_darius_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_darius_q(&mut commands, entity),
        SkillSlot::W => cast_darius_w(&mut commands, entity),
        SkillSlot::E => cast_darius_e(&mut commands, entity),
        SkillSlot::R => cast_darius_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_darius_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Darius_Q_Cast"));
    // Q is a cleave with inner and outer damage (using circle as approximation)
    skill_damage(
        commands,
        entity,
        DARIUS_Q_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Darius_Q_Hit")),
    );
    // FUTURE: Inner/outer damage differentiation
}

fn cast_darius_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Darius_W_Cast"));
    // W is an empowered auto attack that applies slow
    reset_skill_attack(commands, entity);
    debug!("{:?} 的技能 {} 应对目标施加 {}",
        entity, "Darius W", "出血叠层 + DebuffSlow");
}

fn cast_darius_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Darius_E_Cast"));
    // E is a cone pull
    skill_damage(
        commands,
        entity,
        DARIUS_E_KEY,
        DamageShape::Sector { radius: 300.0, angle: 90.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Darius_E_Hit")),
    );
    // FUTURE: Add pull effect
}

fn cast_darius_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Darius_R_Cast"));
    // R is a targeted execute ability
    skill_damage(
        commands,
        entity,
        DARIUS_R_KEY,
        DamageShape::Nearest { max_distance: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Darius_R_Hit")),
    );
    // FUTURE: Execute mechanic and noxian Might stacking
}

fn add_skills(
    mut commands: Commands,
    q_darius: Query<Entity, (With<Darius>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_darius.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Darius/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Darius/Spells/DariusPassiveAbility/DariusPassive",
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
