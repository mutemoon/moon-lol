use std::sync::Arc;

use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_utils::hash_bin;

use crate::{
    abilities::{AbilityFioraPassive, BuffFioraE, BuffFioraR},
    core::{
        ActionAnimationPlay, ActionAttackReset, ActionBuffSpawn, ActionCommand, ActionDamage,
        ActionDash, ActionParticleDespawn, ActionParticleSpawn, AttackBuff, BuffOf, Skill, SkillOf,
        Skills,
    },
    entities::champion::Champion,
};

#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.register_type::<Fiora>();

        app.add_systems(FixedUpdate, add_skills);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;

fn add_skills(mut commands: Commands, q_fiora: Query<Entity, (With<Fiora>, Without<Skills>)>) {
    for entity in q_fiora.iter() {
        commands
            .entity(entity)
            .with_related::<SkillOf>((Skill { effect: None }, AbilityFioraPassive))
            .with_related::<SkillOf>((Skill {
                effect: Some(behave! {
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
            },))
            .with_related::<SkillOf>((Skill {
                effect: Some(behave! {
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
            },))
            .with_related::<SkillOf>((Skill {
                effect: Some(behave! {
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
            },))
            .with_related::<SkillOf>((Skill {
                effect: Some(behave! {
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
            },));
    }
}
