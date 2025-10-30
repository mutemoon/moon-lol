use std::sync::Arc;

use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_utils::hash_bin;

use crate::{
    abilities::{AbilityFioraPassive, BuffFioraE},
    core::{
        ActionAnimationPlay, ActionAttackReset, ActionBuffSpawn, ActionDamage, ActionDash,
        ActionParticleDespawn, ActionParticleSpawn, Attack, AttackBuff, Bounding, BuffOf, Health,
        Movement, Skill, SkillOf,
    },
    entities::champion::Champion,
};

#[derive(Component)]
#[require(Champion)]
pub struct Fiora;

#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, _app: &mut App) {}
}

pub fn spawn_fiora(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert((
            Movement { speed: 325.0 },
            Health {
                value: 600.0,
                max: 600.0,
            },
            Attack::new(150.0, 0.2, 1.45),
            Fiora,
            Bounding {
                radius: 35.0,
                height: 300.0,
            },
        ))
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
                        Behave::trigger(
                            ActionParticleSpawn { hash: hash_bin("Fiora_Q_Slash_Cas") },
                        ),
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
                                    bonus_attack_speed: 10.,
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
                        ActionAnimationPlay { hash: hash_bin("Spell2") }
                    ),
                    Behave::Wait(1.),
                    Behave::trigger(ActionDamage),
                }
            }),
        },))
        .log_components();
}
