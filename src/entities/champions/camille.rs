use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_dash, skill_slot_from_index,
    spawn_skill_particle, CoolDown, DamageShape, EventSkillCast, Skill, SkillCooldownMode,
    SkillOf, SkillRecastWindow, SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::PassiveSkillOf;
use crate::DamageType;

const CAMILLE_Q_KEY: &str = "Characters/Camille/Spells/CamilleQ/CamilleQ";
const CAMILLE_W_KEY: &str = "Characters/Camille/Spells/CamilleW/CamilleW";
const CAMILLE_E_KEY: &str = "Characters/Camille/Spells/CamilleE/CamilleE";
const CAMILLE_R_KEY: &str = "Characters/Camille/Spells/CamilleR/CamilleR";
const CAMILLE_Q_RECAST_WINDOW: f32 = 3.0;
const CAMILLE_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginCamille;

impl Plugin for PluginCamille {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_camille_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Camille"))]
#[reflect(Component)]
pub struct Camille;

fn on_camille_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_camille_q(&mut commands, entity, trigger.skill_entity, cooldown, recast),
        SkillSlot::W => cast_camille_w(&mut commands, entity),
        SkillSlot::E => cast_camille_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_camille_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_camille_q(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell1"));

    if stage == 1 {
        // First cast: Prepares the hookshot
        spawn_skill_particle(commands, entity, hash_bin("Camille_Q_Cast"));
        // Q1 doesn't deal damage, just marks for second cast
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_Q_RECAST_WINDOW));
    } else {
        // Second cast: Deals bonus damage and resets attack
        spawn_skill_particle(commands, entity, hash_bin("Camille_Q2_Cast"));
        reset_skill_attack(commands, entity);
        skill_damage(
            commands,
            entity,
            CAMILLE_Q_KEY,
            DamageShape::Nearest { max_distance: 150.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("Camille_Q2_Hit")),
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Camille Q", stage);
    }
}

fn cast_camille_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Camille_W_Cast"));
    // W is a swept cone that slows
    skill_damage(
        commands,
        entity,
        CAMILLE_W_KEY,
        DamageShape::Sector { radius: 300.0, angle: 90.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Camille_W_Hit")),
    );
    debug!("{:?} W 扫击命中，施加减速", entity);
}

fn cast_camille_e(
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
        // First cast: Hookshot - launches toward terrain
        spawn_skill_particle(commands, entity, hash_bin("Camille_E_Cast"));
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_E_RECAST_WINDOW));
        debug!("{:?} E1 钩索射出，搜索地形附着点", entity);
    } else {
        // Second cast: Dash toward hooked terrain
        spawn_skill_particle(commands, entity, hash_bin("Camille_E2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &crate::ActionDash {
                skill: CAMILLE_E_KEY.into(),
                move_type: crate::DashMoveType::Pointer { max: 400.0 },
                damage: Some(crate::DashDamage {
                    radius_end: 150.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 900.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Camille E", stage);
    }
}

fn cast_camille_r(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Camille_R_Cast"));
    // R is a hookshot-like leap that marks and traps target champion
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: CAMILLE_R_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 350.0 },
            damage: Some(crate::DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::Champion,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 800.0,
        },
    );
    debug!("{:?} R 创建海克斯壁垒，困住目标", entity);
}

fn add_skills(
    mut commands: Commands,
    q_camille: Query<Entity, (With<Camille>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_camille.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Camille/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Camille/Spells/CamillePassiveAbility/CamillePassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // Q and E use manual cooldown mode for recast windows
            if index == 0 || index == 2 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
