//! 统一位移动作系统
//!
//! 英雄技能只需描述「对谁、往哪移、附带什么效果」，
//! 由 [`ActionDisplace`] 事件统一处理目标选择、运动模式与效果挂载。
//!
//! # 使用此机制的英雄
//!
//! | 英雄 | 技能 | TargetSelection | Motion | Effects |
//! |------|------|-----------------|--------|---------|
//! | Darius E | 无情立场 | Cone{535, 90°} | PullToward{535} | Knockup(0.75)+Slow|
//! | Sett E | 迎面痛击 | Cone{490, 90°} | PullToward{490} | DualSide(Stun/Slow)|
//! | Mordekaiser E | 断魂一拽 | Cone{550, 100°} | PullToward{250} | Damage |
//! | Riven Q3 | 折翼第三段 | Circle{250} | PushAway{75} | Knockup(0.75)+Damage|
//! | Aatrox W 引爆 | 冥府之链 | Explicit | PullToPoint | Knockup+Damage|
//! | Sett R | 消防官 | Nearest{475} | GrabAndFollow | AoE Damage+Slow|
//! | Camille E2 | 钩索命中英雄 | Circle{150} | PushAway | Stun(1.0)|
//! | Volibear R落地 | 风暴之怒 | Circle{300} | None | Damage+Slow|

use bevy::prelude::*;

use crate::base::buff::BuffOf;
use crate::buffs::cc_debuffs::{DebuffKnockup, DebuffSlow, DebuffStun};
use crate::damage::{CommandDamageCreate, DamageType};
use crate::life::Death;
use crate::movement::{CommandMovement, MovementAction, MovementSource, MovementWay};
use crate::team::Team;

// ---------------------------------------------------------------------------
// 公共类型
// ---------------------------------------------------------------------------

/// 目标选择策略
#[derive(Debug, Clone)]
pub enum DisplaceTargetSelection {
    /// 锥形范围内所有敌方英雄（Darius E / Sett E / Mordekaiser E）
    Cone {
        range: f32,
        angle: f32,
        direction: Vec2,
    },
    /// 圆形范围内所有敌方英雄（Riven Q3 / Volibear R 落地）
    Circle {
        radius: f32,
        center: DisplaceCenter,
    },
    /// 最近的单个敌方英雄（Sett R 抓取）
    Nearest {
        range: f32,
    },
    /// 由英雄自行筛选后传入
    Explicit(Vec<Entity>),
}

/// 圆形中心模式
#[derive(Debug, Clone, Copy)]
pub enum DisplaceCenter {
    Caster,
    Point(Vec2),
}

/// 位移运动描述
#[derive(Debug, Clone)]
pub enum DisplaceMotion {
    /// 向施法者拉回（Darius E / Mordekaiser E）
    PullToward {
        distance: f32,
        speed: f32,
    },
    /// 从施法者击退（Riven Q3）
    PushAway {
        distance: f32,
        speed: f32,
    },
    /// 拉回到指定点（Aatrox W 引爆）
    PullToPoint {
        point: Vec2,
        distance: f32,
        speed: f32,
    },
    /// 无位移，仅施加效果
    None,
}

/// 命中附带效果
#[derive(Debug, Clone)]
pub enum DisplaceEffect {
    Knockup { duration: f32 },
    Stun { duration: f32 },
    Slow { percent: f32, duration: f32 },
    Damage {
        amount: f32,
        damage_type: DamageType,
        tag: Option<u32>,
    },
}

/// Sett E 双侧检测策略：前后两个锥形均命中 → 升级为 Stun
#[derive(Debug, Clone)]
pub struct ConeHitPolicy {
    /// 前后锥形均命中的眩晕时长
    pub stun_duration: f32,
    /// 单侧命中的减速比例
    pub slow_percent: f32,
    /// 单侧命中的减速时长
    pub slow_duration: f32,
}

/// 统一位移动作
#[derive(Debug, Clone, EntityEvent)]
pub struct ActionDisplace {
    pub entity: Entity,
    pub targets: DisplaceTargetSelection,
    pub motion: DisplaceMotion,
    pub effects: Vec<DisplaceEffect>,
    /// Sett E 双锥检测策略：Some 时启用前后锥形双检
    pub cone_hit_policy: Option<ConeHitPolicy>,
}

// ---------------------------------------------------------------------------
// GrabbedBy 组件（Sett R 抱人用）
// ---------------------------------------------------------------------------

/// 被抓取标记：挂在被抓者身上，每帧同步到 grabber 位置
#[derive(Component, Debug)]
pub struct GrabbedBy {
    pub grabber: Entity,
}

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 判断目标是否在锥形内
fn in_cone(
    target_pos: Vec3,
    origin: Vec3,
    range: f32,
    half_angle: f32,
    direction: Vec2,
) -> bool {
    let diff = (target_pos - origin).xz();
    let distance = diff.length();
    if distance > range || distance == 0.0 {
        return false;
    }
    let target_dir = diff.normalize();
    direction.dot(target_dir).acos() <= half_angle
}

/// 选择目标：从世界 Query 中筛选
fn select_targets(
    selection: &DisplaceTargetSelection,
    entity: Entity,
    q_transform: &Query<&Transform>,
    q_targets: &Query<
        (Entity, &Team, &Transform),
        (Without<Death>, Without<GrabbedBy>),
    >,
    team: &Team,
) -> Vec<Entity> {
    match selection {
        DisplaceTargetSelection::Cone {
            range,
            angle,
            direction,
        } => {
            let Ok(transform) = q_transform.get(entity) else {
                return vec![];
            };
            let origin = transform.translation;
            let half_angle = angle.to_radians() / 2.0;
            q_targets
                .iter()
                .filter(|(_, t, tf)| {
                    *t != team && in_cone(tf.translation, origin, *range, half_angle, *direction)
                })
                .map(|(e, _, _)| e)
                .collect()
        }
        DisplaceTargetSelection::Circle { radius, center } => {
            let origin = match center {
                DisplaceCenter::Caster => {
                    q_transform.get(entity).map(|t| t.translation).unwrap_or_default()
                }
                DisplaceCenter::Point(p) => Vec3::new(p.x, 0.0, p.y),
            };
            q_targets
                .iter()
                .filter(|(_, t, tf)| {
                    *t != team && tf.translation.distance(origin) <= *radius
                })
                .map(|(e, _, _)| e)
                .collect()
        }
        DisplaceTargetSelection::Nearest { range } => {
            let Ok(transform) = q_transform.get(entity) else {
                return vec![];
            };
            let origin = transform.translation;
            let mut best_dist = *range;
            let mut best = None;
            for (e, t, tf) in q_targets.iter() {
                if t == team {
                    continue;
                }
                let d = tf.translation.distance(origin);
                if d < best_dist {
                    best_dist = d;
                    best = Some(e);
                }
            }
            best.into_iter().collect()
        }
        DisplaceTargetSelection::Explicit(entities) => entities.clone(),
    }
}

/// 应用单个位移效果
///
/// 使用 `CommandMovement` 直接控制位移，不通过 `CommandKnockback`，
/// 因为后者会自动施加 `DebuffKnockup`，而 CC 效果由 `apply_effect` 统一管理。
fn apply_motion(
    commands: &mut Commands,
    target: Entity,
    source: Entity,
    motion: &DisplaceMotion,
    q_transform: &Query<&Transform>,
) {
    match motion {
        DisplaceMotion::PullToward { distance, speed } => {
            let Ok(target_transform) = q_transform.get(target) else {
                return;
            };
            let Ok(source_transform) = q_transform.get(source) else {
                return;
            };
            let diff = source_transform.translation.xz() - target_transform.translation.xz();
            let direction = diff.normalize_or_zero();
            // 钳制距离：不越过 source
            let clamped = distance.min(diff.length());
            let dest_xz = target_transform.translation.xz() + direction * clamped;
            let destination = Vec3::new(dest_xz.x, target_transform.translation.y, dest_xz.y);
            commands.entity(target).trigger(|e| CommandMovement {
                entity: e,
                priority: 100,
                action: MovementAction::Start {
                    way: MovementWay::Path(vec![destination]),
                    speed: Some(*speed),
                    source: MovementSource::Knockback,
                },
            });
        }
        DisplaceMotion::PushAway { distance, speed } => {
            let Ok(target_transform) = q_transform.get(target) else {
                return;
            };
            let Ok(source_transform) = q_transform.get(source) else {
                return;
            };
            let diff = target_transform.translation.xz() - source_transform.translation.xz();
            let direction = diff.normalize_or_zero();
            let dest_xz = target_transform.translation.xz() + direction * distance;
            let destination = Vec3::new(dest_xz.x, target_transform.translation.y, dest_xz.y);
            commands.entity(target).trigger(|e| CommandMovement {
                entity: e,
                priority: 100,
                action: MovementAction::Start {
                    way: MovementWay::Path(vec![destination]),
                    speed: Some(*speed),
                    source: MovementSource::Knockback,
                },
            });
        }
        DisplaceMotion::PullToPoint { point, distance, speed } => {
            let Ok(target_transform) = q_transform.get(target) else {
                return;
            };
            let target_pos = target_transform.translation.xz();
            let direction = (point - target_pos).normalize_or_zero();
            // 钳制距离：不超过到点的实际距离
            let actual_dist = target_pos.distance(*point);
            let clamped = distance.min(actual_dist);
            let dest_xz = target_transform.translation.xz() + direction * clamped;
            let destination = Vec3::new(dest_xz.x, target_transform.translation.y, dest_xz.y);
            commands.entity(target).trigger(|e| CommandMovement {
                entity: e,
                priority: 100,
                action: MovementAction::Start {
                    way: MovementWay::Path(vec![destination]),
                    speed: Some(*speed),
                    source: MovementSource::Knockback,
                },
            });
        }
        DisplaceMotion::None => {}
    }
}

/// 应用单个效果到目标
fn apply_effect(
    commands: &mut Commands,
    target: Entity,
    source: Entity,
    effect: &DisplaceEffect,
) {
    match effect {
        DisplaceEffect::Knockup { duration } => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffKnockup::new(*duration));
        }
        DisplaceEffect::Stun { duration } => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffStun::new(*duration));
        }
        DisplaceEffect::Slow { percent, duration } => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffSlow::new(*percent, *duration));
        }
        DisplaceEffect::Damage {
            amount,
            damage_type,
            tag,
        } => {
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source,
                damage_type: *damage_type,
                amount: *amount,
                tag: *tag,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Observer
// ---------------------------------------------------------------------------

/// 处理 `ActionDisplace` 事件
pub fn on_action_displace(
    trigger: On<ActionDisplace>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_targets: Query<
        (Entity, &Team, &Transform),
        (Without<Death>, Without<GrabbedBy>),
    >,
    q_team: Query<&Team>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_team.get(entity) else {
        return;
    };

    let targets = select_targets(&trigger.targets, entity, &q_transform, &q_targets, team);

    // Sett E 双侧检测
    if let Some(policy) = &trigger.cone_hit_policy {
        let Ok(transform) = q_transform.get(entity) else {
            return;
        };
        let origin = transform.translation;
        // 从 targets 的 selection 中读取锥形参数
        if let DisplaceTargetSelection::Cone {
            range,
            angle,
            direction,
        } = trigger.targets
        {
            let half_angle = angle.to_radians() / 2.0;
            // 前方锥形（用传入 direction），直接已有 targets
            let front_hit = !targets.is_empty();

            // 后方锥形（反向 direction）
            let back_dir = -direction;
            let back_targets: Vec<Entity> = q_targets
                .iter()
                .filter(|(_, t, tf)| {
                    t != &team && in_cone(tf.translation, origin, range, half_angle, back_dir)
                })
                .map(|(e, _, _)| e)
                .collect();
            let back_hit = !back_targets.is_empty();

            // 收集所有目标（前方锥形 + 后方锥形）
            let all_targets: Vec<Entity> = targets
                .iter()
                .chain(back_targets.iter())
                .copied()
                .collect();
            let total_hit = !all_targets.is_empty();

            // 所有命中目标执行位移运动 + 效果（伤害）
            for t in &all_targets {
                apply_motion(&mut commands, *t, entity, &trigger.motion, &q_transform);
                for effect in &trigger.effects {
                    apply_effect(&mut commands, *t, entity, effect);
                }
            }

            // 双侧命中 → 全部眩晕；单侧 → 该侧减速
            if front_hit && back_hit && total_hit {
                for t in &all_targets {
                    commands
                        .entity(*t)
                        .with_related::<BuffOf>(DebuffStun::new(policy.stun_duration));
                }
            } else {
                for t in &targets {
                    commands
                        .entity(*t)
                        .with_related::<BuffOf>(DebuffSlow::new(policy.slow_percent, policy.slow_duration));
                }
                for t in &back_targets {
                    commands
                        .entity(*t)
                        .with_related::<BuffOf>(DebuffSlow::new(policy.slow_percent, policy.slow_duration));
                }
            }
            return;
        }
    }

    // 常规处理：位移 + 效果
    for target in &targets {
        apply_motion(&mut commands, *target, entity, &trigger.motion, &q_transform);
        for effect in &trigger.effects {
            apply_effect(&mut commands, *target, entity, effect);
        }
    }
}

// ---------------------------------------------------------------------------
// GrabbedBy 同步系统
// ---------------------------------------------------------------------------

/// 每帧同步被抓取者位置到 grabber
pub fn update_grabbed_entities(
    q_grabbed: Query<(Entity, &GrabbedBy)>,
    mut transforms: ParamSet<(
        Query<&Transform>,
        Query<&mut Transform>,
    )>,
) {
    // 先收集所有 grabber 位置
    let mut updates: Vec<(Entity, Vec3)> = Vec::new();
    for (grabbed_entity, grabbed) in q_grabbed.iter() {
        if let Ok(grabber_transform) = transforms.p0().get(grabbed.grabber) {
            updates.push((grabbed_entity, grabber_transform.translation));
        }
    }
    // 再写入被抓取者位置
    for (grabbed_entity, pos) in updates {
        if let Ok(mut target_transform) = transforms.p1().get_mut(grabbed_entity) {
            target_transform.translation = pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::action::PluginAction;
    use crate::buffs::cc_debuffs::PluginCc;
    use crate::movement::{Movement, PluginMovement};
    use crate::navigation::grid::ResourceGrid;
    use crate::navigation::navigation::PluginNavigaton;
    use crate::team::Team;
    use lol_base::grid::ConfigNavigationGrid;
    use lol_base::spell::Spell;

    fn app_with_grid() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
        app.add_plugins(PluginCc);
        app.init_asset::<Spell>();
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ConfigNavigationGrid>>()
            .add(ConfigNavigationGrid::default());
        app.insert_resource(ResourceGrid(handle));
        app
    }

    #[test]
    fn displace_cone_selects_enemies_in_cone() {
        let mut app = app_with_grid();
        let caster = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();
        let enemy = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(200.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();
        let behind = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(-200.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();

        app.world_mut()
            .entity_mut(caster)
            .trigger(|e| ActionDisplace {
                entity: e,
                targets: DisplaceTargetSelection::Cone {
                    range: 500.0,
                    angle: 90.0,
                    direction: Vec2::new(1.0, 0.0),
                },
                motion: DisplaceMotion::PullToward {
                    distance: 500.0,
                    speed: 1000.0,
                },
                effects: vec![DisplaceEffect::Knockup { duration: 0.75 }],
                cone_hit_policy: None,
            });

        // 几帧后 enemy 应被移到 caster 附近
        for _ in 0..15 {
            app.update();
        }

        let enemy_x = app.world().get::<Transform>(enemy).unwrap().translation.x;
        assert!(
            enemy_x.abs() < 50.0,
            "Cone PullToward 应把前方敌人拉到脚下，实际 x = {enemy_x}"
        );

        // 背后的敌人不应被拉到（不在锥形内）
        let behind_x = app.world().get::<Transform>(behind).unwrap().translation.x;
        assert!(
            behind_x < -150.0,
            "不在锥形内的敌人不应被拉动，实际 x = {behind_x}"
        );
    }

    #[test]
    fn displace_circle_selects_all_nearby() {
        let mut app = app_with_grid();
        let caster = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();
        let e1 = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(100.0, 0.0, 100.0),
                Movement { speed: 340.0 },
            ))
            .id();
        let e2 = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(300.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();

        app.world_mut()
            .entity_mut(caster)
            .trigger(|e| ActionDisplace {
                entity: e,
                targets: DisplaceTargetSelection::Circle {
                    radius: 250.0,
                    center: DisplaceCenter::Point(Vec2::new(100.0, 100.0)),
                },
                motion: DisplaceMotion::PushAway {
                    distance: 100.0,
                    speed: 1000.0,
                },
                effects: vec![],
                cone_hit_policy: None,
            });

        for _ in 0..15 {
            app.update();
        }

        // e1 (距离圆心≈0) 应被推开
        let e1_x = app.world().get::<Transform>(e1).unwrap().translation.x;
        assert!(
            e1_x > 10.0,
            "Circle 命中 e1 应被推开，实际 x = {e1_x}"
        );

        // e2 在 250 外 → Query 检查它未被击退（位置不变 ≈ 300）
        let e2_x = app.world().get::<Transform>(e2).unwrap().translation.x;
        assert!(
            e2_x > 250.0,
            "e2 超出半径不应被推，实际 x = {e2_x}"
        );
    }

    #[test]
    fn displace_with_stun_effect_adds_debuff() {
        let mut app = app_with_grid();
        let caster = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();
        let enemy = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(100.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();

        app.world_mut()
            .entity_mut(caster)
            .trigger(|e| ActionDisplace {
                entity: e,
                targets: DisplaceTargetSelection::Cone {
                    range: 500.0,
                    angle: 90.0,
                    direction: Vec2::new(1.0, 0.0),
                },
                motion: DisplaceMotion::None,
                effects: vec![DisplaceEffect::Stun { duration: 1.0 }],
                cone_hit_policy: None,
            });

        app.update();

        // 检查 enemy 是否有 Stun buff
        let buffs = app.world().get::<crate::base::buff::Buffs>(enemy);
        assert!(buffs.is_some(), "enemy 应有 Buffs 组件");
        let has_stun = buffs.unwrap().iter().any(|e| {
            app.world().get::<DebuffStun>(e).is_some()
        });
        assert!(has_stun, "enemy 应有 DebuffStun");
    }

    #[test]
    fn test_grab_and_sync_position() {
        let mut app = app_with_grid();

        let grabber = app
            .world_mut()
            .spawn(Transform::from_xyz(100.0, 0.0, 200.0))
            .id();
        let grabbed = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                GrabbedBy { grabber },
            ))
            .id();

        for _ in 0..5 {
            app.update();
        }

        // 被抓者位置应同步到 grabber
        let grabbed_pos = app.world().get::<Transform>(grabbed).unwrap().translation;
        assert!(
            (grabbed_pos - Vec3::new(100.0, 0.0, 200.0)).length() < 0.01,
            "GrabbedBy 应同步到 grabber 位置，实际 {grabbed_pos:?}"
        );
    }
}