use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventSkillCast, Skill, SkillOf, SkillSlot, Skills, TargetDamage,
    TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffMoveSpeed, BuffOf, PassiveSkillOf};
use crate::DamageType;

const KAYN_Q_KEY: &str = "Characters/Kayn/Spells/KaynQ/KaynQ";
const KAYN_W_KEY: &str = "Characters/Kayn/Spells/KaynW/KaynW";

#[derive(Default)]
pub struct PluginKayn;

impl Plugin for PluginKayn {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kayn_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kayn"))]
#[reflect(Component)]
pub struct Kayn;

/// Kayn has two forms: Blue (assassin) and Red (bruiser)
/// The form is determined by which enemy champion is damaged first with R
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub enum KaynForm {
    #[default]
    None,
    Blue,  // Assassin form
    Red,   // Bruiser form
}

fn on_kayn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayn.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kayn_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_kayn_w(&mut commands, entity, trigger.point),
        SkillSlot::E => cast_kayn_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_kayn_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_kayn_q(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kayn_Q_Cast"));
    // Q is a dash that deals damage
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: KAYN_Q_KEY.into(),
            move_type: crate::DashMoveType::Fixed(250.0),
            damage: Some(crate::DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 700.0,
        },
    );
}

fn cast_kayn_w(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kayn_W_Cast"));
    // W is an upward slash that slows
    skill_damage(
        commands,
        entity,
        KAYN_W_KEY,
        DamageShape::Sector { radius: 300.0, angle: 60.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kayn_W_Hit")),
    );
    debug!("{:?} 的技能 {} 应对目标施加 {}",
        entity, "Kayn W", "减速 DebuffSlow");
}

fn cast_kayn_e(commands: &mut Commands, _q_transform: &Query<&Transform>, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kayn_E_Cast"));
    // E is a ghost-like dash that allows passing through terrain
    // Movement speed buff
    commands.entity(entity).with_related::<BuffOf>(BuffMoveSpeed::new(0.4, 1.5));
}

fn cast_kayn_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kayn_R_Cast"));
    // R is an extended dash that reappears after a delay
    // Blue form: Assassin - single target damage
    // Red form: Bruiser - AoE damage around reappearance
    // FUTURE: Add form-dependent damage and reappearance mechanic
}

fn add_skills(
    mut commands: Commands,
    q_kayn: Query<Entity, (With<Kayn>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kayn.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kayn/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kayn/Spells/KaynPassiveAbility/KaynPassive",
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
