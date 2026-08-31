use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_core::character::CharacterReady;
use lol_core::life::Health;

use super::action::{FioraV2Action, FioraV2DiscreteAction, OFFSET_SCALE};
use super::obs::{FioraV2Obs, get_v2_obs_from_world};
use super::reward::{FioraV2RewardContext, FioraV2RewardModel, is_v2_aligned_with_vital};
use crate::fiora_riven_common::{FioraRivenEntities, VitalBreakTracker, unpause_virtual_time};
use crate::flash_plugin::{FLASH_DISTANCE, FlashCooldown, dispatch_flash};
use crate::modifier_obs::ModifierNameId;
use crate::reward::RewardModel;
use crate::traits::StepResult;

/// 靶子瑞雯在 V2 中的生命值上限
pub const RIVEN_V2_HP: f32 = 10000.0;

pub fn setup_v2_riven_health_world(world: &mut World, riven: Entity) {
    if let Some(mut hp) = world.get_mut::<Health>(riven) {
        hp.value = RIVEN_V2_HP;
        hp.max = RIVEN_V2_HP;
    }
}

pub fn on_v2_character_ready_setup_riven_health(
    trigger: On<Add, CharacterReady>,
    entities: Res<FioraRivenEntities>,
    mut q_health: Query<&mut Health>,
) {
    if trigger.entity == entities.riven {
        if let Ok(mut hp) = q_health.get_mut(entities.riven) {
            hp.value = RIVEN_V2_HP;
            hp.max = RIVEN_V2_HP;
        }
    }
}

/// 统一的有头/无头世界初始化与重置逻辑（重设瑞雯 10000 血量并重置剑姬闪现）
pub fn setup_v2_fiora_riven_world(champions: &[Entity], world: &mut World) {
    if champions.len() >= 2 {
        setup_v2_riven_health_world(world, champions[1]);
        if let Some(mut flash) = world.get_mut::<FlashCooldown>(champions[0]) {
            flash.reset();
        } else {
            world.entity_mut(champions[0]).insert(FlashCooldown::default());
        }
    }
}

pub fn dispatch_action_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    action: FioraV2Action,
) {
    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();

    let target_pos = Vec3::new(
        rpos.x + action.offset_x.clamp(-1.0, 1.0) * OFFSET_SCALE,
        rpos.y,
        rpos.z + action.offset_z.clamp(-1.0, 1.0) * OFFSET_SCALE,
    );

    match action.discrete {
        FioraV2DiscreteAction::NoOp => {}
        FioraV2DiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Move(Vec2::new(target_pos.x, target_pos.z)),
            });
        }
        FioraV2DiscreteAction::Attack => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Attack(riven),
            });
        }
        FioraV2DiscreteAction::CastQ => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 0,
                    point: Vec2::new(target_pos.x, target_pos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastE => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 2,
                    point: Vec2::new(fpos.x, fpos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastR => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 3,
                    point: Vec2::new(rpos.x, rpos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastFlash => {
            let offset_dir = Vec3::new(action.offset_x, 0.0, action.offset_z);
            let dir = if offset_dir.length_squared() > 1e-4 {
                offset_dir.normalize()
            } else {
                let to_riven = rpos - fpos;
                if to_riven.length_squared() > 1e-4 {
                    to_riven.normalize()
                } else {
                    Vec3::X
                }
            };
            dispatch_flash(world, fiora, dir, FLASH_DISTANCE);
        }
    }

    dispatch_v2_riven_action(world, fiora, riven);
}

pub fn dispatch_v2_riven_action(world: &mut World, fiora: Entity, riven: Entity) {
    let rhp = world.get::<Health>(riven).map(|h| h.value).unwrap_or(0.0);
    if rhp <= 0.0 {
        return;
    }

    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();

    let roll = rand::random::<f32>();
    let target = if roll < 0.5 {
        let away = (rpos - fpos).normalize_or_zero();
        let dir = if away.length_squared() > 1e-4 {
            away
        } else {
            Vec3::X
        };
        rpos + dir * 300.0
    } else {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        rpos + Vec3::new(angle.cos(), 0.0, angle.sin()) * 300.0
    };

    world.trigger(CommandAction {
        entity: riven,
        action: Action::Move(Vec2::new(target.x, target.z)),
    });
}

pub fn step_v2_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraV2Action,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraV2Obs> {
    let prev_obs = get_v2_obs_from_world(app.world(), fiora, riven);
    let prev_riven_hp = prev_obs.riven_hp;
    let prev_fpos = prev_obs.fiora_pos;

    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    dispatch_action_world(app.world_mut(), fiora, riven, action);
    unpause_virtual_time(app.world_mut());

    for _ in 0..10 {
        app.update();
    }

    let obs = get_v2_obs_from_world(app.world(), fiora, riven);
    let curr_riven_hp = obs.riven_hp;
    let curr_fpos = obs.fiora_pos;

    let tracker_hit = app.world().resource::<VitalBreakTracker>().hit;
    let had_active_vital = prev_obs
        .target_modifiers
        .iter()
        .any(|m| m.name_id == ModifierNameId::FioraPassiveVital && m.stack_count > 0.5);
    let is_vital_break = tracker_hit && had_active_vital;

    let has_vital = prev_obs
        .target_modifiers
        .iter()
        .any(|m| m.name_id == ModifierNameId::FioraPassiveVital);
    let prev_aligned =
        has_vital && is_v2_aligned_with_vital(prev_fpos, prev_obs.riven_pos, &prev_obs);
    let curr_aligned =
        has_vital && is_v2_aligned_with_vital(curr_fpos, prev_obs.riven_pos, &prev_obs);

    let ctx = FioraV2RewardContext {
        prev_aligned,
        curr_aligned,
        is_vital_break,
        prev_riven_hp,
        curr_riven_hp,
        riven_max_hp: prev_obs.riven_max_hp,
        elapsed_secs: step_count as f32 * (10.0 / 60.0),
    };

    let model = FioraV2RewardModel;
    let (reward, items, vars) = model.evaluate(&ctx);

    let reward_breakdown = items
        .into_iter()
        .map(|it| crate::traits::RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    let terminated = curr_riven_hp <= 0.0 || obs.fiora_hp <= 0.0;
    let truncated = step_count >= max_steps;

    StepResult {
        obs,
        reward,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown,
        reward_variables: vars,
    }
}
