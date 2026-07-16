//! 延迟范围伤害系统
//!
//! 一条命令 `ActionDelayedDamage` 同时驱动伤害逻辑与三阶段视觉生命周期：
//! - **Delay**：指示器出现（可生长/脉动/淡入），不造成伤害
//! - **Impact**：延迟结束瞬间结算伤害（复用 `apply_damage_effects`），指示器爆发
//! - **Fade**：指示器缩小并淡出（alpha 渐变），到期销毁
//!
//! 伤害在施法瞬间快照的位置/朝向上结算，不跟随施法者。
//! 视觉由 [`AoEVisual`] 组件承载：本系统驱动其 `alpha`，`lol_render` 负责构建形状 mesh
//! 并将 `alpha` 同步到材质。

use bevy::prelude::*;
use lol_base::spell::Spell;

use crate::action::damage::{ActionDamageEffect, DamageShape, apply_damage_effects};
use crate::damage::{AbilityPower, Damage};
use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::team::Team;

/// 视觉指示器配置（三阶段外观）
#[derive(Debug, Clone)]
pub struct AoEIndicator {
    /// 延迟阶段基础颜色（RGB；alpha 由生命周期系统单独驱动）
    pub color: Color,
    /// 延迟阶段是否脉动闪烁（scale 脉动 + alpha 闪烁）
    pub pulse: bool,
    /// 是否从零生长到完整尺寸（如狗熊 E 从天而降），同时淡入
    pub grow_from_zero: bool,
    /// 爆发阶段缩放倍数（相对完整尺寸）
    pub impact_burst_scale: f32,
    /// 褪去持续时间（秒）
    pub fade_duration: f32,
}

impl Default for AoEIndicator {
    fn default() -> Self {
        Self {
            color: Color::srgba(1.0, 0.6, 0.2, 0.4),
            pulse: false,
            grow_from_zero: false,
            impact_burst_scale: 1.3,
            fade_duration: 0.3,
        }
    }
}

/// 延迟伤害的原点模式
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AoEOrigin {
    /// 以施法者为中心（默认）：矩形/扇形从施法者向外，圆形以施法者为圆心
    #[default]
    Caster,
    /// 以施法目标点为中心（地面靶向 AoE，如狗熊 E 落雷）
    CastPoint,
}

/// 延迟范围伤害命令
#[derive(Debug, Clone, EntityEvent)]
pub struct ActionDelayedDamage {
    pub entity: Entity,
    pub skill: Handle<Spell>,
    /// 技能等级（用于伤害公式与 data value 读取）
    pub skill_level: usize,
    /// 延迟秒数（可从 castFrame / 30 计算，或读 ChargeDuration 等 dataValue）
    pub delay: f32,
    /// 施法目标点（XZ）：方向性形状的朝向由此与施法者位置之差决定，
    /// 重合时退回 Transform 面向方向。与 Darius E 的锥形朝向逻辑一致。
    pub point: Vec2,
    pub effects: Vec<ActionDamageEffect>,
    pub indicator: AoEIndicator,
    /// 伤害原点：以施法者还是施法目标点为中心
    pub origin: AoEOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayedDamagePhase {
    /// 延迟阶段：指示器显示，不造成伤害
    Delay,
    /// 褪去阶段：伤害已结算，指示器缩小淡出
    Fade,
}

/// AoE 视觉组件：携带形状（决定 mesh 几何）与颜色/alpha。
///
/// `alpha` 由 [`update_delayed_damage`] 每帧写入；`lol_render` 的同步系统将其应用到材质。
#[derive(Component, Debug, Clone)]
pub struct AoEVisual {
    /// 形状（决定 mesh 几何，由 lol_render 读取）
    pub shape: DamageShape,
    /// 基础颜色（RGB），alpha 由 `alpha` 字段单独驱动
    pub color: Color,
    /// 当前 alpha（0..1），由生命周期系统驱动
    pub alpha: f32,
}

/// 延迟伤害实例（挂载在世界空间指示器实体上）
#[derive(Component, Debug)]
pub struct DelayedDamageInstance {
    pub caster: Entity,
    pub team: Team,
    pub skill: Handle<Spell>,
    pub skill_level: usize,
    /// 施法瞬间快照位置
    pub origin: Vec3,
    /// 施法瞬间快照朝向（XZ）
    pub forward: Vec2,
    pub effects: Vec<ActionDamageEffect>,
    pub indicator: AoEIndicator,
    pub delay_timer: Timer,
    pub fade_timer: Option<Timer>,
    pub phase: DelayedDamagePhase,
    pub applied: bool,
}

/// 指示器离地高度，避免与地面 z-fighting
const INDICATOR_Y_LIFT: f32 = 0.1;

/// 计算伤害原点（纯函数，便于测试）
fn compute_origin(mode: AoEOrigin, caster_pos: Vec3, point: Vec2) -> Vec3 {
    match mode {
        AoEOrigin::Caster => caster_pos,
        // 地面靶向：用施法点的 XZ 与施法者的 Y（贴地）
        AoEOrigin::CastPoint => Vec3::new(point.x, caster_pos.y, point.y),
    }
}

/// 将 XZ 平面朝向转为绕 Y 轴旋转的四元数（+X 朝向为默认）
fn forward_to_rotation(forward: Vec2) -> Quat {
    // atan2(z, x)：forward=(x,z) 在 XZ 平面相对 +X 的偏角
    let angle = forward.y.atan2(forward.x);
    Quat::from_rotation_y(angle)
}

pub fn on_action_delayed_damage(
    trigger: On<ActionDelayedDamage>,
    mut commands: Commands,
    q_caster: Query<(&Team, &Transform)>,
) {
    let event = trigger.event();
    let caster = event.entity;

    let shape = event
        .effects
        .first()
        .map(|e| e.shape.clone())
        .unwrap_or_default();

    // 施法瞬间快照队伍/位置/朝向
    let (team, origin, forward) = match q_caster.get(caster) {
        Ok((t, tf)) => {
            let pos = tf.translation;
            // 方向性形状朝向：施法点方向，重合时退回 Transform 面向方向
            let fwd = (event.point - pos.xz()).normalize_or_zero();
            let fwd = if fwd == Vec2::ZERO {
                tf.forward().xz()
            } else {
                fwd
            };
            let origin = compute_origin(event.origin, pos, event.point);
            (*t, origin, fwd)
        }
        Err(_) => (
            Team::default(),
            compute_origin(event.origin, Vec3::ZERO, event.point),
            Vec2::new(1.0, 0.0),
        ),
    };

    // 生长型从 0 尺寸/0 alpha 起步，否则直接满尺寸/满 alpha
    let initial_scale = if event.indicator.grow_from_zero {
        0.0
    } else {
        1.0
    };
    let initial_alpha = if event.indicator.grow_from_zero {
        0.0
    } else {
        1.0
    };

    let delay_timer = Timer::from_seconds(event.delay.max(0.0), TimerMode::Once);

    // 世界空间指示器实体（不跟随施法者）：mesh 由 lol_render 按 shape 构建，
    // 故 Transform.scale 只承载生命周期相位因子（0..1 / burst），不再乘半径
    commands.spawn((
        DelayedDamageInstance {
            caster,
            team,
            skill: event.skill.clone(),
            skill_level: event.skill_level,
            origin,
            forward,
            effects: event.effects.clone(),
            indicator: event.indicator.clone(),
            delay_timer,
            fade_timer: None,
            phase: DelayedDamagePhase::Delay,
            applied: false,
        },
        AoEVisual {
            shape: shape.clone(),
            color: event.indicator.color,
            alpha: initial_alpha,
        },
        Transform::from_translation(origin + Vec3::Y * INDICATOR_Y_LIFT)
            .with_rotation(forward_to_rotation(forward))
            .with_scale(Vec3::new(initial_scale, 1.0, initial_scale)),
    ));
}

/// FixedUpdate 系统：推进延迟伤害生命周期并在 Impact 结算伤害，
/// 同时驱动 [`AoEVisual`] 的 alpha 与 Transform 的 scale 相位因子。
pub fn update_delayed_damage(
    mut commands: Commands,
    mut q_inst: Query<(
        Entity,
        &mut DelayedDamageInstance,
        &mut Transform,
        &mut AoEVisual,
    )>,
    res_assets_spell: Res<Assets<Spell>>,
    q_target: Query<
        (
            Entity,
            &Team,
            Option<&Champion>,
            Option<&Minion>,
            &Transform,
        ),
        Without<DelayedDamageInstance>,
    >,
    q_damage: Query<&Damage>,
    q_ap: Query<&AbilityPower>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut inst, mut transform, mut visual) in q_inst.iter_mut() {
        match inst.phase {
            DelayedDamagePhase::Delay => {
                inst.delay_timer.tick(time.delta());
                let progress = if inst.delay_timer.duration().as_secs_f32() > 0.0 {
                    (inst.delay_timer.elapsed_secs() / inst.delay_timer.duration().as_secs_f32())
                        .clamp(0.0, 1.0)
                } else {
                    1.0
                };

                // 基础相位：生长型随进度放大并淡入，否则满尺寸/满 alpha
                let (mut scale, mut alpha) = if inst.indicator.grow_from_zero {
                    (progress, progress)
                } else {
                    (1.0, 1.0)
                };

                // 脉动：scale 微胀缩 + alpha 闪烁
                if inst.indicator.pulse {
                    let p = (inst.delay_timer.elapsed_secs() * 12.0).sin();
                    scale *= 1.0 + 0.1 * p;
                    alpha *= 0.8 + 0.2 * (p * 0.5 + 0.5);
                }

                transform.scale = Vec3::new(scale, 1.0, scale);
                visual.alpha = alpha.clamp(0.0, 1.0);

                if inst.delay_timer.is_finished() && !inst.applied {
                    // Impact：结算伤害
                    if let Some(skill_object) = res_assets_spell.get(&inst.skill) {
                        apply_damage_effects(
                            &mut commands,
                            inst.caster,
                            inst.origin,
                            inst.forward,
                            &inst.team,
                            &inst.effects,
                            skill_object,
                            inst.skill_level,
                            &q_target,
                            &q_damage,
                            &q_ap,
                        );
                    }
                    inst.applied = true;
                    // 爆发缩放 + 满 alpha，进入褪去
                    let burst = inst.indicator.impact_burst_scale;
                    transform.scale = Vec3::new(burst, 1.0, burst);
                    visual.alpha = 1.0;
                    inst.fade_timer = Some(Timer::from_seconds(
                        inst.indicator.fade_duration.max(0.0),
                        TimerMode::Once,
                    ));
                    inst.phase = DelayedDamagePhase::Fade;
                }
            }
            DelayedDamagePhase::Fade => {
                let burst = inst.indicator.impact_burst_scale;
                if let Some(timer) = inst.fade_timer.as_mut() {
                    timer.tick(time.delta());
                    let progress = if timer.duration().as_secs_f32() > 0.0 {
                        (timer.elapsed_secs() / timer.duration().as_secs_f32()).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    // 从爆发尺寸缩到 0（塌缩）+ alpha 从 1 淡到 0（溶解）
                    let scale = burst * (1.0 - progress);
                    let alpha = 1.0 - progress;
                    transform.scale = Vec3::new(scale, 1.0, scale);
                    visual.alpha = alpha.clamp(0.0, 1.0);
                    if timer.is_finished() {
                        commands.entity(entity).despawn();
                    }
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_origin_caster_vs_cast_point() {
        let caster = Vec3::new(100.0, 5.0, 200.0);
        let point = Vec2::new(800.0, 600.0);
        // Caster 模式：原点即施法者位置
        assert_eq!(compute_origin(AoEOrigin::Caster, caster, point), caster);
        // CastPoint 模式：原点为施法点 XZ，但 Y 贴施法者（地面）
        assert_eq!(
            compute_origin(AoEOrigin::CastPoint, caster, point),
            Vec3::new(800.0, 5.0, 600.0)
        );
    }

    #[test]
    fn forward_to_rotation_points_along_forward() {
        // +X 朝向应为单位四元数
        let q = forward_to_rotation(Vec2::new(1.0, 0.0));
        let v = q * Vec3::new(1.0, 0.0, 0.0);
        assert!(v.x.abs() > 0.99, "+X 朝向应保持 +X");

        // +Z 朝向（forward=(0,1) in XZ）应把 +X 旋到 +Z
        let q = forward_to_rotation(Vec2::new(0.0, 1.0));
        let v = q * Vec3::new(1.0, 0.0, 0.0);
        assert!(v.z.abs() > 0.99, "+Z 朝向应把 +X 旋到 +Z");
    }
}
