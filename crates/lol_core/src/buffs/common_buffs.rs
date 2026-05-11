use bevy::prelude::*;

use crate::base::buff::Buff;
use crate::movement::{CastBlock, MovementBlock};

/// 施法期间阻塞 buff（通用）
/// 阻止移动和技能施放
#[derive(Component, Debug)]
#[require(CastBlock)]
pub struct BuffCastBlock {
    pub timer: Timer,
}

impl BuffCastBlock {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 更新施法阻塞计时，结束后移除阻塞
pub fn update_buff_cast_block(
    mut commands: Commands,
    mut q_cast_block: Query<(Entity, &mut BuffCastBlock)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut cast_block) in q_cast_block.iter_mut() {
        cast_block.timer.tick(time.delta());
        if cast_block.timer.is_finished() {
            commands.entity(entity).remove::<BuffCastBlock>();
            commands.entity(entity).remove::<MovementBlock>();
            commands.entity(entity).remove::<CastBlock>();
        }
    }
}

/// 移动速度加成 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MoveSpeed" })]
pub struct BuffMoveSpeed {
    pub bonus_percent: f32,
    pub timer: Timer,
}

impl BuffMoveSpeed {
    pub fn new(bonus_percent: f32, duration: f32) -> Self {
        Self {
            bonus_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 双抗加成 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Resist" })]
pub struct BuffResist {
    pub armor: f32,
    pub magic_resist: f32,
    pub timer: Timer,
}

impl BuffResist {
    pub fn new(armor: f32, magic_resist: f32, duration: f32) -> Self {
        Self {
            armor,
            magic_resist,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 自我治疗 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SelfHeal" })]
pub struct BuffSelfHeal {
    pub amount: f32,
}

impl BuffSelfHeal {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}

/// 强化下次攻击 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "EmpoweredAttack" })]
pub struct BuffEmpoweredAttack {
    pub bonus_damage: f32,
    pub remaining_hits: u8,
}

impl BuffEmpoweredAttack {
    pub fn new(bonus_damage: f32, hits: u8) -> Self {
        Self {
            bonus_damage,
            remaining_hits: hits,
        }
    }
}
