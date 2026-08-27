//! Visual runner: drives a Bevy `App` (constructed in `RenderMode::WindowCustomLoop`) through a
//! custom winit event loop, accepting external commands and emitting step results.
//!
//! This module is generic over `VisualEnvironment` and has **no knowledge** of WebSocket
//! or any specific transport protocol.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RawHandleWrapper, WindowWrapper};
use lol_rl_protocol::{ObsFeaturePayload, ObsValueNode, PolicyDisplay, RewardFormulaSpec};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window as WinitWindow, WindowId};

use crate::traits::{RewardBreakdownItem, VisualEnvironment};

/// 可视化环境渲染目标帧率（锁定 60 FPS 供 OBS 稳定捕获并杜绝 CPU 100% 满载）
const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

// ── Public types (WS-agnostic) ──────────────────────────────────────────────

/// Command sent from an external driver (e.g. WS server) to the visual runner.
#[derive(Debug, Clone)]
pub enum VisualRunnerCmd {
    Reset,
    Pause,
    Resume,
    StepOnce,
    SetAutoPause(bool),
}

/// Output emitted by the visual runner after each RL step or state change.
#[derive(Debug, Clone)]
pub struct VisualStepOutput {
    pub step: usize,
    pub reward: f32,
    /// 本局从开始累积到当前步的总奖励（每次对局重置后归零）。
    pub episode_reward: f32,
    pub terminated: bool,
    pub truncated: bool,
    pub is_paused: bool,
    pub reward_breakdown: Vec<RewardBreakdownItem>,
    pub reward_variables: std::collections::HashMap<String, f32>,
    pub policy: PolicyDisplay,
    pub obs_payload: Option<ObsFeaturePayload>,
    /// 策略真实输入的观测向量（与 `obs_labels` 一一对应）。
    pub obs_vector: Vec<f32>,
    /// 观测向量每一维的简要说明。
    pub obs_labels: Vec<String>,
    /// 结构化 AST 观测树。
    pub obs_tree: Option<Vec<ObsValueNode>>,
    pub reward_formula: Option<RewardFormulaSpec>,
}

type PolicyFn<E> = dyn FnMut(
        &<E as crate::traits::RlEnvironment>::Obs,
    ) -> (<E as crate::traits::RlEnvironment>::Action, PolicyDisplay)
    + Send
    + 'static;

/// 从观测生成「原始向量 + 每维说明 + AST 观测树」，供可视化 UI 逐维展示真实计算值与树状渲染。
fn obs_vector_with_labels<E: VisualEnvironment>(
    obs: &E::Obs,
) -> (Vec<f32>, Vec<String>, Option<Vec<ObsValueNode>>) {
    let vec = E::obs_to_vector(obs);
    if let Some(schema) = E::obs_schema() {
        let labels = schema.to_dim_labels();
        let tree = schema.decode_tree(&vec);
        (vec, labels, Some(tree))
    } else {
        (vec, vec![], None)
    }
}

pub fn pause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.pause();
    }
}

pub fn unpause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.unpause();
    }
}

// ── Custom winit runner ─────────────────────────────────────────────────────

#[derive(Debug)]
struct VisualRunnerMetrics {
    last_log: Instant,
    frame_count: u32,
    step_count: u32,
    total_update_duration: Duration,
    max_update_duration: Duration,
    total_step_duration: Duration,
    max_step_duration: Duration,
}

impl Default for VisualRunnerMetrics {
    fn default() -> Self {
        Self {
            last_log: Instant::now(),
            frame_count: 0,
            step_count: 0,
            total_update_duration: Duration::ZERO,
            max_update_duration: Duration::ZERO,
            total_step_duration: Duration::ZERO,
            max_step_duration: Duration::ZERO,
        }
    }
}

struct CustomVisualRunner<E: VisualEnvironment> {
    app: App,
    env: E,
    policy_arc: Arc<Mutex<PolicyFn<E>>>,
    cmd_rx: Receiver<VisualRunnerCmd>,
    step_tx: Sender<VisualStepOutput>,
    paused: bool,
    auto_pause_on_done: bool,
    step_count: usize,
    current_ep_steps: usize,
    episode_reward: f32,
    assets_loaded: bool,
    load_wait_frames: usize,
    pending_step_once: bool,
    window_created: bool,
    next_frame_time: Instant,
    metrics: VisualRunnerMetrics,
}

impl<E: VisualEnvironment> ApplicationHandler for CustomVisualRunner<E> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.window_created {
            let window_attributes = WinitWindow::default_attributes()
                .with_title(self.env.window_title())
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
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        // 1. 节拍控制：未到目标 60 FPS 时间点时通知操作系统休眠，彻底消除 CPU 100% 空转
        if now < self.next_frame_time {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
            return;
        }

        // 2. 规划下一个 16.666ms 周期，防止时间累积漂移
        self.next_frame_time = (self.next_frame_time + FRAME_DURATION).max(now + FRAME_DURATION);

        // 3. Process commands from external driver
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            match cmd {
                VisualRunnerCmd::Pause => {
                    self.paused = true;
                    self.pending_step_once = false;
                    pause_virtual_time(self.app.world_mut());

                    let obs = self.env.get_current_obs(self.app.world());
                    let (_, policy_items) = (self.policy_arc.lock().unwrap())(&obs);
                    let (obs_vector, obs_labels, obs_tree) = obs_vector_with_labels::<E>(&obs);
                    let pause_output = VisualStepOutput {
                        step: self.step_count,
                        reward: 0.0,
                        episode_reward: self.episode_reward,
                        terminated: false,
                        truncated: false,
                        is_paused: true,
                        reward_breakdown: Vec::new(),
                        reward_variables: std::collections::HashMap::new(),
                        policy: policy_items,
                        obs_payload: E::obs_to_payload(&obs),
                        obs_vector,
                        obs_labels,
                        obs_tree,
                        reward_formula: self.env.reward_formula(),
                    };
                    let _ = self.step_tx.send(pause_output);
                }
                VisualRunnerCmd::Resume => {
                    self.paused = false;
                    self.pending_step_once = false;
                    unpause_virtual_time(self.app.world_mut());
                }
                VisualRunnerCmd::Reset => {
                    self.paused = false;
                    self.pending_step_once = false;
                    self.current_ep_steps = 0;
                    self.episode_reward = 0.0;
                    self.env.reset_world(&mut self.app);
                    unpause_virtual_time(self.app.world_mut());
                    let obs = self.env.get_current_obs(self.app.world());
                    let (_, policy_items) = (self.policy_arc.lock().unwrap())(&obs);
                    let (obs_vector, obs_labels, obs_tree) = obs_vector_with_labels::<E>(&obs);
                    let initial_output = VisualStepOutput {
                        step: 0,
                        reward: 0.0,
                        episode_reward: 0.0,
                        terminated: false,
                        truncated: false,
                        is_paused: self.paused,
                        reward_breakdown: Vec::new(),
                        reward_variables: std::collections::HashMap::new(),
                        policy: policy_items,
                        obs_payload: E::obs_to_payload(&obs),
                        obs_vector,
                        obs_labels,
                        obs_tree,
                        reward_formula: self.env.reward_formula(),
                    };
                    let _ = self.step_tx.send(initial_output);
                }
                VisualRunnerCmd::StepOnce => {
                    self.pending_step_once = true;
                }
                VisualRunnerCmd::SetAutoPause(auto) => {
                    self.auto_pause_on_done = auto;
                }
            }
        }

        // 4. Asset loading wait (with fallback timeout)
        if !self.assets_loaded {
            self.load_wait_frames += 1;
            if self.env.is_assets_loaded(self.app.world()) || self.load_wait_frames >= 60 {
                self.assets_loaded = true;
                self.env.on_assets_loaded(&mut self.app);

                // Send initial first frame to front-end
                let obs = self.env.get_current_obs(self.app.world());
                let (_, policy_items) = (self.policy_arc.lock().unwrap())(&obs);
                let (obs_vector, obs_labels, obs_tree) = obs_vector_with_labels::<E>(&obs);
                let initial_output = VisualStepOutput {
                    step: 0,
                    reward: 0.0,
                    episode_reward: 0.0,
                    terminated: false,
                    truncated: false,
                    is_paused: self.paused,
                    reward_breakdown: Vec::new(),
                    reward_variables: std::collections::HashMap::new(),
                    policy: policy_items,
                    obs_payload: E::obs_to_payload(&obs),
                    obs_vector,
                    obs_labels,
                    obs_tree,
                    reward_formula: self.env.reward_formula(),
                };
                let _ = self.step_tx.send(initial_output);
            }

            self.app.update();
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
            return;
        }

        // 5. Determine if an RL step should execute
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

            let obs_all = self.env.get_current_obs_all(self.app.world());
            let mut actions = Vec::with_capacity(obs_all.len());
            for obs in &obs_all {
                let (policy_action, _) = (self.policy_arc.lock().unwrap())(obs);
                actions.push(policy_action);
            }

            let step_start = Instant::now();
            let step_results = self.env.step_world(&mut self.app, &actions);
            let step_duration = step_start.elapsed();

            self.metrics.step_count += 1;
            self.metrics.total_step_duration += step_duration;
            self.metrics.max_step_duration = self.metrics.max_step_duration.max(step_duration);

            let step_result = step_results
                .into_iter()
                .next()
                .expect("empty step result from step_world");

            // 累计本局主视角奖励
            self.episode_reward += step_result.reward;

            let terminated = step_result.terminated;
            let truncated = step_result.truncated;

            // 展示「下一步」预测：基于 step 后观测重新计算，而非本步执行前的策略。
            let (_, next_policy) = (self.policy_arc.lock().unwrap())(&step_result.obs);
            let (obs_vector, obs_labels, obs_tree) = obs_vector_with_labels::<E>(&step_result.obs);

            let will_pause = (terminated || truncated) && self.auto_pause_on_done;
            let output = VisualStepOutput {
                step: step_result.step,
                reward: step_result.reward,
                episode_reward: self.episode_reward,
                terminated: step_result.terminated,
                truncated: step_result.truncated,
                is_paused: self.paused || will_pause,
                reward_breakdown: step_result.reward_breakdown,
                reward_variables: step_result.reward_variables,
                policy: next_policy,
                obs_payload: E::obs_to_payload(&step_result.obs),
                obs_vector,
                obs_labels,
                obs_tree,
                reward_formula: self.env.reward_formula(),
            };
            let _ = self.step_tx.send(output);

            if terminated || truncated {
                let will_pause = self.auto_pause_on_done;
                self.current_ep_steps = 0;
                self.env.reset_world(&mut self.app);
                self.episode_reward = 0.0;
                if will_pause {
                    self.paused = true;
                    pause_virtual_time(self.app.world_mut());
                }

                // Send newly reset start frame
                let next_obs = self.env.get_current_obs(self.app.world());
                let (_, next_policy_items) = (self.policy_arc.lock().unwrap())(&next_obs);
                let (obs_vector, obs_labels, obs_tree) = obs_vector_with_labels::<E>(&next_obs);
                let reset_output = VisualStepOutput {
                    step: self.step_count,
                    reward: 0.0,
                    episode_reward: 0.0,
                    terminated: false,
                    truncated: false,
                    is_paused: self.paused,
                    reward_breakdown: Vec::new(),
                    reward_variables: std::collections::HashMap::new(),
                    policy: next_policy_items,
                    obs_payload: E::obs_to_payload(&next_obs),
                    obs_vector,
                    obs_labels,
                    obs_tree,
                    reward_formula: self.env.reward_formula(),
                };
                let _ = self.step_tx.send(reset_output);
            } else if self.paused {
                pause_virtual_time(self.app.world_mut());
            }

            let update_start = Instant::now();
            self.app.update();
            let update_duration = update_start.elapsed();

            self.metrics.frame_count += 1;
            self.metrics.total_update_duration += update_duration;
            self.metrics.max_update_duration =
                self.metrics.max_update_duration.max(update_duration);
        } else {
            // Paused: ensure Time<Virtual> is paused so FixedUpdate tick does 0 simulation
            // progress, but app.update() still renders the frame (持续以 60 FPS 提交 SwapChain Present 供 OBS 后台捕获)
            pause_virtual_time(self.app.world_mut());

            let update_start = Instant::now();
            self.app.update();
            let update_duration = update_start.elapsed();

            self.metrics.frame_count += 1;
            self.metrics.total_update_duration += update_duration;
            self.metrics.max_update_duration =
                self.metrics.max_update_duration.max(update_duration);
        }

        // 7. 每秒输出 Visual Runner 性能指标 (FPS / update 耗时 / SPS / step 耗时)
        if self.metrics.last_log.elapsed() >= Duration::from_secs(1) {
            let elapsed_secs = self.metrics.last_log.elapsed().as_secs_f64();
            let frames = self.metrics.frame_count.max(1) as f64;
            let steps = self.metrics.step_count as f64;
            let fps = frames / elapsed_secs;
            let sps = steps / elapsed_secs;

            let avg_update_ms =
                (self.metrics.total_update_duration.as_secs_f64() * 1000.0) / frames;
            let max_update_ms = self.metrics.max_update_duration.as_secs_f64() * 1000.0;

            let avg_step_ms = if self.metrics.step_count > 0 {
                (self.metrics.total_step_duration.as_secs_f64() * 1000.0) / steps
            } else {
                0.0
            };
            let max_step_ms = self.metrics.max_step_duration.as_secs_f64() * 1000.0;

            info!(
                "[VISUAL-RUNNER] FPS: {:.1} (update 均: {:.2}ms, 峰: {:.2}ms) | SPS: {:.1} (step 均: {:.2}ms, 峰: {:.2}ms)",
                fps, avg_update_ms, max_update_ms, sps, avg_step_ms, max_step_ms
            );

            self.metrics.last_log = Instant::now();
            self.metrics.frame_count = 0;
            self.metrics.step_count = 0;
            self.metrics.total_update_duration = Duration::ZERO;
            self.metrics.max_update_duration = Duration::ZERO;
            self.metrics.total_step_duration = Duration::ZERO;
            self.metrics.max_step_duration = Duration::ZERO;
        }

        // 8. 设置下一次 60 FPS 唤醒
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Run a visual Bevy loop with a given Env, policy function, and external cmd/step channels.
///
/// The `env` **must** have been constructed with `RenderMode::WindowCustomLoop`.
pub fn run_visual_env<E, F>(
    mut env: E,
    policy: F,
    cmd_rx: Receiver<VisualRunnerCmd>,
    step_tx: Sender<VisualStepOutput>,
) where
    E: VisualEnvironment,
    F: FnMut(&E::Obs) -> (E::Action, PolicyDisplay) + Send + 'static,
{
    let policy_arc = Arc::new(Mutex::new(policy));
    let mut app = env.take_app();

    app.set_runner(move |app: App| {
        let event_loop = EventLoop::new().unwrap();

        let mut runner = CustomVisualRunner {
            app,
            env,
            policy_arc,
            cmd_rx,
            step_tx,
            paused: false,
            auto_pause_on_done: true,
            step_count: 0,
            current_ep_steps: 0,
            episode_reward: 0.0,
            assets_loaded: false,
            load_wait_frames: 0,
            pending_step_once: false,
            window_created: false,
            next_frame_time: Instant::now(),
            metrics: VisualRunnerMetrics::default(),
        };

        let _ = event_loop.run_app(&mut runner);
        bevy::app::AppExit::Success
    });

    app.run();
}
