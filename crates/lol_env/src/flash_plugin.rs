use bevy::prelude::*;

/// 默认闪现冷却时间（秒）
pub const FLASH_COOLDOWN_SECS: f32 = 300.0;
/// 默认闪现位移距离
pub const FLASH_DISTANCE: f32 = 300.0;

/// 闪现冷却组件。
/// 挂载在英雄实体上，通过 [`tick_flash_cooldown`] 系统在 `FixedUpdate` 中推进计时。
#[derive(Component, Default, Debug, Clone)]
pub struct FlashCooldown(pub Option<Timer>);

impl FlashCooldown {
    pub fn is_ready(&self) -> bool {
        self.0.as_ref().map_or(true, |t| t.is_finished())
    }

    pub fn remaining_secs(&self) -> f32 {
        self.0
            .as_ref()
            .map(|t| {
                if t.is_finished() {
                    0.0
                } else {
                    t.remaining_secs()
                }
            })
            .unwrap_or(0.0)
    }

    /// 启动闪现冷却（默认 300 秒）
    pub fn start(&mut self) {
        self.0 = Some(Timer::from_seconds(FLASH_COOLDOWN_SECS, TimerMode::Once));
    }

    /// 启动指定时长的冷却
    pub fn start_duration(&mut self, duration_secs: f32) {
        self.0 = Some(Timer::from_seconds(duration_secs, TimerMode::Once));
    }

    /// 重置冷却为就绪
    pub fn reset(&mut self) {
        self.0 = None;
    }
}

/// FixedUpdate 中推进所有实体的闪现冷却计时器。
pub fn tick_flash_cooldown(time: Res<Time<Fixed>>, mut q: Query<&mut FlashCooldown>) {
    for mut flash in q.iter_mut() {
        if let Some(timer) = flash.0.as_mut() {
            timer.tick(time.delta());
            if timer.is_finished() {
                flash.0 = None;
            }
        }
    }
}

/// 注册闪现插件系统到 App。
pub fn register_flash_plugin(app: &mut App) {
    app.add_systems(FixedUpdate, tick_flash_cooldown);
}

/// 从 ECS World 提取指定实体的闪现状态 `(is_ready, remaining_secs)`。
pub fn extract_flash_obs(world: &World, entity: Entity) -> (bool, f32) {
    world
        .get::<FlashCooldown>(entity)
        .map(|f| (f.is_ready(), f.remaining_secs()))
        .unwrap_or((true, 0.0))
}

/// 派发闪现物理位移并启动冷却。
/// 如果实体没有 FlashCooldown 组件，则自动插入并启动。
pub fn dispatch_flash(world: &mut World, entity: Entity, direction: Vec3, distance: f32) -> bool {
    let is_ready = world
        .get::<FlashCooldown>(entity)
        .map(|f| f.is_ready())
        .unwrap_or(true);

    if !is_ready {
        return false;
    }

    let dir_normalized = if direction.length_squared() > 1e-4 {
        direction.normalize()
    } else {
        Vec3::X
    };

    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.translation += dir_normalized * distance;
    }

    if let Some(mut flash) = world.get_mut::<FlashCooldown>(entity) {
        flash.start();
    } else {
        let mut flash = FlashCooldown::default();
        flash.start();
        world.entity_mut(entity).insert(flash);
    }

    true
}
