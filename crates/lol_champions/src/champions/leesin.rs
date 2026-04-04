use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::{DebuffSlow, DebuffStun};
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::movement::{CommandMovement, MovementAction, MovementWay};
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

use crate::buffs::leesin_buffs::BuffLeeSinIronWill;

const LEESIN_Q_KEY: &str = "Characters/LeeSin/Spells/LeeSinQ/LeeSinQ";
const LEESIN_W_KEY: &str = "Characters/LeeSin/Spells/LeeSinW/LeeSinW";
const LEESIN_E_KEY: &str = "Characters/LeeSin/Spells/LeeSinE/LeeSinE";
const LEESIN_R_KEY: &str = "Characters/LeeSin/Spells/LeeSinR/LeeSinR";
const LEESIN_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginLeeSin;

impl Plugin for PluginLeeSin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_leesin_skill_cast);
        app.add_observer(on_leesin_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("LeeSin"))]
#[reflect(Component)]
pub struct LeeSin;

/// 标记李青当前释放的技能，用于伤害命中 observer 判断是哪个技能命中
/// stage: 1=E1, 2=E2, 3=R
#[derive(Component, Debug, Clone)]
pub struct LeeSinActiveAbility {
    pub stage: u8,
}

fn on_leesin_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leesin: Query<(), With<LeeSin>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_leesin.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_leesin_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::W => cast_leesin_w(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::E => cast_leesin_e(
            &mut commands,
            entity,
            trigger.skill_entity,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_leesin_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_leesin_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell1"));

    if stage == 1 {
        // First cast: Sonic Wave - skillshot that marks enemy
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_Q_Cast"));
        skill_damage(
            commands,
            entity,
            LEESIN_Q_KEY,
            DamageShape::Sector {
                radius: 400.0,
                angle: 30.0,
            },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("LeeSin_Q_Hit")),
        );
        // Insert recast window for second cast (Resonating Strike)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Resonating Strike - dash to marked enemy
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_Q2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &ActionDash {
                skill: LEESIN_Q_KEY.into(),
                move_type: DashMoveType::Pointer { max: 500.0 },
                damage: Some(DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 800.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Lee Sin Q", stage
        );
    }
}

fn cast_leesin_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell2"));

    if stage == 1 {
        // First cast: Safeguard - dash to ally/windwall
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_W_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &ActionDash {
                skill: LEESIN_W_KEY.into(),
                move_type: DashMoveType::Pointer { max: 300.0 },
                damage: None,
                speed: 700.0,
            },
        );
        // Insert recast window for second cast (Iron Will)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Iron Will - lifesteal and attack speed buff
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_W2_Cast"));
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffLeeSinIronWill::new(0.1, 0.1, 4.0));
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Lee Sin W", stage
        );
    }
}

fn cast_leesin_e(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: Tempest - AoE damage (no slow)
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_E_Cast"));
        skill_damage(
            commands,
            entity,
            LEESIN_E_KEY,
            DamageShape::Circle { radius: 250.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("LeeSin_E_Hit")),
        );
        // Insert recast window for second cast (Cripple)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Cripple - slow enemies already affected by Tempest
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_E2_Cast"));
        // Mark E2 so observer applies slow on damage hit
        commands
            .entity(entity)
            .insert(LeeSinActiveAbility { stage: 2 });
        skill_damage(
            commands,
            entity,
            LEESIN_E_KEY,
            DamageShape::Circle { radius: 250.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("LeeSin_E2_Hit")),
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Lee Sin E", stage
        );
    }
}

fn cast_leesin_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("LeeSin_R_Cast"));
    // Mark R so observer applies knockback + stun on damage hit
    commands
        .entity(entity)
        .insert(LeeSinActiveAbility { stage: 3 });

    skill_damage(
        commands,
        entity,
        LEESIN_R_KEY,
        DamageShape::Nearest {
            max_distance: 150.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("LeeSin_R_Hit")),
    );
}

/// 监听李青造成的伤害，应用E2减速和R击退眩晕
fn on_leesin_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_leesin: Query<(Entity, &LeeSinActiveAbility, &GlobalTransform)>,
    q_target: Query<&GlobalTransform>,
) {
    let target_entity = trigger.event_target();
    let Ok((leesin_entity, active_ability, leesin_transform)) = q_leesin.get(trigger.source) else {
        return;
    };

    match active_ability.stage {
        2 => {
            // E2命中给目标减速
            commands
                .entity(target_entity)
                .with_related::<BuffOf>(DebuffSlow::new(0.6, 2.0));
        }
        3 => {
            // R命中：眩晕 + 击退
            commands
                .entity(target_entity)
                .with_related::<BuffOf>(DebuffStun::new(1.0));

            // 计算击退方向（推向远离李青的方向）
            if let Ok(target_transform) = q_target.get(target_entity) {
                let knockback_dir = target_transform.translation() - leesin_transform.translation();
                if knockback_dir.length() > 0.1 {
                    let knockback_pos =
                        target_transform.translation() + knockback_dir.normalize() * 200.0;
                    commands.trigger(CommandMovement {
                        entity: target_entity,
                        priority: 100,
                        action: MovementAction::Start {
                            way: MovementWay::Pathfind(Vec3::new(
                                knockback_pos.x,
                                target_transform.translation().y,
                                knockback_pos.z,
                            )),
                            speed: Some(1200.0),
                            source: "LeeSinR".to_string(),
                        },
                    });
                }
            }

            // R用完后移除标记
            commands
                .entity(leesin_entity)
                .remove::<LeeSinActiveAbility>();
        }
        _ => {}
    }
}

fn add_skills(
    mut commands: Commands,
    q_leesin: Query<Entity, (With<LeeSin>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_leesin.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/LeeSin/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/LeeSin/Spells/LeeSinPassiveAbility/LeeSinPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // Q, W, E all use manual cooldown mode for recast windows
            if index < 3 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
