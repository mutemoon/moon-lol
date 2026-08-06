use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::{PluginRiven, Riven};
use lol_core::action::{Action, CommandAction};
use lol_core::base::direction::Direction;
use lol_core::life::Health;
use lol_core::skill::{Skill, Skills};
use lol_core::team::Team;
use rand::Rng;

use crate::fiora_vs_riven::{FioraVsRivenAction, FioraVsRivenObs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalPhase {
    RandomBaseline,
    UntrainedPPO,
}

pub struct PreTrainingEvalResults {
    pub random_baseline: (f32, f32, f32),
    pub untrained_ppo: (f32, f32, f32),
}

#[derive(Resource)]
pub struct GuiEvalResource {
    pub ppo_policy: Arc<Mutex<dyn FnMut(&FioraVsRivenObs) -> FioraVsRivenAction + Send + 'static>>,
    pub phase: EvalPhase,
    pub episodes_target: usize,
    pub max_steps: usize,
    pub current_episode: usize,
    pub step_count: usize,
    pub frame_counter: usize,
    pub frame_skip: usize,
    pub total_rewards: f32,
    pub total_kills: f32,
    pub total_steps: usize,
    pub current_ep_reward: f32,
    pub current_ep_steps: usize,
    pub fiora: Entity,
    pub riven: Entity,
    pub initial_fiora_pos: Vec3,
    pub initial_riven_pos: Vec3,
    pub random_results: Option<(f32, f32, f32)>,
    pub result_sender: Sender<PreTrainingEvalResults>,
    pub assets_loaded: bool,
    pub fiora_skin_handle: Handle<DynamicWorld>,
    pub riven_skin_handle: Handle<DynamicWorld>,
}

#[derive(SystemParam)]
pub struct GuiEvalQueries<'w, 's> {
    pub q_transform: Query<'w, 's, &'static mut Transform>,
    pub q_health: Query<'w, 's, &'static mut Health>,
    pub q_vital: Query<'w, 's, &'static Vital>,
    pub q_skills: Query<'w, 's, &'static Skills>,
    pub q_skill: Query<'w, 's, &'static mut Skill>,
    pub q_cooldown: Query<'w, 's, &'static mut lol_core::skill::CoolDown>,
    pub q_recast: Query<'w, 's, &'static lol_core::skill::SkillRecastWindow>,
}

pub fn run_pre_training_gui_eval<F>(
    num_episodes: usize,
    max_steps: usize,
    ppo_policy: F,
) -> PreTrainingEvalResults
where
    F: FnMut(&FioraVsRivenObs) -> FioraVsRivenAction + Send + 'static,
{
    let mut app = App::new();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .map(|p| p.parent())
        .flatten()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&manifest_dir));

    let asset_plugin = bevy::asset::AssetPlugin {
        file_path: workspace_root.join("assets").to_string_lossy().to_string(),
        ..Default::default()
    };

    app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
        primary_window: Some(Window {
            title: "Fiora vs Riven Evaluation (Phase 1: Random Baseline)".to_string(),
            resolution: (1280, 720).into(),
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.add_plugins(lol_render::PluginRender);
    app.add_plugins(lol_core::PluginCore);
    app.add_plugins(lol_particle::PluginParticle);
    app.add_plugins(PluginFiora);
    app.add_plugins(PluginRiven);

    app.insert_resource(lol_base::map::MapPaths::new("test"));

    app.finish();
    app.cleanup();

    let asset_server = app.world().resource::<AssetServer>();
    let fiora_config_handle = asset_server.load::<DynamicWorld>("characters/fiora/config.ron");
    let riven_config_handle = asset_server.load::<DynamicWorld>("characters/Riven/config.ron");
    let fiora_skin_handle = asset_server.load::<DynamicWorld>("characters/fiora/skins/skin0.ron");
    let riven_skin_handle = asset_server.load::<DynamicWorld>("characters/Riven/skins/skin0.ron");

    let initial_fiora_pos = Vec3::ZERO;
    let initial_riven_pos = Vec3::new(250.0, 0.0, 0.0);

    let fiora = app
        .world_mut()
        .spawn((
            Fiora::default(),
            Transform::from_translation(initial_fiora_pos),
            Team::Order,
            ConfigCharacterRecord {
                character_record: fiora_config_handle.clone(),
            },
            Health::new(500.0),
            lol_core::damage::Armor(35.0),
            lol_core::movement::Movement { speed: 345.0 },
            lol_render::controller::SelfPlayer,
            lol_base_render::camera::Focus,
            ConfigSkin {
                skin: fiora_skin_handle.clone(),
            },
        ))
        .id();

    let riven = app
        .world_mut()
        .spawn((
            Riven::default(),
            Transform::from_translation(initial_riven_pos),
            Team::Chaos,
            ConfigCharacterRecord {
                character_record: riven_config_handle.clone(),
            },
            Health::new(500.0),
            lol_core::damage::Armor(33.0),
            lol_core::movement::Movement { speed: 340.0 },
            ConfigSkin {
                skin: riven_skin_handle.clone(),
            },
        ))
        .id();

    let (tx, rx) = std::sync::mpsc::channel();

    let eval_resource = GuiEvalResource {
        ppo_policy: Arc::new(Mutex::new(ppo_policy)),
        phase: EvalPhase::RandomBaseline,
        episodes_target: num_episodes,
        max_steps,
        current_episode: 0,
        step_count: 0,
        frame_counter: 0,
        frame_skip: 3,
        total_rewards: 0.0,
        total_kills: 0.0,
        total_steps: 0,
        current_ep_reward: 0.0,
        current_ep_steps: 0,
        fiora,
        riven,
        initial_fiora_pos,
        initial_riven_pos,
        random_results: None,
        result_sender: tx,
        assets_loaded: false,
        fiora_skin_handle,
        riven_skin_handle,
    };

    app.insert_resource(eval_resource);
    app.add_systems(Update, gui_eval_step_system);

    app.run();

    rx.recv().unwrap_or(PreTrainingEvalResults {
        random_baseline: (0.0, 0.0, 0.0),
        untrained_ppo: (0.0, 0.0, 0.0),
    })
}

fn build_obs(fiora: Entity, riven: Entity, queries: &GuiEvalQueries) -> FioraVsRivenObs {
    let fpos = queries
        .q_transform
        .get(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();
    let rpos = queries
        .q_transform
        .get(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let dist = fpos.distance(rpos);

    let fhp = queries.q_health.get(fiora);
    let rhp = queries.q_health.get(riven);

    let vital = queries.q_vital.get(riven).ok();
    let (has_vital, vital_is_active, v_x, v_neg_x, v_z, v_neg_z) = match vital {
        Some(v) => {
            let (vx, vnx, vz, vnz) = match v.direction {
                Direction::X => (1.0, 0.0, 0.0, 0.0),
                Direction::NegX => (0.0, 1.0, 0.0, 0.0),
                Direction::Z => (0.0, 0.0, 1.0, 0.0),
                Direction::NegZ => (0.0, 0.0, 0.0, 1.0),
            };
            (true, v.is_active(), vx, vnx, vz, vnz)
        }
        None => (false, false, 0.0, 0.0, 0.0, 0.0),
    };

    let (q_ready, w_ready, e_ready, r_ready) = {
        let mut ready = (true, true, true, true);
        if let Ok(skills) = queries.q_skills.get(fiora) {
            let skill_entities = skills.to_vec();
            let check_ready = |idx: usize| -> bool {
                if idx < skill_entities.len() {
                    let s_entity = skill_entities[idx];
                    let cd = queries.q_cooldown.get(s_entity).ok();
                    let recast = queries.q_recast.get(s_entity).ok();
                    match cd {
                        Some(c) => lol_core::skill::is_skill_ready(c, recast),
                        None => true,
                    }
                } else {
                    true
                }
            };
            ready = (
                check_ready(0),
                check_ready(1),
                check_ready(2),
                check_ready(3),
            );
        }
        ready
    };

    FioraVsRivenObs {
        fiora_pos: fpos,
        fiora_hp: fhp.map(|h| h.value).unwrap_or(0.0),
        fiora_max_hp: fhp.map(|h| h.max).unwrap_or(500.0),
        riven_pos: rpos,
        riven_hp: rhp.map(|h| h.value).unwrap_or(0.0),
        riven_max_hp: rhp.map(|h| h.max).unwrap_or(500.0),
        distance: dist,
        q_ready,
        w_ready,
        e_ready,
        r_ready,
        has_vital,
        vital_is_active,
        vital_dir_x: v_x,
        vital_dir_neg_x: v_neg_x,
        vital_dir_z: v_z,
        vital_dir_neg_z: v_neg_z,
    }
}

fn gui_eval_step_system(
    eval_res: ResMut<GuiEvalResource>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut exit_writer: MessageWriter<AppExit>,
    mut q_window: Query<&mut Window>,
    mut queries: GuiEvalQueries,
) {
    let eval = eval_res.into_inner();

    if !eval.assets_loaded {
        let fiora_ready = asset_server
            .get_recursive_dependency_load_state(&eval.fiora_skin_handle)
            .is_some_and(|s| s.is_loaded());
        let riven_ready = asset_server
            .get_recursive_dependency_load_state(&eval.riven_skin_handle)
            .is_some_and(|s| s.is_loaded());

        if fiora_ready && riven_ready {
            eval.assets_loaded = true;
            // Setup skill levels (Q3 W1 E1 R1)
            let fiora = eval.fiora;
            let riven = eval.riven;
            if let Ok(skills) = queries.q_skills.get(fiora) {
                let skill_entities = skills.to_vec();
                if skill_entities.len() >= 4 {
                    if let Ok(mut q) = queries.q_skill.get_mut(skill_entities[0]) {
                        q.level = 3;
                    }
                    if let Ok(mut w) = queries.q_skill.get_mut(skill_entities[1]) {
                        w.level = 1;
                    }
                    if let Ok(mut e) = queries.q_skill.get_mut(skill_entities[2]) {
                        e.level = 1;
                    }
                    if let Ok(mut r) = queries.q_skill.get_mut(skill_entities[3]) {
                        r.level = 1;
                    }
                }
            }
            if let Ok(skills) = queries.q_skills.get(riven) {
                let skill_entities = skills.to_vec();
                if skill_entities.len() >= 4 {
                    if let Ok(mut q) = queries.q_skill.get_mut(skill_entities[0]) {
                        q.level = 3;
                    }
                    if let Ok(mut w) = queries.q_skill.get_mut(skill_entities[1]) {
                        w.level = 1;
                    }
                    if let Ok(mut e) = queries.q_skill.get_mut(skill_entities[2]) {
                        e.level = 1;
                    }
                    if let Ok(mut r) = queries.q_skill.get_mut(skill_entities[3]) {
                        r.level = 1;
                    }
                }
            }
        }
        return;
    }

    eval.frame_counter += 1;
    if eval.frame_counter % eval.frame_skip != 0 {
        return;
    }

    eval.current_ep_steps += 1;
    eval.step_count += 1;

    let fiora = eval.fiora;
    let riven = eval.riven;

    // Read previous state for reward calculation
    let prev_riven_hp = queries
        .q_health
        .get(riven)
        .map(|h| h.value)
        .unwrap_or(500.0);
    let prev_fiora_hp = queries
        .q_health
        .get(fiora)
        .map(|h| h.value)
        .unwrap_or(500.0);
    let prev_fpos = queries
        .q_transform
        .get(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();
    let riven_pos = queries
        .q_transform
        .get(riven)
        .map(|t| t.translation)
        .unwrap_or(eval.initial_riven_pos);
    let riven_pos_2d = Vec2::new(riven_pos.x, riven_pos.z);

    // Build observation
    let obs = build_obs(fiora, riven, &queries);

    // Get action from policy depending on phase
    let action = match eval.phase {
        EvalPhase::RandomBaseline => {
            let act_idx = rand::rng().random_range(0..9);
            FioraVsRivenAction::from_index(act_idx)
        }
        EvalPhase::UntrainedPPO => (eval.ppo_policy.lock().unwrap())(&obs),
    };

    let is_repeat_move = match action {
        FioraVsRivenAction::MoveEast50 => {
            (prev_fpos.x - (riven_pos.x + 50.0)).abs() < 5.0
                && (prev_fpos.z - riven_pos.z).abs() < 5.0
        }
        FioraVsRivenAction::MoveWest50 => {
            (prev_fpos.x - (riven_pos.x - 50.0)).abs() < 5.0
                && (prev_fpos.z - riven_pos.z).abs() < 5.0
        }
        FioraVsRivenAction::MoveNorth50 => {
            (prev_fpos.x - riven_pos.x).abs() < 5.0
                && (prev_fpos.z - (riven_pos.z + 50.0)).abs() < 5.0
        }
        FioraVsRivenAction::MoveSouth50 => {
            (prev_fpos.x - riven_pos.x).abs() < 5.0
                && (prev_fpos.z - (riven_pos.z - 50.0)).abs() < 5.0
        }
        _ => false,
    };
    let repeat_move_penalty = if is_repeat_move { -5.0 } else { 0.0 };

    // Apply action
    match action {
        FioraVsRivenAction::MoveEast50 => {
            if let Ok(mut t) = queries.q_transform.get_mut(fiora) {
                t.translation = Vec3::new(riven_pos.x + 50.0, riven_pos.y, riven_pos.z);
            }
        }
        FioraVsRivenAction::MoveWest50 => {
            if let Ok(mut t) = queries.q_transform.get_mut(fiora) {
                t.translation = Vec3::new(riven_pos.x - 50.0, riven_pos.y, riven_pos.z);
            }
        }
        FioraVsRivenAction::MoveNorth50 => {
            if let Ok(mut t) = queries.q_transform.get_mut(fiora) {
                t.translation = Vec3::new(riven_pos.x, riven_pos.y, riven_pos.z + 50.0);
            }
        }
        FioraVsRivenAction::MoveSouth50 => {
            if let Ok(mut t) = queries.q_transform.get_mut(fiora) {
                t.translation = Vec3::new(riven_pos.x, riven_pos.y, riven_pos.z - 50.0);
            }
        }
        FioraVsRivenAction::AttackRiven => {
            commands.trigger(CommandAction {
                entity: fiora,
                action: Action::Attack(riven),
            });
        }
        FioraVsRivenAction::CastQ => {
            commands.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 0,
                    point: riven_pos_2d,
                },
            });
        }
        FioraVsRivenAction::CastW => {
            commands.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 1,
                    point: riven_pos_2d,
                },
            });
        }
        FioraVsRivenAction::CastE => {
            commands.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 2,
                    point: riven_pos_2d,
                },
            });
        }
        FioraVsRivenAction::CastR => {
            commands.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 3,
                    point: riven_pos_2d,
                },
            });
        }
    }

    // Read current state after action
    let curr_riven_hp = queries.q_health.get(riven).map(|h| h.value).unwrap_or(0.0);
    let curr_fiora_hp = queries.q_health.get(fiora).map(|h| h.value).unwrap_or(0.0);

    let damage_dealt = (prev_riven_hp - curr_riven_hp).max(0.0);
    let damage_taken = (prev_fiora_hp - curr_fiora_hp).max(0.0);

    let prev_dist = prev_fpos.distance(riven_pos);
    let is_movement_action = matches!(
        action,
        FioraVsRivenAction::MoveEast50
            | FioraVsRivenAction::MoveWest50
            | FioraVsRivenAction::MoveNorth50
            | FioraVsRivenAction::MoveSouth50
    );
    let close_move_penalty = if prev_dist < 80.0 && is_movement_action {
        -3.0
    } else {
        0.0
    };

    let is_skill_action = matches!(
        action,
        FioraVsRivenAction::CastQ
            | FioraVsRivenAction::CastW
            | FioraVsRivenAction::CastE
            | FioraVsRivenAction::CastR
    );
    let skill_bonus = if is_skill_action { 50.0 } else { 0.0 };

    let vital_break_bonus = if damage_dealt > 25.0 { 300.0 } else { 0.0 };

    let steps_left: f32 = 100.0 - eval.current_ep_steps as f32;
    let speed_multiplier: f32 = 1.0 + steps_left.max(0.0) / 100.0 * 2.0;
    let mut step_reward: f32 = damage_dealt * 4.0 * speed_multiplier
        + vital_break_bonus
        + skill_bonus
        + repeat_move_penalty
        + close_move_penalty
        - damage_taken * 0.1
        - 0.2;

    let terminated = curr_riven_hp <= 0.0 || curr_fiora_hp <= 0.0;
    let truncated = eval.current_ep_steps >= eval.max_steps;

    if curr_riven_hp <= 0.0 {
        let speed_kill_bonus: f32 = steps_left.max(0.0) * 10.0;
        step_reward += 500.0 + speed_kill_bonus;
    } else if curr_fiora_hp <= 0.0 {
        step_reward -= 100.0;
    }

    eval.current_ep_reward += step_reward;

    if terminated || truncated {
        if curr_riven_hp <= 0.0 {
            eval.total_kills += 1.0;
        }
        eval.total_rewards += eval.current_ep_reward;
        eval.total_steps += eval.current_ep_steps;
        eval.current_episode += 1;

        if eval.current_episode >= eval.episodes_target {
            let avg_reward = eval.total_rewards / eval.episodes_target as f32;
            let kill_rate = eval.total_kills / eval.episodes_target as f32;
            let avg_steps = eval.total_steps as f32 / eval.episodes_target as f32;
            let res = (avg_reward, kill_rate, avg_steps);

            match eval.phase {
                EvalPhase::RandomBaseline => {
                    println!(
                        "\n>>> Phase 1 (Random Action Baseline) Complete! Switching GUI Window to Phase 2 (Untrained PPO Policy)..."
                    );
                    eval.random_results = Some(res);
                    eval.phase = EvalPhase::UntrainedPPO;
                    eval.current_episode = 0;
                    eval.total_rewards = 0.0;
                    eval.total_kills = 0.0;
                    eval.total_steps = 0;
                    eval.current_ep_reward = 0.0;
                    eval.current_ep_steps = 0;
                    if let Ok(mut window) = q_window.single_mut() {
                        window.title =
                            "Fiora vs Riven Evaluation (Phase 2: Untrained PPO Model)".to_string();
                    }
                }
                EvalPhase::UntrainedPPO => {
                    let rand_res = eval.random_results.unwrap_or((0.0, 0.0, 0.0));
                    let results = PreTrainingEvalResults {
                        random_baseline: rand_res,
                        untrained_ppo: res,
                    };
                    let _ = eval.result_sender.send(results);
                    exit_writer.write(AppExit::Success);
                    return;
                }
            }
        }

        // Reset episode
        eval.current_ep_reward = 0.0;
        eval.current_ep_steps = 0;

        if let Ok(mut t) = queries.q_transform.get_mut(fiora) {
            t.translation = eval.initial_fiora_pos;
        }
        if let Ok(mut t) = queries.q_transform.get_mut(riven) {
            t.translation = eval.initial_riven_pos;
        }
        if let Ok(mut h) = queries.q_health.get_mut(fiora) {
            h.value = if h.max > 0.0 { h.max } else { 500.0 };
        }
        if let Ok(mut h) = queries.q_health.get_mut(riven) {
            h.value = if h.max > 0.0 { h.max } else { 500.0 };
        }

        if let Ok(skills) = queries.q_skills.get(fiora) {
            for s_entity in skills.to_vec() {
                if let Ok(mut cd) = queries.q_cooldown.get_mut(s_entity) {
                    cd.timer = None;
                }
            }
        }

        commands.entity(riven).remove::<Vital>();
    }
}
