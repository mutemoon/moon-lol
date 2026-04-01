use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    ActionAnimationPlay, ActionBuffSpawn, ActionDamage, ActionDamageEffect, ActionDash,
    ActionParticleSpawn, CoolDown, DamageShape, Skill, SkillOf, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{
    BuffRivenPassive, BuffRivenQ2, BuffRivenQ3, BuffShieldWhite, DamageType, DashDamage,
    DashMoveType, PassiveSkillOf, SkillAction, SkillEffect,
};

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

fn startup_load_assets(mut res_assets_skill_effect: ResMut<Assets<SkillEffect>>) {
    // Q 技能三段
    res_assets_skill_effect.add_hash(
        "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave",
        create_riven_tri_cleave_q1(),
    );
    res_assets_skill_effect.add_hash(
        "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleaveQ2",
        create_riven_tri_cleave_q2(),
    );
    res_assets_skill_effect.add_hash(
        "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleaveQ3",
        create_riven_tri_cleave_q3(),
    );

    res_assets_skill_effect.add_hash(
        "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr",
        create_riven_martyr(),
    );

    res_assets_skill_effect.add_hash(
        "Characters/Riven/Spells/RivenFeintAbility/RivenFeint",
        create_riven_feint(),
    );

    res_assets_skill_effect.add_hash(
        "Characters/Riven/Spells/RivenFengShuiEngineAbility/RivenFengShuiEngine",
        create_riven_feng_shui_engine(),
    );
}

fn create_riven_tri_cleave_q1() -> SkillEffect {
    SkillEffect(vec![
        SkillAction::Animation(ActionAnimationPlay {
            hash: hash_bin("Spell1A"),
        }),
        SkillAction::Dash(ActionDash {
            skill: "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave".into(),
            move_type: DashMoveType::Fixed(250.0),
            damage: Some(DashDamage {
                radius_end: 250.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("FirstSlashDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1000.0,
        }),
        SkillAction::Buff(ActionBuffSpawn::new(BuffRivenQ2)),
        SkillAction::Buff(ActionBuffSpawn::new(BuffRivenPassive)),
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_Q_01_Detonate"),
        }),
    ])
}

fn create_riven_tri_cleave_q2() -> SkillEffect {
    SkillEffect(vec![
        SkillAction::Animation(ActionAnimationPlay {
            hash: hash_bin("Spell1B"),
        }),
        SkillAction::Dash(ActionDash {
            skill: "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave".into(),
            move_type: DashMoveType::Fixed(250.0),
            damage: Some(DashDamage {
                radius_end: 250.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("FirstSlashDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1000.0,
        }),
        SkillAction::Buff(ActionBuffSpawn::new(BuffRivenQ3)),
        SkillAction::Buff(ActionBuffSpawn::new(BuffRivenPassive)),
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_Q_02_Detonate"),
        }),
    ])
}

fn create_riven_tri_cleave_q3() -> SkillEffect {
    SkillEffect(vec![
        SkillAction::Animation(ActionAnimationPlay {
            hash: hash_bin("Spell1C"),
        }),
        SkillAction::Dash(ActionDash {
            skill: "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave".into(),
            move_type: DashMoveType::Fixed(250.0),
            damage: Some(DashDamage {
                radius_end: 250.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("FirstSlashDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1000.0,
        }),
        SkillAction::Buff(ActionBuffSpawn::new(BuffRivenPassive)),
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_Q_03_Detonate"),
        }),
    ])
}

fn create_riven_martyr() -> SkillEffect {
    SkillEffect(vec![
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_W_Cast"),
        }),
        SkillAction::Animation(ActionAnimationPlay {
            hash: hash_bin("Spell2"),
        }),
        SkillAction::Damage(ActionDamage {
            entity: Entity::PLACEHOLDER,
            skill: "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr".into(),
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 300.0 },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                }],
                particle: None,
            }],
        }),
    ])
}

fn create_riven_feint() -> SkillEffect {
    SkillEffect(vec![
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_E_Mis"),
        }),
        SkillAction::Animation(ActionAnimationPlay {
            hash: hash_bin("Spell3"),
        }),
        SkillAction::Buff(ActionBuffSpawn::new(BuffShieldWhite::new(100.0))),
        SkillAction::Dash(ActionDash {
            skill: "Characters/Riven/Spells/RivenFeintAbility/RivenFeint".into(),
            move_type: DashMoveType::Fixed(250.0),
            damage: None,
            speed: 1000.0,
        }),
    ])
}

fn create_riven_feng_shui_engine() -> SkillEffect {
    SkillEffect(vec![
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_R_Indicator_Ring"),
        }),
        SkillAction::Particle(ActionParticleSpawn {
            hash: hash_bin("Riven_R_ALL_Warning"),
        }),
    ])
}

fn add_skills(
    mut commands: Commands,
    q_riven: Query<Entity, (With<Riven>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_riven.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Riven/CharacterRecords/Root")
        else {
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
