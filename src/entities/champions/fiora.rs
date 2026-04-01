use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    ActionAnimationPlay, ActionBuffSpawn, ActionDamage, ActionDamageEffect, ActionDash,
    ActionParticleDespawn, ActionParticleSpawn, CoolDown, DamageShape, DamageType, Skill, SkillOf,
    Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{
    AbilityFioraPassive, BuffAttack, BuffFioraE, BuffFioraR, DashMoveType, PassiveSkillOf,
    SkillAction, SkillEffect,
};

#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_load_assets);
        app.add_systems(FixedUpdate, add_skills);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;

fn startup_load_assets(mut res_assets_skill_effect: ResMut<Assets<SkillEffect>>) {
    res_assets_skill_effect.add_hash(
        "Characters/Fiora/Spells/FioraQAbility/FioraQ",
        SkillEffect(vec![
            SkillAction::Animation(ActionAnimationPlay {
                hash: hash_bin("Spell1"),
            }),
            SkillAction::Particle(ActionParticleSpawn {
                hash: hash_bin("Fiora_Q_Dash_Trail_ground"),
            }),
            SkillAction::Dash(ActionDash {
                skill: "Characters/Fiora/Spells/FioraQAbility/FioraQ".into(),
                move_type: DashMoveType::Pointer { max: 300. },
                damage: None,
                speed: 1000.0,
            }),
            SkillAction::Damage(ActionDamage {
                entity: Entity::PLACEHOLDER,
                skill: "Characters/Fiora/Spells/FioraQAbility/FioraQ".into(),
                effects: vec![ActionDamageEffect {
                    shape: DamageShape::Nearest {
                        max_distance: 300.0,
                    },
                    damage_list: vec![TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    }],
                    particle: Some(hash_bin("Fiora_Q_Slash_Cas")),
                }],
            }),
        ]),
    );

    res_assets_skill_effect.add_hash(
        "Characters/Fiora/Spells/FioraWAbility/FioraW",
        SkillEffect(vec![
            SkillAction::Particle(ActionParticleSpawn {
                hash: hash_bin("Fiora_W_Telegraph_Blue"),
            }),
            SkillAction::Animation(ActionAnimationPlay {
                hash: hash_bin("Spell2_In"),
            }),
            SkillAction::Particle(ActionParticleSpawn {
                hash: hash_bin("Fiora_W_Cas"),
            }),
            SkillAction::Animation(ActionAnimationPlay {
                hash: hash_bin("Spell2"),
            }),
            SkillAction::ParticleDespawn(ActionParticleDespawn {
                hash: hash_bin("Fiora_W_Telegraph_Blue"),
            }),
        ]),
    );

    res_assets_skill_effect.add_hash(
        "Characters/Fiora/Spells/FioraEAbility/FioraE",
        SkillEffect(vec![
            SkillAction::Buff(ActionBuffSpawn::new((
                BuffAttack {
                    bonus_attack_speed: 0.5,
                },
                BuffFioraE::default(),
            ))),
            SkillAction::AttackReset,
        ]),
    );

    res_assets_skill_effect.add_hash(
        "Characters/Fiora/Spells/FioraRAbility/FioraR",
        SkillEffect(vec![
            SkillAction::Particle(ActionParticleSpawn {
                hash: hash_bin("Fiora_R_Indicator_Ring"),
            }),
            SkillAction::Particle(ActionParticleSpawn {
                hash: hash_bin("Fiora_R_ALL_Warning"),
            }),
            SkillAction::Buff(ActionBuffSpawn::new(BuffFioraR::default())),
        ]),
    );
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
            Skill {
                key_spell_object: "Characters/Fiora/Spells/FioraPassiveAbility/FioraPassive".into(),
                key_skill_effect: "Characters/Fiora/Spells/FioraPassiveAbility/FioraPassive".into(),
                ..default()
            },
            CoolDown::default(),
            AbilityFioraPassive,
        ));

        for &skill in character_record.spells.as_ref().unwrap().iter() {
            commands.entity(entity).with_related::<SkillOf>((
                Skill {
                    key_spell_object: skill.into(),
                    key_skill_effect: skill.into(),
                    ..default()
                },
                CoolDown::default(),
            ));
        }
    }
}
