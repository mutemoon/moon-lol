use bevy::prelude::*;
use bevy_behave::{behave, Behave};
use league_core::CharacterRecord;
use league_utils::{get_asset_id_by_hash, get_asset_id_by_path, hash_bin};

use crate::{
    core::{ActionAnimationPlay, ActionParticleSpawn, CoolDown, Skill, SkillOf, Skills},
    entities::champion::Champion,
    PassiveSkillOf, SkillEffect,
};

#[derive(Default)]
pub struct PluginHwei;

impl Plugin for PluginHwei {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_load_assets);
        app.add_systems(FixedUpdate, add_skills);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Hwei"))]
#[reflect(Component)]
pub struct Hwei;

fn startup_load_assets(mut res_assets_skill_effect: ResMut<Assets<SkillEffect>>) {
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Hwei/Spells/HweiQAbility/HweiQ"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell1") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Hwei_Q_Q_Tar") },
                ),
            }
        }),
    );
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Hwei/Spells/HweiWAbility/HweiW"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell1") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Hwei_Q_W_AoE") },
                ),
            }
        }),
    );
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Hwei/Spells/HweiEAbility/HweiE"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell1") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Hwei_Q_Q_Tar") },
                ),
            }
        }),
    );
    res_assets_skill_effect.insert(
        get_asset_id_by_path("Characters/Hwei/Spells/HweiRAbility/HweiR"),
        SkillEffect(behave! {
            Behave::Sequence => {
                Behave::trigger(
                    ActionAnimationPlay { hash: hash_bin("Spell1") }
                ),
                Behave::trigger(
                    ActionParticleSpawn { hash: hash_bin("Hwei_Q_Q_Tar") },
                ),
            }
        }),
    );
}

fn add_skills(
    mut commands: Commands,
    q_hwei: Query<Entity, (With<Hwei>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_hwei.iter() {
        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill {
                key_spell_object: get_asset_id_by_path(
                    "Characters/Hwei/Spells/HweiPassiveAbility/HweiPassive",
                ),
                ..default()
            },
            CoolDown::default(),
        ));

        let character_record = res_assets_character_record
            .get(get_asset_id_by_path(
                "Characters/Hwei/CharacterRecords/Root",
            ))
            .unwrap();

        for &skill in character_record.spells.as_ref().unwrap().iter() {
            commands.entity(entity).with_related::<SkillOf>((
                Skill {
                    key_spell_object: get_asset_id_by_hash(skill),
                    key_skill_effect: get_asset_id_by_hash(skill),
                    ..default()
                },
                CoolDown::default(),
            ));
        }
    }
}
