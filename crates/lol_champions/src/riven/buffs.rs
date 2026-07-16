use bevy::prelude::*;
use lol_core::base::buff::Buff;
use lol_core::buffs::shield_white::BuffShieldWhite;

/// R 被动 buff，作为子实体添加
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RivenR" })]
pub struct BuffRivenR {
    pub timer: Timer,
    /// 存储开启时增加的 AD 比例（从 RON PercentBonusAD 读取），用于到期恢复
    pub bonus_ad_ratio: f32,
    /// 存储开启时增加的攻击距离（从 RON TooltipAttackRange 读取），用于到期恢复
    pub bonus_range: f32,
}

/// 标记 Q3 位移中，落地后以落点为圆心触发范围伤害 + 震退
#[derive(Component, Debug, Default)]
pub struct RivenQ3Pending {
    /// Q3 落点范围伤害数值（位移结束时一次性结算）
    pub damage: f32,
}

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
