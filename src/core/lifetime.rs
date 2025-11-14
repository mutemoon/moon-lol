use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct PluginLifetime;

impl Plugin for PluginLifetime {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub enum LifetimeMode {
    #[default]
    Timer,
    TimerAndNoChildren,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Lifetime {
    timer: Option<Timer>,
    mode: LifetimeMode,
}

impl Lifetime {
    pub fn new(duration: f32, mode: LifetimeMode) -> Self {
        let timer = if duration > 0.0 {
            Some(Timer::from_seconds(duration, TimerMode::Once))
        } else {
            None // duration <= 0.0 意味着无限生命周期
        };
        Self { timer, mode }
    }

    pub fn new_timer(duration: f32) -> Self {
        let timer = if duration > 0.0 {
            Some(Timer::from_seconds(duration, TimerMode::Once))
        } else {
            None // duration <= 0.0 意味着无限生命周期
        };
        Self {
            timer,
            mode: LifetimeMode::Timer,
        }
    }

    /// 检查生命周期是否结束。
    /// 如果 timer 是 None (无限生命周期), 永远返回 false。
    pub fn is_dead(self: &Self) -> bool {
        self.timer.as_ref().map_or(false, |t| t.is_finished())
    }

    /// 检查生命周期是否仍在进行中。
    /// 如果 timer 是 None (无限生命周期), 永远返回 true。
    pub fn is_alive(self: &Self) -> bool {
        !self.is_dead()
    }

    /// 返回生命周期的进度 (0.0 到 1.0)。
    /// 如果是无限生命周期 (duration <= 0.0)，永远返回 0.0。
    pub fn progress(&self) -> f32 {
        self.timer.as_ref().map_or(0.0, |t| {
            // 构造函数保证了 duration > 0.0
            (t.elapsed_secs() / t.duration().as_secs_f32()).clamp(0.0, 1.0)
        })
    }

    /// 返回已经过的时间。
    /// 如果是无限生命周期，返回 0.0。
    pub fn elapsed_secs(&self) -> f32 {
        self.timer.as_ref().map_or(0.0, |t| t.elapsed_secs())
    }

    /// 立即结束生命周期。
    /// 对无限生命周期的实体无效。
    pub fn dead(&mut self) {
        if let Some(timer) = self.timer.as_mut() {
            timer.tick(timer.duration());
        }
    }
}

fn update(
    mut commands: Commands,
    mut q_lifetime: Query<(Entity, &mut Lifetime)>,
    q_children: Query<&Children>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in q_lifetime.iter_mut() {
        if lifetime.is_alive() {
            // 如果 timer 存在 (即非无限生命周期)，则推进它
            if let Some(timer) = lifetime.timer.as_mut() {
                timer.tick(time.delta());
            }
            // 无论是无限生命周期 (None) 还是仍在计时的 (Some)，都继续
            continue;
        }

        // 如果执行到这里，意味着 is_alive() 为 false，即 is_dead() 为 true
        // 这只可能在 timer 是 Some(finished_timer) 时发生
        match lifetime.mode {
            LifetimeMode::Timer => commands.entity(entity).despawn(),
            LifetimeMode::TimerAndNoChildren => {
                if q_children.get(entity).is_err() {
                    commands.entity(entity).despawn()
                }
            }
        }
    }
}
