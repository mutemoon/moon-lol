//! Camille E（钩索 / Hookshot）二段攻速加成。
//!
//! 完整 E 的地形钩索（E1 贴墙、E2 冲刺）按位移框架 Phase 4.2 暂缓；
//! 这里实现可独立验证的部分：E2 命中后获得攻速加成（`ASBuff`，持续 `ASDuration`）。
//! 攻速由通用 `BuffAttack` 承载，到期由 `BuffCamilleE` 计时器回收。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::attack::BuffAttack;
use lol_core::base::buff::{Buff, BuffOf};

/// E 攻速加成持续时间（ron ASDuration = 5s）。
pub const CAMILLE_E_AS_DURATION: f32 = 5.0;

/// E 攻速加成计时器：到期回收 `BuffAttack`。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleE" })]
pub struct BuffCamilleE {
    pub timer: Timer,
}

impl BuffCamilleE {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// E2 施法：挂攻速加成 + 计时 buff。
pub fn apply_camille_e_as(commands: &mut Commands, entity: Entity, as_percent: f32, duration: f32) {
    commands.entity(entity).insert(BuffAttack {
        bonus_attack_speed: as_percent,
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCamilleE::new(duration));
}

/// E 攻速计时：到期移除 `BuffAttack` 与计时 buff。
pub fn update_camille_e(
    mut commands: Commands,
    mut q: Query<(Entity, &BuffOf, &mut BuffCamilleE)>,
    time: Res<Time<Fixed>>,
) {
    for (e, bo, mut buff) in q.iter_mut() {
        buff.timer.tick(time.delta());
        if !buff.timer.is_finished() {
            continue;
        }
        commands.entity(bo.0).remove::<BuffAttack>();
        commands.entity(e).despawn();
    }
}
