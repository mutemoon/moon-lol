use std::sync::Arc;

use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_core::CharacterRecord;
use league_utils::{get_asset_id_by_hash, get_asset_id_by_path, hash_bin};

use crate::{
    abilities::{AbilityFioraPassive, BuffFioraE, BuffFioraR},
    core::{
        ActionAnimationPlay, ActionAttackReset, ActionBuffSpawn, ActionCommand, ActionDamage,
        ActionDash, ActionParticleDespawn, ActionParticleSpawn, AttackBuff, BuffOf, CoolDown,
        Skill, SkillOf, Skills,
    },
    entities::champion::Champion,
    PassiveSkillOf, SkillEffect,
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
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Fiora/Spells/FioraQAbility/FioraQ"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell1") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Fiora_Q_Dash_Trail_ground") },
                ),
                Behave::trigger(
                    ActionDash::Pointer { speed: 1000., max: 300. },
                ),
                Behave::IfThen => {
                    Behave::trigger(ActionDamage),
                    Behave::Sequence => {
                    },
                },
            }
        }),
    );
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Fiora/Spells/FioraEAbility/FioraW"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Fiora_W_Telegraph_Blue") },
                ),
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell2_In") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Fiora_W_Cas") },
                ),
                Behave::Wait(0.5),
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell2") }
                ),
                Behave::trigger(ActionDamage),
                Behave::Wait(0.1),
                Behave::trigger(
                    ActionParticleDespawn{ hash: hash_bin("Fiora_W_Telegraph_Blue") },
                ),
            }
        }),
    );
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Fiora/Spells/FioraEAbility/FioraE"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(ActionBuffSpawn{
                    bundle: Arc::new(|commands: &mut EntityCommands| {
                        commands.with_related::<BuffOf>((
                            AttackBuff {
                                bonus_attack_speed: 0.5,
                            },
                            BuffFioraE {
                                left: 2
                            },
                        ));
                    }),
                }),
                Behave::trigger(ActionAttackReset),
            }
        }),
    );
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Fiora/Spells/FioraRAbility/FioraR"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Fiora_R_Indicator_Ring") },
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Fiora_R_ALL_Warning") },
                ),
                Behave::trigger(ActionCommand {
                    bundle: Arc::new(|commands: &mut EntityCommands| {
                        commands.with_related::<BuffOf>((
                            BuffFioraR::default(),
                        ));
                    }),
                }),
            }
        }),
    );
}

fn add_skills(
    mut commands: Commands,
    q_fiora: Query<Entity, (With<Fiora>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_fiora.iter() {
        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill {
                key_spell_object: get_asset_id_by_path(
                    "Characters/Fiora/Spells/FioraPassiveAbility/FioraPassive",
                ),
                ..default()
            },
            CoolDown::default(),
            AbilityFioraPassive,
        ));

        let character_record = res_assets_character_record
            .get(get_asset_id_by_path(
                "Characters/Fiora/CharacterRecords/Root",
            ))
            .unwrap();

        for &skill in character_record.spells.as_ref().unwrap().iter() {
            commands.entity(entity).with_related::<SkillOf>((
                Skill {
                    key_spell_object: get_asset_id_by_hash(skill),
                    ..default()
                },
                CoolDown::default(),
            ));
        }
    }
}
