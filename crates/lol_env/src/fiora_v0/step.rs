use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};

use super::action::FioraVsRivenAction;
use crate::fiora_riven_common::{
    AttackEventTracker, FioraVsRivenObs, VitalBreakTracker, compute_step_reward,
    get_obs_from_world, unpause_virtual_time,
};
use crate::traits::StepResult;

pub fn dispatch_action_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
) {
    match action {
        FioraVsRivenAction::MoveEast50
        | FioraVsRivenAction::MoveWest50
        | FioraVsRivenAction::MoveNorth50
        | FioraVsRivenAction::MoveSouth50 => {
            let rpos = world
                .get::<Transform>(riven)
                .map(|t| t.translation)
                .unwrap_or_default();
            let new_pos = match action {
                FioraVsRivenAction::MoveEast50 => Vec3::new(rpos.x + 50.0, rpos.y, rpos.z),
                FioraVsRivenAction::MoveWest50 => Vec3::new(rpos.x - 50.0, rpos.y, rpos.z),
                FioraVsRivenAction::MoveNorth50 => Vec3::new(rpos.x, rpos.y, rpos.z + 50.0),
                FioraVsRivenAction::MoveSouth50 => Vec3::new(rpos.x, rpos.y, rpos.z - 50.0),
                _ => unreachable!(),
            };
            if let Some(mut t) = world.get_mut::<Transform>(fiora) {
                t.translation = new_pos;
            }
        }
        FioraVsRivenAction::AttackRiven => {}
    }
}

pub fn advance_action_simulation(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
) -> Option<FioraVsRivenObs> {
    if action == FioraVsRivenAction::AttackRiven {
        for _ in 0..300 {
            let is_active = app
                .world()
                .get::<lol_champions::fiora::passive::Vital>(riven)
                .map(|v| v.is_active())
                .unwrap_or(false);
            if is_active {
                break;
            }
            app.update();
        }

        let attack_obs = get_obs_from_world(app.world(), fiora, riven);

        app.world_mut().trigger(CommandAction {
            entity: fiora,
            action: Action::Attack(riven),
        });

        if let Some(mut tracker) = app.world_mut().get_resource_mut::<AttackEventTracker>() {
            tracker.attack_hit = false;
            tracker.attack_ready = false;
        }

        for _ in 0..100 {
            app.update();
            let tracker = app.world().resource::<AttackEventTracker>();
            if tracker.attack_hit && tracker.attack_ready {
                break;
            }
        }

        app.world_mut()
            .trigger(lol_core::attack_auto::CommandAttackAutoStop { entity: fiora });

        Some(attack_obs)
    } else {
        app.update();
        None
    }
}

pub fn step_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraVsRivenObs> {
    let prev_obs = get_obs_from_world(app.world(), fiora, riven);
    let prev_fpos = prev_obs.fiora_pos;
    let prev_riven_hp = prev_obs.riven_hp;

    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    dispatch_action_world(app.world_mut(), fiora, riven, action);
    unpause_virtual_time(app.world_mut());

    let attack_obs = advance_action_simulation(app, fiora, riven, action);

    let obs = get_obs_from_world(app.world(), fiora, riven);
    let curr_fpos = obs.fiora_pos;
    let curr_riven_hp = obs.riven_hp;

    let is_attack = action == FioraVsRivenAction::AttackRiven;
    let tracker_hit = app.world().resource::<VitalBreakTracker>().hit;
    let is_vital_break = tracker_hit && prev_obs.has_vital && prev_obs.vital_is_active;

    let reward_obs = attack_obs.as_ref().unwrap_or(&prev_obs);
    let (reward, reward_breakdown, reward_vars) = compute_step_reward(
        prev_riven_hp,
        curr_riven_hp,
        prev_fpos,
        curr_fpos,
        prev_obs.riven_pos,
        is_attack,
        is_vital_break,
        reward_obs,
        step_count as f32 / 60.0,
    );

    let terminated = curr_riven_hp <= 0.0 || obs.fiora_hp <= 0.0;
    let truncated = step_count >= max_steps;

    StepResult {
        obs,
        reward,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown,
        reward_variables: reward_vars,
    }
}
