use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_utils::hash_bin;

use crate::{
    core::{ActionAnimationPlay, ActionParticleSpawn, Skill, SkillOf, Skills},
    entities::champion::Champion,
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
            .with_related::<SkillOf>(Skill {
                level: 0,
                key: hash_bin("Characters/Hwei/Spells/HweiPassiveAbility/HweiPassive"),
                effect: None,
            })
            .with_related::<SkillOf>((Skill {
                level: 0,
                key: hash_bin("Characters/Hwei/Spells/HweiQAbility/HweiQ"),
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
            },))
            .with_related::<SkillOf>((Skill {
                level: 0,
                key: hash_bin("Characters/Hwei/Spells/HweiWAbility/HweiW"),
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
            },))
            .with_related::<SkillOf>((Skill {
                level: 0,
                key: hash_bin("Characters/Hwei/Spells/HweiEAbility/HweiE"),
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
            },))
            .with_related::<SkillOf>((Skill {
                level: 0,
                key: hash_bin("Characters/Hwei/Spells/HweiRAbility/HweiR"),
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
            },));
    }
}
