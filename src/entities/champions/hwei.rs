use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_utils::hash_bin;

use crate::{
    core::{ActionAnimationPlay, ActionParticleSpawn, CoolDown, Skill, SkillOf, Skills},
    entities::champion::Champion,
    PassiveSkillOf,
};

#[derive(Default)]
pub struct PluginHwei;

impl Plugin for PluginHwei {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Hwei"))]
#[reflect(Component)]
pub struct Hwei;

fn add_skills(mut commands: Commands, q_hwei: Query<Entity, (With<Hwei>, Without<Skills>)>) {
    for entity in q_hwei.iter() {
        commands
            .entity(entity)
            .with_related::<PassiveSkillOf>((
                Skill {
                    level: 0,
                    key: hash_bin("Characters/Hwei/Spells/HweiPassiveAbility/HweiPassive"),
                    effect: None,
                    mana_cost: 0.0,
                },
                CoolDown {
                    duration: 0.0,
                    ..default()
                },
            ))
            .with_related::<SkillOf>((
                Skill {
                    level: 0,
                    key: hash_bin("Characters/Hwei/Spells/HweiQAbility/HweiQ"),
                    mana_cost: 60.0,
                    effect: Some(behave! {
                        Behave::Sequence => {
                            Behave::trigger(
                                ActionAnimationPlay { hash: hash_bin("Spell1") }
                            ),
                            Behave::trigger(
                                ActionParticleSpawn { hash: hash_bin("Hwei_Q_Q_Tar") },
                            ),
                        }
                    }),
                },
                CoolDown {
                    duration: 8.0,
                    ..default()
                },
            ))
            .with_related::<SkillOf>((
                Skill {
                    level: 0,
                    key: hash_bin("Characters/Hwei/Spells/HweiWAbility/HweiW"),
                    mana_cost: 70.0,
                    effect: Some(behave! {
                        Behave::Sequence => {
                            Behave::trigger(
                                ActionAnimationPlay { hash: hash_bin("Spell1") }
                            ),
                            Behave::trigger(
                                ActionParticleSpawn { hash: hash_bin("Hwei_Q_W_AoE") },
                            ),
                        }
                    }),
                },
                CoolDown {
                    duration: 8.0,
                    ..default()
                },
            ))
            .with_related::<SkillOf>((
                Skill {
                    level: 0,
                    key: hash_bin("Characters/Hwei/Spells/HweiEAbility/HweiE"),
                    mana_cost: 50.0,
                    effect: Some(behave! {
                        Behave::Sequence => {
                            Behave::trigger(
                                ActionAnimationPlay { hash: hash_bin("Spell1") }
                            ),
                            Behave::trigger(
                                ActionParticleSpawn { hash: hash_bin("Hwei_Q_Q_Tar") },
                            ),
                        }
                    }),
                },
                CoolDown {
                    duration: 8.0,
                    ..default()
                },
            ))
            .with_related::<SkillOf>((
                Skill {
                    level: 0,
                    key: hash_bin("Characters/Hwei/Spells/HweiRAbility/HweiR"),
                    mana_cost: 100.0,
                    effect: Some(behave! {
                        Behave::Sequence => {
                            Behave::trigger(
                                ActionAnimationPlay { hash: hash_bin("Spell1") }
                            ),
                            Behave::trigger(
                                ActionParticleSpawn { hash: hash_bin("Hwei_Q_Q_Tar") },
                            ),
                        }
                    }),
                },
                CoolDown {
                    duration: 80.0,
                    ..default()
                },
            ));
    }
}
