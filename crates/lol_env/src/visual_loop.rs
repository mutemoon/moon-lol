use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

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

use crate::fiora_vs_riven::{
    FioraVsRivenAction, FioraVsRivenObs, RewardBreakdownItem, compute_step_reward,
};

/// Command sent from external client to control the visual loop.
#[derive(Debug, Clone)]
pub enum VisualCmd {
    Reset,
    Pause,
    Resume,
    StepOnce,
}

/// Telemetry frame sent from visual loop to external client.
#[derive(Debug, Clone)]
pub struct VisualTelemetry {
    pub step: usize,
    pub obs: FioraVsRivenObs,
    pub reward: f32,
    pub reward_breakdown: Vec<RewardBreakdownItem>,
    pub terminated: bool,
    pub truncated: bool,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
}

// Wrapper to make Sender<VisualTelemetry> usable as a Bevy Resource
#[derive(Resource, Clone)]
pub struct TelemetryTx(pub Sender<VisualTelemetry>);

#[derive(Resource, Clone)]
pub struct CmdRx(pub Arc<Mutex<Receiver<VisualCmd>>>);

#[derive(Resource)]
pub struct VisualLoopResource {
    pub policy: Arc<Mutex<dyn FnMut(&FioraVsRivenObs) -> FioraVsRivenAction + Send + 'static>>,
    pub max_steps: usize,
    pub step_count: usize,
    pub frame_counter: usize,
    pub frame_skip: usize,
    pub current_ep_reward: f32,
    pub current_ep_steps: usize,
    pub paused: bool,
    pub step_once: bool,
    pub fiora: Entity,
    pub riven: Entity,
    pub initial_fiora_pos: Vec3,
    pub initial_riven_pos: Vec3,
    pub assets_loaded: bool,
    pub fiora_skin_handle: Handle<DynamicWorld>,
    pub riven_skin_handle: Handle<DynamicWorld>,
}

#[derive(SystemParam)]
pub struct VisualLoopQueries<'w, 's> {
    pub q_transform: Query<'w, 's, &'static mut Transform>,
    pub q_health: Query<'w, 's, &'static mut Health>,
    pub q_vital: Query<'w, 's, &'static Vital>,
    pub q_skills: Query<'w, 's, &'static Skills>,
    pub q_skill: Query<'w, 's, &'static mut Skill>,
    pub q_cooldown: Query<'w, 's, &'static mut lol_core::skill::CoolDown>,
    pub q_recast: Query<'w, 's, &'static lol_core::skill::SkillRecastWindow>,
}

/// Run a visual Bevy loop with a given policy function.
pub fn run_visual_loop<F>(
    max_steps: usize,
    policy: F,
    cmd_rx: Receiver<VisualCmd>,
    frame_tx: Sender<VisualTelemetry>,
) where
    F: FnMut(&FioraVsRivenObs) -> FioraVsRivenAction + Send + 'static,
{
    let mut app = App::new();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&manifest_dir));

    let asset_plugin = bevy::asset::AssetPlugin {
        file_path: workspace_root.join("assets").to_string_lossy().to_string(),
        ..Default::default()
    };

    app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
        primary_window: Some(Window {
            title: "Fiora vs Riven - RL Visual Viewer".to_string(),
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

    app.insert_resource(CmdRx(Arc::new(Mutex::new(cmd_rx))));
    app.insert_resource(TelemetryTx(frame_tx));

    let visual_resource = VisualLoopResource {
        policy: Arc::new(Mutex::new(policy)),
        max_steps,
        step_count: 0,
        frame_counter: 0,
        frame_skip: 3,
        current_ep_reward: 0.0,
        current_ep_steps: 0,
        paused: false,
        step_once: false,
        fiora,
        riven,
        initial_fiora_pos,
        initial_riven_pos,
        assets_loaded: false,
        fiora_skin_handle,
        riven_skin_handle,
    };

    app.insert_resource(visual_resource);
    app.add_systems(Update, visual_loop_step_system);
    app.run();
}

fn build_obs(fiora: Entity, riven: Entity, queries: &VisualLoopQueries) -> FioraVsRivenObs {
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
                    cd.is_none_or(|c| lol_core::skill::is_skill_ready(c, recast))
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

fn visual_loop_step_system(
    mut visual: ResMut<VisualLoopResource>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cmd_rx: Res<CmdRx>,
    frame_tx: Res<TelemetryTx>,
    mut queries: VisualLoopQueries,
) {
    // Handle commands from the command channel
    loop {
        let rx = cmd_rx.0.lock().unwrap();
        match rx.try_recv() {
            Ok(cmd) => {
                drop(rx);
                match cmd {
                    VisualCmd::Pause => visual.paused = true,
                    VisualCmd::Resume => visual.paused = false,
                    VisualCmd::Reset => {
                        reset_episode(&mut visual, &mut queries, &mut commands);
                    }
                    VisualCmd::StepOnce => visual.step_once = true,
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }

    if !visual.assets_loaded {
        let fiora_ready = asset_server
            .get_recursive_dependency_load_state(&visual.fiora_skin_handle)
            .is_some_and(|s| s.is_loaded());
        let riven_ready = asset_server
            .get_recursive_dependency_load_state(&visual.riven_skin_handle)
            .is_some_and(|s| s.is_loaded());

        if fiora_ready && riven_ready {
            visual.assets_loaded = true;
            setup_skill_levels(visual.fiora, visual.riven, &mut queries);
        }
        return;
    }

    if visual.paused && !visual.step_once {
        return;
    }
    visual.step_once = false;

    visual.frame_counter += 1;
    if visual.frame_counter % visual.frame_skip != 0 {
        return;
    }

    visual.current_ep_steps += 1;
    visual.step_count += 1;

    let fiora = visual.fiora;
    let riven = visual.riven;

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
        .unwrap_or(visual.initial_riven_pos);
    let riven_pos_2d = Vec2::new(riven_pos.x, riven_pos.z);

    let obs = build_obs(fiora, riven, &queries);
    let action = (visual.policy.lock().unwrap())(&obs);

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

    let new_obs = build_obs(fiora, riven, &queries);
    let curr_riven_hp = new_obs.riven_hp;
    let curr_fiora_hp = new_obs.fiora_hp;

    let (step_reward, breakdown) = compute_step_reward(
        prev_riven_hp,
        prev_fiora_hp,
        curr_riven_hp,
        curr_fiora_hp,
        prev_fpos,
        riven_pos,
        action,
        visual.current_ep_steps,
        visual.max_steps,
    );

    visual.current_ep_reward += step_reward;

    let terminated = curr_riven_hp <= 0.0 || curr_fiora_hp <= 0.0;
    let truncated = visual.current_ep_steps >= visual.max_steps;

    let telemetry = VisualTelemetry {
        step: visual.step_count,
        obs: new_obs.clone(),
        reward: step_reward,
        reward_breakdown: breakdown,
        terminated,
        truncated,
        fiora_hp: curr_fiora_hp,
        fiora_max_hp: new_obs.fiora_max_hp,
        riven_hp: curr_riven_hp,
        riven_max_hp: new_obs.riven_max_hp,
    };
    let _ = frame_tx.0.send(telemetry);

    if terminated || truncated {
        reset_episode(&mut visual, &mut queries, &mut commands);
    }
}

fn reset_episode(
    visual: &mut VisualLoopResource,
    queries: &mut VisualLoopQueries,
    commands: &mut Commands,
) {
    visual.current_ep_reward = 0.0;
    visual.current_ep_steps = 0;

    if let Ok(mut t) = queries.q_transform.get_mut(visual.fiora) {
        t.translation = visual.initial_fiora_pos;
    }
    if let Ok(mut t) = queries.q_transform.get_mut(visual.riven) {
        t.translation = visual.initial_riven_pos;
    }
    if let Ok(mut h) = queries.q_health.get_mut(visual.fiora) {
        h.value = if h.max > 0.0 { h.max } else { 500.0 };
    }
    if let Ok(mut h) = queries.q_health.get_mut(visual.riven) {
        h.value = if h.max > 0.0 { h.max } else { 500.0 };
    }

    if let Ok(skills) = queries.q_skills.get(visual.fiora) {
        for s_entity in skills.to_vec() {
            if let Ok(mut cd) = queries.q_cooldown.get_mut(s_entity) {
                cd.timer = None;
            }
        }
    }

    commands.entity(visual.riven).remove::<Vital>();
}

fn setup_skill_levels(fiora: Entity, riven: Entity, queries: &mut VisualLoopQueries) {
    for champion in [fiora, riven] {
        if let Ok(skills) = queries.q_skills.get(champion) {
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
}
