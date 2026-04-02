use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_dash, skill_slot_from_index,
    spawn_skill_particle, AbilityResource, BuffOf, CoolDown, DamageShape, EventSkillCast, Skill,
    SkillCooldownMode, SkillOf, SkillRecastWindow, SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffRenektonR, BuffSelfHeal, PassiveSkillOf};
use crate::DamageType;

const RENECKTON_Q_KEY: &str = "Characters/Renekton/Spells/RenektonQ/RenektonQ";
const RENECKTON_W_KEY: &str = "Characters/Renekton/Spells/RenektonW/RenektonW";
const RENECKTON_E_KEY: &str = "Characters/Renekton/Spells/RenektonE/RenektonE";
const RENECKTON_R_KEY: &str = "Characters/Renekton/Spells/RenektonR/RenektonR";
const RENECKTON_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginRenekton;

impl Plugin for PluginRenekton {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_renekton_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Renekton"))]
#[reflect(Component)]
pub struct Renekton;

fn on_renekton_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_ability_resource: Query<&AbilityResource>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_renekton_q(&mut commands, entity, &q_ability_resource),
        SkillSlot::W => cast_renekton_w(&mut commands, entity),
        SkillSlot::E => cast_renekton_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_renekton_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_renekton_q(
    commands: &mut Commands,
    entity: Entity,
    q_ability_resource: &Query<&AbilityResource>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Renekton_Q_Cast"));
    // Q is a cleave that deals damage in a circle
    let rage = q_ability_resource.get(entity).map(|r| r.value).unwrap_or(0.0);
    if rage >= 50.0 {
        debug!("{:?} Q 怒气强化：消耗50怒气，提升伤害和治疗", entity);
    } else {
        debug!("{:?} Q 普通释放", entity);
    }
    skill_damage(
        commands,
        entity,
        RENECKTON_Q_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Renekton_Q_Hit")),
    );
    // Self-heal based on damage dealt
    commands.entity(entity).with_related::<BuffOf>(BuffSelfHeal::new(40.0));
}

fn cast_renekton_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Renekton_W_Cast"));
    // W is an empowered auto attack that stuns
    reset_skill_attack(commands, entity);
    skill_damage(
        commands,
        entity,
        RENECKTON_W_KEY,
        DamageShape::Nearest { max_distance: 150.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Renekton_W_Hit")),
    );
}

fn cast_renekton_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: Dash forward
        spawn_skill_particle(commands, entity, hash_bin("Renekton_E_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &crate::ActionDash {
                skill: RENECKTON_E_KEY.into(),
                move_type: crate::DashMoveType::Pointer { max: 200.0 },
                damage: Some(crate::DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 700.0,
            },
        );
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, RENECKTON_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash again
        spawn_skill_particle(commands, entity, hash_bin("Renekton_E2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &crate::ActionDash {
                skill: RENECKTON_E_KEY.into(),
                move_type: crate::DashMoveType::Pointer { max: 200.0 },
                damage: Some(crate::DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 700.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Renekton E", stage);
    }
}

fn cast_renekton_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Renekton_R_Cast"));
    // R is a transformation that deals damage around and generates rage
    skill_damage(
        commands,
        entity,
        RENECKTON_R_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Renekton_R_Hit")),
    );
    commands.entity(entity).with_related::<BuffOf>(BuffRenektonR::new(0.0, 5.0, 15.0));
}

fn add_skills(
    mut commands: Commands,
    q_renekton: Query<Entity, (With<Renekton>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_renekton.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Renekton/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Renekton/Spells/RenektonPassiveAbility/RenektonPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // E uses manual cooldown mode for recast window
            if index == 2 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
