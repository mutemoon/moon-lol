use std::sync::Arc;

use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_core::CharacterRecord;
use league_utils::{get_asset_id_by_path, hash_bin};
use lol_config::LeagueProperties;

use crate::core::{
    ActionAnimationPlay, ActionAttackReset, ActionBuffSpawn, ActionCommand, ActionDamage,
    ActionDash, ActionParticleDespawn, ActionParticleSpawn, AttackBuff, BuffOf, CoolDown, Skill,
    SkillOf, Skills,
};
use crate::entities::champion::Champion;
use crate::{PassiveSkillOf, SkillEffect};

#[derive(Default)]
pub struct PluginRiven;

impl Plugin for PluginRiven {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_load_assets);
        app.add_systems(FixedUpdate, add_skills);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Riven"))]
#[reflect(Component)]
pub struct Riven;

fn startup_load_assets(
    mut res_assets_skill_effect: ResMut<Assets<SkillEffect>>,
    mut res_league_properties: ResMut<LeagueProperties>,
) {
    res_league_properties.add(
        &mut res_assets_skill_effect,
        "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave",
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell1A") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Riven_Q_01_Detonate") },
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

    res_league_properties.add(
        &mut res_assets_skill_effect,
        "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr",
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Riven_W_Telegraph_Blue") },
                ),
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell2_In") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Riven_W_Cas") },
                ),
                Behave::Wait(0.5),
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell2") }
                ),
                Behave::trigger(ActionDamage),
                Behave::Wait(0.1),
                Behave::trigger(
                    ActionParticleDespawn{ hash: hash_bin("Riven_W_Telegraph_Blue") },
                ),
            }
        }),
    );

    res_league_properties.add(
        &mut res_assets_skill_effect,
        "Characters/Riven/Spells/RivenFeintAbility/RivenFeint",
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(ActionBuffSpawn{
                    bundle: Arc::new(|commands: &mut EntityCommands| {
                        commands.with_related::<BuffOf>((
                            AttackBuff {
                                bonus_attack_speed: 0.5,
                            },
                        ));
                    }),
                }),
                Behave::trigger(ActionAttackReset),
            }
        }),
    );

    res_league_properties.add(
        &mut res_assets_skill_effect,
        "Characters/Riven/Spells/RivenFengShuiEngineAbility/RivenFengShuiEngine",
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Riven_R_Indicator_Ring") },
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Riven_R_ALL_Warning") },
                ),
                Behave::trigger(ActionCommand {
                    bundle: Arc::new(|commands: &mut EntityCommands| {
                        commands.with_related::<BuffOf>((
                        ));
                    }),
                }),
            }
        }),
    );
}

fn add_skills(
    mut commands: Commands,
    q_riven: Query<Entity, (With<Riven>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
    res_league_properties: Res<LeagueProperties>,
) {
    for entity in q_riven.iter() {
        let Some(character_record) = res_league_properties.get(
            &res_assets_character_record,
            "Characters/Riven/CharacterRecords/Root",
        ) else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill {
                key_spell_object: "Characters/Riven/Spells/RivenPassiveAbility/RivenPassive".into(),
                key_skill_effect: "Characters/Riven/Spells/RivenPassiveAbility/RivenPassive".into(),
                ..default()
            },
            CoolDown::default(),
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
