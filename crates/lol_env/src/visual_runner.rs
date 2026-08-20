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
use lol_rl_protocol::{ObsFeaturePayload, PolicyDisplay, RewardFormulaSpec};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window as WinitWindow, WindowId};

use crate::traits::{RewardBreakdownItem, VisualEnvironment};

/// 可视化窗口逻辑分辨率（与 env 的 bevy Window resolution 一致，用于把物理鼠标坐标归一化为逻辑视口坐标）。
const VIEWPORT_W: f32 = 1280.0;
const VIEWPORT_H: f32 = 720.0;

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
    StepWithAction(usize),
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
    pub reward_formula: Option<RewardFormulaSpec>,
}

type PolicyFn<E> = dyn FnMut(
        &<E as crate::traits::RlEnvironment>::Obs,
    ) -> (<E as crate::traits::RlEnvironment>::Action, PolicyDisplay)
    + Send
    + 'static;

/// 从观测生成「原始向量 + 每维说明」，供可视化 UI 逐维展示真实计算值。
fn obs_vector_with_labels<E: VisualEnvironment>(obs: &E::Obs) -> (Vec<f32>, Vec<String>) {
    (
        E::obs_to_vector(obs),
        E::obs_dim_labels().iter().map(|s| s.to_string()).collect(),
    )
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
    pending_manual_action: Option<E::Action>,
    pending_step_once: bool,
    window_created: bool,
    window_size: Option<Vec2>,
    cursor_pos: Option<Vec2>,
    pending_click: Option<Vec2>,
    next_frame_time: Instant,
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
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.window_size = Some(Vec2::new(size.width as f32, size.height as f32));
            }
            WindowEvent::CursorMoved { position, .. } => {
                // 参考 bevy 默认窗口插件（bevy_winit/state.rs）：物理坐标 → 逻辑视口坐标，
                // 用窗口物理尺寸归一化到 VIEWPORT_W×VIEWPORT_H，绕开 WinitPlugin 关闭导致的 scale_factor 失真。
                let Some(size) = self.window_size else {
                    return;
                };
                if size.x <= 0.0 || size.y <= 0.0 {
                    return;
                }
                let logical = Vec2::new(
                    position.x as f32 / size.x * VIEWPORT_W,
                    position.y as f32 / size.y * VIEWPORT_H,
                );
                self.cursor_pos = Some(logical);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.pending_click = self.cursor_pos;
            }
            _ => {}
        }
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
                    let (obs_vector, obs_labels) = obs_vector_with_labels::<E>(&obs);
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
                    self.current_ep_steps = 0;
                    self.episode_reward = 0.0;
                    self.env.reset_world(self.app.world_mut());
                    let obs = self.env.get_current_obs(self.app.world());
                    let (_, policy_items) = (self.policy_arc.lock().unwrap())(&obs);
                    let (obs_vector, obs_labels) = obs_vector_with_labels::<E>(&obs);
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
                        reward_formula: self.env.reward_formula(),
                    };
                    let _ = self.step_tx.send(initial_output);
                }
                VisualRunnerCmd::StepOnce => {
                    self.pending_step_once = true;
                    self.pending_manual_action = None;
                }
                VisualRunnerCmd::StepWithAction(action_id) => {
                    self.pending_step_once = true;
                    self.pending_manual_action = Some(E::action_from_index(action_id));
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
                self.env.on_assets_loaded(self.app.world_mut());

                // Send initial first frame to front-end
                let obs = self.env.get_current_obs(self.app.world());
                let (_, policy_items) = (self.policy_arc.lock().unwrap())(&obs);
                let (obs_vector, obs_labels) = obs_vector_with_labels::<E>(&obs);
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
                    reward_formula: self.env.reward_formula(),
                };
                let _ = self.step_tx.send(initial_output);
            }

            self.app.update();
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
            return;
        }

        // 5. 鼠标点击 → 手动 step action（仅暂停时生效；点击在窗口插件内消费，不经过 Controller/on_click_map 移动管线）
        if self.paused {
            if let Some(screen_pos) = self.pending_click.take() {
                if let Some(action) = self
                    .env
                    .action_from_screen_click(self.app.world_mut(), screen_pos)
                {
                    self.pending_manual_action = Some(action);
                    self.pending_step_once = true;
                }
            }
        } else {
            self.pending_click = None;
        }

        // 6. Determine if an RL step should execute
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
            for (agent_idx, obs) in obs_all.iter().enumerate() {
                let (policy_action, _) = (self.policy_arc.lock().unwrap())(obs);
                let chosen = if agent_idx == 0 {
                    self.pending_manual_action.take().unwrap_or(policy_action)
                } else {
                    policy_action
                };
                actions.push(chosen);
            }

            let step_results = self.env.step_world(&mut self.app, &actions);
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
            let (obs_vector, obs_labels) = obs_vector_with_labels::<E>(&step_result.obs);

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
                reward_formula: self.env.reward_formula(),
            };
            let _ = self.step_tx.send(output);

            if terminated || truncated {
                if self.auto_pause_on_done {
                    self.paused = true;
                    pause_virtual_time(self.app.world_mut());
                }
                self.current_ep_steps = 0;
                self.env.reset_world(self.app.world_mut());
                self.episode_reward = 0.0;

                // Send newly reset start frame
                let next_obs = self.env.get_current_obs(self.app.world());
                let (_, next_policy_items) = (self.policy_arc.lock().unwrap())(&next_obs);
                let (obs_vector, obs_labels) = obs_vector_with_labels::<E>(&next_obs);
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
                    reward_formula: self.env.reward_formula(),
                };
                let _ = self.step_tx.send(reset_output);
            } else if self.paused {
                pause_virtual_time(self.app.world_mut());
            }

            self.app.update();
        } else {
            // Paused: ensure Time<Virtual> is paused so FixedUpdate tick does 0 simulation
            // progress, but app.update() still renders the frame (持续以 60 FPS 提交 SwapChain Present 供 OBS 后台捕获)
            pause_virtual_time(self.app.world_mut());
            self.app.update();
        }

        // 7. 设置下一次 60 FPS 唤醒
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
            pending_manual_action: None,
            pending_step_once: false,
            window_created: false,
            window_size: None,
            cursor_pos: None,
            pending_click: None,
            next_frame_time: Instant::now(),
        };

        let _ = event_loop.run_app(&mut runner);
        bevy::app::AppExit::Success
    });

    app.run();
}
