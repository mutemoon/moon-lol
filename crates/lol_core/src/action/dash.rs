use std::collections::HashSet;

use bevy::prelude::*;
use lol_base::spell::Spell;

use crate::action::damage::{TargetDamage, TargetFilter};
use crate::damage::{CommandDamageCreate, Damage};
use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::life::Death;
use crate::movement::{
    CommandMovement, EventMovementEnd, MovementAction, MovementSource, MovementState, MovementWay,
};
use crate::skill::{Skill, Skills, get_skill_value};
use crate::team::Team;

#[derive(Debug, Clone, EntityEvent)]
pub struct ActionDash {
    pub entity: Entity,
    pub move_type: DashMoveType,
    pub speed: f32,
    pub point: Vec2,
}

#[derive(Debug, Clone)]
pub enum DashMoveType {
    Fixed(f32),
    Pointer {
        max: f32,
    },
    /// 冲向绝对世界点（忽略 `point` 字段）。用于锚点拉拽（青钢影 E1 拉墙）、传送落点等。
    WorldPoint(Vec2),
    /// 冲向实体并追踪：每帧把终点重设为 target 当前位置，接触（<= stop_radius）即停。
    /// 用于刀妹 Q（冲向小兵/英雄）、青钢影 E2（冲向英雄）。
    Entity {
        target: Entity,
        stop_radius: f32,
    },
}

/// 追踪位移标记：由 `DashMoveType::Entity` 起手时插入。
/// `update_tracking_dash` 每帧把 `MovementState.path` 终点重设为 target 当前位置，
/// 接触或目标消失时完成位移（清路径、发 `EventMovementEnd`、移除自身）。
#[derive(Component, Debug)]
pub struct TrackingDash {
    pub target: Entity,
    pub stop_radius: f32,
}

#[derive(Debug, Clone)]
pub struct DashDamage {
    pub radius_end: f32,
    pub damage: TargetDamage,
}

#[derive(Component)]
pub struct DashDamageComponent {
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub damage: DashDamage,
    pub skill: Handle<Spell>,
    pub hit_entities: HashSet<Entity>,
}

/// 位移开始生命周期事件：由 `on_action_dash` 在解析出目的地后发出。
/// 伤害等副作用作为挂在此事件上的观察者实现（位移本身只管运动）。
#[derive(EntityEvent, Debug, Clone)]
pub struct EventDashStart {
    #[event_target]
    pub entity: Entity,
    pub start: Vec3,
    pub destination: Vec3,
}

/// 位移沿途伤害意图：champion 在触发 `ActionDash` 前插入此组件，
/// `on_dash_start_attach_damage` 观察 `EventDashStart` 时读取它、挂载 `DashDamageComponent`、
/// 随后移除意图。`ActionDash` 因此不再携带伤害字段（位移 = 纯运动）。
#[derive(Component, Debug, Clone)]
pub struct DashDamageIntent {
    pub damage: DashDamage,
    pub skill: Handle<Spell>,
}

pub fn on_action_dash(
    trigger: On<ActionDash>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
) {
    let entity = trigger.event_target();

    let Ok(transform) = q_transform.get(entity) else {
        debug!(
            "on_action_dash: entity {:?} has no Transform, skipping",
            entity
        );
        return;
    };

    let current_pos = transform.translation;
    let vector = trigger.point - current_pos.xz();
    let distance = vector.length();

    debug!(
        "on_action_dash: entity {:?} at ({:.2}, {:.2}, {:.2}) -> point ({:.2}, {:.2}), distance: {:.2}, speed: {:.2}",
        entity,
        current_pos.x,
        current_pos.y,
        current_pos.z,
        trigger.point.x,
        trigger.point.y,
        distance,
        trigger.speed
    );

    let (destination, move_type_desc) = match trigger.move_type {
        DashMoveType::Fixed(fixed_distance) => {
            let direction = if distance < 0.001 {
                transform.forward().xz().normalize()
            } else {
                vector.normalize()
            };
            let dest = current_pos.xz() + direction * fixed_distance;
            debug!(
                "on_action_dash: DashMoveType::Fixed distance {:.2}, direction ({:.2}, {:.2}), destination ({:.2}, {:.2})",
                fixed_distance, direction.x, direction.y, dest.x, dest.y
            );
            (dest, format!("Fixed({:.2})", fixed_distance))
        }
        DashMoveType::Pointer { max } => {
            let dest = if distance < max {
                debug!(
                    "on_action_dash: DashMoveType::Pointer distance {:.2} < max {:.2}, going to point",
                    distance, max
                );
                trigger.point
            } else {
                let direction = vector.normalize();
                let dest = current_pos.xz() + direction * max;
                debug!(
                    "on_action_dash: DashMoveType::Pointer distance {:.2} >= max {:.2}, clamping to max",
                    distance, max
                );
                dest
            };
            (dest, format!("Pointer(max: {:.2})", max))
        }
        DashMoveType::WorldPoint(p) => {
            debug!(
                "on_action_dash: DashMoveType::WorldPoint destination ({:.2}, {:.2})",
                p.x, p.y
            );
            (p, format!("WorldPoint({:.2},{:.2})", p.x, p.y))
        }
        DashMoveType::Entity {
            target,
            stop_radius,
        } => {
            let Ok(target_transform) = q_transform.get(target) else {
                debug!(
                    "on_action_dash: DashMoveType::Entity target {:?} has no Transform, skipping",
                    target
                );
                return;
            };
            let dest = target_transform.translation.xz();
            commands.entity(entity).insert(TrackingDash {
                target,
                stop_radius,
            });
            debug!(
                "on_action_dash: DashMoveType::Entity target {:?}, stop_radius {:.2}, destination ({:.2}, {:.2})",
                target, stop_radius, dest.x, dest.y
            );
            (
                dest,
                format!("Entity(target={:?}, stop={})", target, stop_radius),
            )
        }
    };

    debug!(
        "on_action_dash: triggering CommandMovement for entity {:?}, destination ({:.2}, {:.2}, {:.2}), move_type: {}",
        entity, destination.x, current_pos.y, destination.y, move_type_desc
    );
    let destination_y = current_pos.y;
    let destination = Vec3::new(destination.x, destination_y, destination.y);
    commands.trigger(CommandMovement {
        entity,
        priority: 100,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![destination]),
            speed: Some(trigger.speed),
            source: MovementSource::Dash,
        },
    });

    // 发出位移开始生命周期事件；沿途伤害等副作用由观察者挂载（见 on_dash_start_attach_damage）。
    commands.trigger(EventDashStart {
        entity,
        start: current_pos,
        destination,
    });
}

pub fn on_dash_end(
    trigger: On<crate::movement::EventMovementEnd>,
    mut commands: Commands,
    q: Query<&DashDamageComponent>,
) {
    let entity = trigger.event_target();
    if q.get(entity).is_ok() {
        commands.entity(entity).remove::<DashDamageComponent>();
    }
}

/// 位移开始时挂载沿途伤害：读取 `DashDamageIntent`，挂载 `DashDamageComponent`（携带起止点），
/// 随后移除意图。沿途伤害的实际计算由 `update_dash_damage` 每帧推进。
pub fn on_dash_start_attach_damage(
    trigger: On<EventDashStart>,
    mut commands: Commands,
    q_intent: Query<&DashDamageIntent>,
) {
    let entity = trigger.event_target();
    let Ok(intent) = q_intent.get(entity) else {
        return;
    };
    debug!(
        "on_dash_start_attach_damage: 为 {:?} 挂载沿途伤害 radius_end {:.2}",
        entity, intent.damage.radius_end
    );
    commands.entity(entity).insert(DashDamageComponent {
        start_pos: trigger.start,
        target_pos: trigger.destination,
        damage: intent.damage.clone(),
        skill: intent.skill.clone(),
        hit_entities: HashSet::default(),
    });
    commands.entity(entity).remove::<DashDamageIntent>();
}

pub fn update_dash_damage(
    mut commands: Commands,
    mut q_dasher: Query<(Entity, &Transform, &mut DashDamageComponent, &Team)>,
    q_target: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&Champion>,
            Option<&Minion>,
        ),
        Without<Death>,
    >,
    q_skills: Query<&Skills>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_assets_spell_object: Res<Assets<Spell>>,
) {
    for (entity, dasher_transform, mut dash_damage, team) in q_dasher.iter_mut() {
        let Some(skill_object) = res_assets_spell_object.get(&dash_damage.skill) else {
            continue;
        };
        let Ok(skills) = q_skills.get(entity) else {
            continue;
        };
        let skill = skills
            .iter()
            .map(|v| q_skill.get(v))
            .find_map(|v| v.ok())
            .unwrap();

        let start_pos = dash_damage.start_pos;
        let target_pos = dash_damage.target_pos;
        let current_pos = dasher_transform.translation;

        let total_dist = start_pos.distance(target_pos);
        if total_dist < 0.001 {
            continue;
        }
        let current_dist = start_pos.distance(current_pos);
        let progress = (current_dist / total_dist).clamp(0.0, 1.0);

        // TODO: Get actual radius from component
        let radius_start = 65.0;
        let current_radius =
            radius_start + (dash_damage.damage.radius_end - radius_start) * progress;

        for (target, target_transform, target_team, champion, minion) in q_target.iter() {
            if team == target_team {
                continue;
            }

            if dash_damage.hit_entities.contains(&target) {
                continue;
            }

            let apply = match dash_damage.damage.damage.filter {
                TargetFilter::All => true,
                TargetFilter::Champion => champion.is_some(),
                TargetFilter::Minion => minion.is_some(),
            };

            if !apply {
                continue;
            }

            let damage_amount = get_skill_value(
                &skill_object,
                &dash_damage.damage.damage.amount,
                skill.level,
                |stat| {
                    if stat == 2 {
                        if let Ok(damage) = q_damage.get(entity) {
                            return damage.0;
                        }
                    }
                    0.0
                },
            )
            .unwrap();

            if dasher_transform
                .translation
                .distance(target_transform.translation)
                <= current_radius
            {
                commands.entity(target).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source: entity,
                    damage_type: dash_damage.damage.damage.damage_type,
                    amount: damage_amount,
                    tag: None,
                });
                dash_damage.hit_entities.insert(target);
            }
        }
    }
}

/// 追踪位移系统：对带 `TrackingDash` 的实体，每帧把路径终点重设为 target 当前位置；
/// 接触（<= stop_radius）或目标消失时完成位移。固定更新阶段运行，先于 FixedPostUpdate 的位移应用。
pub fn update_tracking_dash(
    mut commands: Commands,
    mut q_dasher: Query<(Entity, &Transform, &mut MovementState, &TrackingDash)>,
    q_target: Query<&Transform>,
) {
    for (entity, dasher_transform, mut ms, tracking) in q_dasher.iter_mut() {
        // 目标消失：完成位移（若尚未完成）
        let target_pos = match q_target.get(tracking.target) {
            Ok(tf) => tf.translation,
            Err(_) => {
                if !ms.path.is_empty() {
                    ms.clear_path();
                    commands.trigger(EventMovementEnd {
                        entity,
                        source: MovementSource::Dash,
                    });
                }
                commands.entity(entity).remove::<TrackingDash>();
                continue;
            }
        };

        // 每帧重新瞄准 target 当前位置
        if !ms.path.is_empty() {
            ms.path[0] = target_pos;
        }

        // 接触即停
        let dist = dasher_transform.translation.distance(target_pos);
        if dist <= tracking.stop_radius {
            // 仅当位移尚未由 update_path_movement 完成时才发 EventMovementEnd，避免重复
            if !ms.path.is_empty() {
                ms.clear_path();
                commands.trigger(EventMovementEnd {
                    entity,
                    source: MovementSource::Dash,
                });
            }
            commands.entity(entity).remove::<TrackingDash>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::time::TimeUpdateStrategy;
    use lol_base::grid::ConfigNavigationGrid;
    use lol_base::spell::Spell;

    use super::*;
    use crate::action::PluginAction;
    use crate::movement::{Movement, PluginMovement};
    use crate::navigation::grid::ResourceGrid;
    use crate::navigation::navigation::PluginNavigaton;
    use crate::team::Team;

    fn app_with_grid() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
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

    fn spawn_dasher(app: &mut App, pos: Vec3) -> Entity {
        app.world_mut()
            .spawn((
                Team::Order,
                Transform::from_translation(pos),
                Movement { speed: 340.0 },
            ))
            .id()
    }

    fn pos_of(app: &App, e: Entity) -> Vec3 {
        app.world()
            .get::<Transform>(e)
            .expect("transform should exist")
            .translation
    }

    fn trigger_dash(app: &mut App, dasher: Entity, move_type: DashMoveType) {
        app.world_mut().entity_mut(dasher).trigger(|e| ActionDash {
            entity: e,
            move_type,
            speed: 1000.0,
            point: Vec2::ZERO,
        });
    }

    #[test]
    fn dash_world_point_goes_to_absolute_point() {
        let mut app = app_with_grid();
        let dasher = spawn_dasher(&mut app, Vec3::ZERO);

        trigger_dash(
            &mut app,
            dasher,
            DashMoveType::WorldPoint(Vec2::new(100.0, 0.0)),
        );
        for _ in 0..15 {
            app.update();
        }

        let x = pos_of(&app, dasher).x;
        assert!(
            (x - 100.0).abs() < 5.0,
            "WorldPoint 应冲向绝对点 (100,0)，实际 x = {x}"
        );
    }

    #[test]
    fn dash_entity_stops_at_target() {
        let mut app = app_with_grid();
        let dasher = spawn_dasher(&mut app, Vec3::ZERO);
        let target = app
            .world_mut()
            .spawn((Team::Chaos, Transform::from_xyz(300.0, 0.0, 0.0)))
            .id();

        trigger_dash(
            &mut app,
            dasher,
            DashMoveType::Entity {
                target,
                stop_radius: 30.0,
            },
        );
        for _ in 0..20 {
            app.update();
        }

        let d = pos_of(&app, dasher);
        let gap = (d - Vec3::new(300.0, 0.0, 0.0)).length();
        assert!(
            gap <= 31.0,
            "Entity 应在 stop_radius 内停止，距 target {gap}"
        );
        assert!(d.x > 260.0, "Entity 应已接近 target，实际 x = {}", d.x);
        assert!(
            !app.world().entity(dasher).contains::<TrackingDash>(),
            "完成后应移除 TrackingDash"
        );
    }

    #[test]
    fn dash_entity_tracks_moving_target() {
        let mut app = app_with_grid();
        let dasher = spawn_dasher(&mut app, Vec3::ZERO);
        let target = app
            .world_mut()
            .spawn((Team::Chaos, Transform::from_xyz(500.0, 0.0, 0.0)))
            .id();

        trigger_dash(
            &mut app,
            dasher,
            DashMoveType::Entity {
                target,
                stop_radius: 5.0,
            },
        );
        // 先朝 +x 跑几帧
        for _ in 0..3 {
            app.update();
        }
        // 目标瞬移到 -x 方向（dasher 身后），追踪系统应重新瞄准
        if let Some(mut t) = app.world_mut().get_mut::<Transform>(target) {
            t.translation = Vec3::new(-500.0, 0.0, 0.0);
        }
        for _ in 0..40 {
            app.update();
        }

        let x = pos_of(&app, dasher).x;
        assert!(
            x < -490.0,
            "追踪系统应跟随移动后的目标到 -500 附近，实际 x = {x}"
        );
    }
}
