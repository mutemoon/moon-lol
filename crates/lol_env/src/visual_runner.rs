//! Visual runner: drives a Bevy `App` (constructed in `RenderMode::Window`) through a
//! custom winit event loop, accepting external commands and emitting step results.
//!
//! This module has **no knowledge** of WebSocket or any specific transport protocol.
//! Callers bridge `VisualRunnerCmd` / `VisualStepOutput` to their own transport layer.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RawHandleWrapper, WindowWrapper};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window as WinitWindow, WindowId};

use crate::fiora_vs_riven::{
    FioraVsRivenAction, FioraVsRivenEnv, FioraVsRivenObs, StepResult, get_obs_from_world,
    pause_virtual_time, reset_episode_world, setup_skill_levels_world, step_world,
    unpause_virtual_time,
};

// ── Public types (WS-agnostic) ──────────────────────────────────────────────

/// Command sent from an external driver (e.g. WS server) to the visual runner.
#[derive(Debug, Clone)]
pub enum VisualRunnerCmd {
    Reset,
    Pause,
    Resume,
    StepOnce,
    StepWithAction(usize),
}

/// Per-action probability output from the policy.
#[derive(Debug, Clone)]
pub struct PolicyOutputItem {
    pub action_id: usize,
    pub action_label: String,
    pub prob: f32,
}

/// Output emitted by the visual runner after each RL step.
#[derive(Debug, Clone)]
pub struct VisualStepOutput {
    pub step_result: StepResult,
    pub policy: Vec<PolicyOutputItem>,
}

type PolicyFn =
    dyn FnMut(&FioraVsRivenObs) -> (FioraVsRivenAction, Vec<PolicyOutputItem>) + Send + 'static;

// ── Custom winit runner ─────────────────────────────────────────────────────

struct CustomVisualRunner {
    app: App,
    policy_arc: Arc<Mutex<PolicyFn>>,
    cmd_rx: Receiver<VisualRunnerCmd>,
    step_tx: Sender<VisualStepOutput>,
    max_steps: usize,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    fiora: Entity,
    riven: Entity,
    fiora_skin_handle: Handle<DynamicWorld>,
    riven_skin_handle: Handle<DynamicWorld>,
    paused: bool,
    step_count: usize,
    current_ep_steps: usize,
    assets_loaded: bool,
    pending_manual_action: Option<usize>,
    pending_step_once: bool,
    window_created: bool,
}

impl ApplicationHandler for CustomVisualRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.window_created {
            let window_attributes = WinitWindow::default_attributes()
                .with_title("Fiora vs Riven - RL Visual Viewer")
                .with_inner_size(LogicalSize::new(1280.0, 720.0));

            let winit_window = event_loop.create_window(window_attributes).unwrap();
            let window_wrapper = WindowWrapper::new(winit_window);
            let raw_handle = RawHandleWrapper::new(&window_wrapper).unwrap();

            let mut window_query = self
                .app
                .world_mut()
                .query_filtered::<Entity, With<PrimaryWindow>>();
            if let Ok(window_entity) = window_query.single(self.app.world()) {
                self.app
                    .world_mut()
                    .entity_mut(window_entity)
                    .insert(raw_handle);
            }
            self.window_created = true;
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
        // 无需处理输入事件
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        // 1. Process commands from external driver
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                VisualRunnerCmd::Pause => {
                    self.paused = true;
                    self.pending_step_once = false;
                    pause_virtual_time(self.app.world_mut());
                }
                VisualRunnerCmd::Resume => {
                    self.paused = false;
                    self.pending_step_once = false;
                    unpause_virtual_time(self.app.world_mut());
                }
                VisualRunnerCmd::Reset => {
                    self.current_ep_steps = 0;
                    reset_episode_world(
                        self.app.world_mut(),
                        self.fiora,
                        self.riven,
                        self.initial_fiora_pos,
                        self.initial_riven_pos,
                    );
                }
                VisualRunnerCmd::StepOnce => {
                    self.pending_step_once = true;
                    self.pending_manual_action = None;
                }
                VisualRunnerCmd::StepWithAction(action_id) => {
                    self.pending_step_once = true;
                    self.pending_manual_action = Some(action_id);
                }
            }
        }

        // 2. Asset loading wait
        if !self.assets_loaded {
            let fiora_ready = {
                let asset_server = self.app.world().resource::<AssetServer>();
                asset_server
                    .get_recursive_dependency_load_state(&self.fiora_skin_handle)
                    .is_some_and(|s| s.is_loaded())
            };
            let riven_ready = {
                let asset_server = self.app.world().resource::<AssetServer>();
                asset_server
                    .get_recursive_dependency_load_state(&self.riven_skin_handle)
                    .is_some_and(|s| s.is_loaded())
            };

            if fiora_ready && riven_ready {
                self.assets_loaded = true;
                setup_skill_levels_world(self.app.world_mut(), self.fiora, self.riven);
            }

            self.app.update();
            return;
        }

        // 3. Determine if an RL step should execute
        let should_step = if self.paused {
            if self.pending_step_once {
                self.pending_step_once = false;
                true
            } else {
                false
            }
        } else {
            true
        };

        if should_step {
            self.step_count += 1;
            self.current_ep_steps += 1;

            let prev_obs = get_obs_from_world(self.app.world(), self.fiora, self.riven);
            let (action, policy_items) = {
                let (policy_action, policy_items) = (self.policy_arc.lock().unwrap())(&prev_obs);
                if let Some(manual_id) = self.pending_manual_action.take() {
                    (FioraVsRivenAction::from_index(manual_id), policy_items)
                } else {
                    (policy_action, policy_items)
                }
            };

            let step_result = step_world(
                &mut self.app,
                self.fiora,
                self.riven,
                action,
                self.step_count,
                self.max_steps,
            );

            if self.paused {
                pause_virtual_time(self.app.world_mut());
            }

            let terminated = step_result.terminated;
            let truncated = step_result.truncated;

            let output = VisualStepOutput {
                step_result,
                policy: policy_items,
            };
            let _ = self.step_tx.send(output);

            if terminated || truncated {
                self.paused = true;
                self.current_ep_steps = 0;
                reset_episode_world(
                    self.app.world_mut(),
                    self.fiora,
                    self.riven,
                    self.initial_fiora_pos,
                    self.initial_riven_pos,
                );
            }
        } else {
            // Paused: ensure Time<Virtual> is paused so FixedUpdate tick does 0 simulation
            // progress, but app.update() still renders the frame
            pause_virtual_time(self.app.world_mut());
            self.app.update();
        }
    }
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Run a visual Bevy loop with a given Env, policy function, and external cmd/step channels.
///
/// The `env` **must** have been constructed with `RenderMode::Window`.
/// The WS protocol or any other transport is **not** the concern of this function —
/// callers bridge `VisualRunnerCmd` / `VisualStepOutput` to their own transport layer.
pub fn run_visual_env<F>(
    mut env: FioraVsRivenEnv,
    policy: F,
    cmd_rx: Receiver<VisualRunnerCmd>,
    step_tx: Sender<VisualStepOutput>,
) where
    F: FnMut(&FioraVsRivenObs) -> (FioraVsRivenAction, Vec<PolicyOutputItem>) + Send + 'static,
{
    // Extract all data we need before set_runner consumes the App
    let fiora = env.fiora();
    let riven = env.riven();
    let initial_fiora_pos = env.initial_fiora_pos();
    let initial_riven_pos = env.initial_riven_pos();
    let max_steps = env.max_steps();

    let asset_server = env.app().world().resource::<AssetServer>();
    let fiora_skin_handle = asset_server.load::<DynamicWorld>("characters/fiora/skins/skin0.ron");
    let riven_skin_handle = asset_server.load::<DynamicWorld>("characters/Riven/skins/skin0.ron");

    let policy_arc = Arc::new(Mutex::new(policy));

    // Register custom runner that disables WinitPlugin and creates window + runs event loop manually
    env.app_mut().set_runner(move |app: App| {
        let event_loop = EventLoop::new().unwrap();
        let mut runner = CustomVisualRunner {
            app,
            policy_arc,
            cmd_rx,
            step_tx,
            max_steps,
            initial_fiora_pos,
            initial_riven_pos,
            fiora,
            riven,
            fiora_skin_handle,
            riven_skin_handle,
            paused: false,
            step_count: 0,
            current_ep_steps: 0,
            assets_loaded: false,
            pending_manual_action: None,
            pending_step_once: false,
            window_created: false,
        };

        let _ = event_loop.run_app(&mut runner);
        bevy::app::AppExit::Success
    });

    env.app_mut().run();
}
