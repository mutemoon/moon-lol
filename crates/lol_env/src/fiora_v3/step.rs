use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_core::base::stats::ChampionStats;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::team::Team;
use rand::Rng;
use rand::seq::SliceRandom;

use super::action::{FIORA_V3_OFFSET_SCALE, FioraV3Action, FioraV3DiscreteAction};
use super::obs::{FioraV3Obs, get_ego_obs_from_world};
use crate::curriculum::CurriculumRewardConfig;
use crate::fiora_riven_common::unpause_virtual_time;
use crate::traits::{RewardBreakdownItem, StepResult};

pub fn setup_fiora_v3_health_world(world: &mut World, fiora: Entity) {
    if let Some(mut hp) = world.get_mut::<Health>(fiora) {
        hp.value = hp.max;
    }
    if let Some(mut stats) = world.get_mut::<ChampionStats>(fiora) {
        stats.kills = 0;
        stats.deaths = 0;
        stats.assists = 0;
        stats.minion_kills = 0;
    }
}

/// 对 Fiora V3 环境中的小兵血量进行随机化分配：
/// 保证每波小兵中既有一击必杀（20~55 HP）的残血小兵，也有中等血量和近乎满血的小兵，
/// 促使智能体学会识别血量并精准选取残血目标进行补刀。
pub fn randomize_fiora_v3_minion_health(world: &mut World) {
    let mut rng = rand::rng();

    let mut order_minions = Vec::new();
    let mut chaos_minions = Vec::new();

    {
        let mut q = world.query_filtered::<(Entity, &Team), With<Minion>>();
        for (entity, team) in q.iter(world) {
            match team {
                Team::Order => order_minions.push(entity),
                Team::Chaos => chaos_minions.push(entity),
                _ => {}
            }
        }
    }

    for mut minion_list in [order_minions, chaos_minions] {
        if minion_list.is_empty() {
            continue;
        }
        // 随机打乱顺序，使得一击必杀和残血小兵的位置在每局不同
        minion_list.shuffle(&mut rng);
        let n = minion_list.len();

        for (i, entity) in minion_list.into_iter().enumerate() {
            if let Some(mut health) = world.get_mut::<Health>(entity) {
                // 分桶分配血量：
                // 前 ~30% (至少1个)：一击必杀残血 (20 ~ 55 HP，Fiora 68 AD 一刀必杀)
                // 紧接着 ~35%：中等血量 (120 ~ 240 HP，需多次攻击)
                // 剩余 ~35%：高血量/满血 (320 ~ max_hp)
                let target_hp: f32 = if i < (n.max(3) / 3).max(1) {
                    rng.random_range(20.0f32..=55.0f32).min(health.max)
                } else if i < (2 * n.max(3) / 3).max(2) {
                    rng.random_range(120.0f32..=240.0f32).min(health.max)
                } else {
                    rng.random_range((health.max * 0.7)..=health.max)
                };

                health.value = target_hp.clamp(1.0, health.max);
            }
        }
    }
}

/// 统一的单人世界初始化与重置逻辑（满血重置与小兵随机血量设置）
pub fn setup_fiora_v3_env_world(champions: &[Entity], world: &mut World) {
    if let Some(&fiora) = champions.first() {
        setup_fiora_v3_health_world(world, fiora);
    }
    randomize_fiora_v3_minion_health(world);
}

pub fn dispatch_single_action(
    world: &mut World,
    self_entity: Entity,
    action: FioraV3Action,
    visible_unit_entities: &[Option<Entity>],
) {
    let spos = world
        .get::<Transform>(self_entity)
        .map(|t| t.translation)
        .unwrap_or_default();

    let self_team = world
        .get::<Team>(self_entity)
        .copied()
        .unwrap_or(Team::Order);

    let chosen_target = visible_unit_entities
        .get(action.target_idx as usize)
        .copied()
        .flatten();

    let chosen_target_pos = chosen_target
        .and_then(|e| world.get::<Transform>(e).map(|t| t.translation))
        .unwrap_or(spos);

    let target_offset_pos = Vec3::new(
        chosen_target_pos.x + action.offset_x.clamp(-1.0, 1.0) * FIORA_V3_OFFSET_SCALE,
        chosen_target_pos.y,
        chosen_target_pos.z + action.offset_z.clamp(-1.0, 1.0) * FIORA_V3_OFFSET_SCALE,
    );

    let is_target_enemy =
        chosen_target.is_some_and(|e| world.get::<Team>(e).is_some_and(|t| *t != self_team));

    // 友方目标或无效目标防御性降级：普攻必须有敌方目标，否则降级为 Move
    let actual_discrete = match action.discrete {
        FioraV3DiscreteAction::Attack if !is_target_enemy => FioraV3DiscreteAction::Move,
        other => other,
    };

    match actual_discrete {
        FioraV3DiscreteAction::NoOp => {}
        FioraV3DiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Move(Vec2::new(target_offset_pos.x, target_offset_pos.z)),
            });
        }
        FioraV3DiscreteAction::Attack => {
            if let Some(target) = chosen_target {
                world.trigger(CommandAction {
                    entity: self_entity,
                    action: Action::Attack(target),
                });
            }
        }
    }
}

pub fn step_fiora_v3_world(
    app: &mut App,
    fiora: Entity,
    act_fiora: FioraV3Action,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraV3Obs> {
    let prev_f_obs = get_ego_obs_from_world(app.world(), fiora);
    let prev_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    // 1. 识别对有效敌方小兵的普通攻击行为
    let fiora_attacked_minion = act_fiora.discrete == FioraV3DiscreteAction::Attack
        && prev_f_obs.is_target_enemy(act_fiora.target_idx as usize);

    dispatch_single_action(
        app.world_mut(),
        fiora,
        act_fiora,
        &prev_f_obs.visible_unit_entities,
    );
    unpause_virtual_time(app.world_mut());

    for _ in 0..10 {
        app.update();
    }

    let curr_f_obs = get_ego_obs_from_world(app.world(), fiora);
    let curr_f_hp = app
        .world()
        .get::<Health>(fiora)
        .map(|h| h.value)
        .unwrap_or(1.0);
    let curr_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    let fiora_cs_diff = curr_f_cs.saturating_sub(prev_f_cs) as f32;

    // 普通攻击但是没产生补刀判定
    let fiora_wasted = if fiora_attacked_minion && fiora_cs_diff == 0.0 {
        1.0
    } else {
        0.0
    };

    let reward_cfg = app
        .world()
        .get_resource::<CurriculumRewardConfig>()
        .cloned()
        .unwrap_or_default();

    let f_vars = HashMap::from([
        ("self_cs".to_string(), fiora_cs_diff),
        ("self_attack_no_cs".to_string(), fiora_wasted),
        ("cs_reward_coef".to_string(), reward_cfg.cs_reward),
        ("penalty_coef".to_string(), reward_cfg.attack_no_cs_penalty),
        ("minion_hp_scale".to_string(), reward_cfg.minion_hp_scale),
    ]);

    let (r_fiora, f_breakdown_items) = super::FIORA_V3_SPEC
        .reward_formula
        .as_ref()
        .expect("FIORA_V3_SPEC 缺少 reward_formula DSL 规范")
        .compute(&f_vars);

    let f_breakdown = f_breakdown_items
        .into_iter()
        .map(|it| RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    let terminated = curr_f_hp <= 0.0;
    let truncated = step_count >= max_steps;

    StepResult {
        obs: curr_f_obs,
        reward: r_fiora,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown: f_breakdown,
        reward_variables: f_vars,
    }
}
