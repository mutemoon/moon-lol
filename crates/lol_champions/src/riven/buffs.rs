use bevy::prelude::*;
use lol_core::base::buff::Buff;
use lol_core::buffs::shield_white::BuffShieldWhite;

/// W 眩晕 buff，添加在受影响的敌人身上
#[derive(Component, Debug, Clone)]
pub struct BuffStun {
    pub timer: Timer,
}

/// R 被动 buff，作为子实体添加
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RivenR" })]
pub struct BuffRivenR {
    pub timer: Timer,
}

/// 标记 Q3 位移中，落地后触发击退
#[derive(Component, Debug, Default)]
pub struct RivenQ3Pending;

/// E 护盾环绕视觉组件
/// 存储 3 个环绕子体，自动旋转表示护盾激活
#[derive(Component, Debug)]
pub struct ShieldVisual {
    pub children: [Entity; 3],
    pub angle: f32,
    /// 对应的 BuffShieldWhite 子实体，用于检测护盾是否消失
    pub buff_entity: Entity,
}

const SHIELD_ORBIT_RADIUS: f32 = 100.0;
const SHIELD_ORBIT_HEIGHT: f32 = 50.0;
const SHIELD_ORBIT_SPEED: f32 = 2.0; // rad/s

pub fn update_shield_visuals(
    mut q: Query<&mut ShieldVisual>,
    mut q_transform: Query<&mut Transform>,
    time: Res<Time<Fixed>>,
) {
    for mut visual in q.iter_mut() {
        visual.angle += time.delta_secs() * SHIELD_ORBIT_SPEED;
        for (i, &child) in visual.children.iter().enumerate() {
            if let Ok(mut transform) = q_transform.get_mut(child) {
                let total_angle = visual.angle + i as f32 * core::f32::consts::TAU / 3.0;
                transform.translation = Vec3::new(
                    SHIELD_ORBIT_RADIUS * total_angle.cos(),
                    SHIELD_ORBIT_HEIGHT,
                    SHIELD_ORBIT_RADIUS * total_angle.sin(),
                );
            }
        }
    }
}

pub fn cleanup_shield_visuals(
    mut commands: Commands,
    q: Query<(Entity, &ShieldVisual)>,
    q_buff: Query<(), With<BuffShieldWhite>>,
) {
    for (entity, visual) in q.iter() {
        if q_buff.get(visual.buff_entity).is_err() {
            // 护盾 buff 已消失，清理视觉
            for &child in visual.children.iter() {
                commands.entity(child).despawn();
            }
            commands.entity(entity).remove::<ShieldVisual>();
        }
    }
}
