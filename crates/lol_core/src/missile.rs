use bevy::color::palettes::tailwind::RED_500;
use bevy::prelude::*;
use lol_base::debug_area::DebugArea;
use lol_base::debug_missile::DebugMissile;
use lol_base::debug_sphere::DebugSphere;
use lol_base::grid::ConfigNavigationGrid;
use lol_base::movement::{MissileBehavior, MovementType};
use lol_base::spell::Spell;
use serde::{Deserialize, Serialize};

use crate::attack::EntityCommandsTrigger;
use crate::damage::{CommandDamageCreate, Damage, DamageType};
use crate::life::Death;
use crate::movement::{
    CommandMovement, EventMovementEnd, Movement, MovementAction, MovementSource, MovementWay,
};
use crate::navigation::grid::ResourceGrid;
use crate::team::Team;

#[derive(Default)]
pub struct PluginMissile;

impl Plugin for PluginMissile {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_missile_create);
        app.add_observer(on_event_movement_end);
        app.add_observer(on_command_attached_field_create);

        app.add_systems(FixedUpdate, fixed_update);
        app.add_systems(FixedUpdate, linear_missile_collision);
        app.add_systems(FixedUpdate, update_attached_fields);
        app.add_systems(FixedUpdate, update_wall_anchors);
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Debug, Component, Clone)]
pub struct Missile {
    pub key: Handle<Spell>,
    pub speed: f32,
}

/// 攻击状态机
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct MissileState {
    pub source: Entity,
    /// 追踪目标（None 表示直线导弹）
    pub target: Option<Entity>,
    pub target_bone: Option<Entity>,
    /// 直线导弹的目标点
    pub destination: Option<Vec3>,
}

/// 直线导弹标记组件
#[derive(Component, Debug)]
pub struct LinearMissile {
    pub width: f32,
    pub damage: f32,
    pub hit_enemies: Vec<Entity>,
    /// 粘性飞弹：检测地形碰撞，碰墙即锚定（不做实体碰撞）。
    pub sticky: bool,
}

#[derive(EntityEvent, Debug)]
pub struct CommandMissileCreate {
    pub entity: Entity,
    /// Some(entity) 为追踪导弹，None 为直线导弹
    pub target: Option<Entity>,
    /// 直线导弹的目标位置
    pub destination: Option<Vec3>,
    pub spell: Handle<Spell>,
    /// 直线导弹的伤害值（追踪导弹从 source.Damage 读取）
    pub damage: f32,
    /// 覆盖导弹飞行速度（None 则使用 spell data 中的 missileSpeed）
    pub speed: Option<f32>,
    /// 覆盖 missile effect particle（None 则使用 spell data 中的 missileEffectKey）
    pub particle_hash: Option<u32>,
    /// 粘性飞弹：碰墙锚定并发出 `EventMissileHit`，用于青钢影 E 钩索等
    pub sticky: bool,
}

/// 粘性飞弹碰墙后留下的锚点实体（世界定点，带生命周期）。
#[derive(Component, Debug)]
pub struct WallAnchor {
    pub timer: Timer,
}

/// 粘性飞弹碰墙事件，触发于 source（施法者），携带墙点与 spell 句柄供英雄观察者按技能分发。
#[derive(EntityEvent, Debug, Clone)]
pub struct EventMissileHit {
    #[event_target]
    pub source: Entity,
    pub spell: Handle<Spell>,
    pub point: Vec3,
}

/// WallAnchor 默认存活时长（秒），覆盖二段技能窗口
const WALL_ANCHOR_DURATION: f32 = 5.0;

/// 附着在施法者身上的范围伤害场组件
///
/// 作为施法者的子实体存在，跟随施法者移动。
/// 在持续时间内每帧检测范围内的敌人，每个敌人只造成一次伤害。
/// 支持半径从 radius_start 到 radius_end 随时间增长（grow_duration 控制增长时长）。
#[derive(Component, Debug)]
pub struct AttachedField {
    pub radius: f32,
    pub radius_start: f32,
    pub radius_end: f32,
    /// 半径从 start 增长到 end 所需的秒数（超过后保持 radius_end）
    pub grow_duration: f32,
    pub damage_amount: f32,
    pub hit_enemies: Vec<Entity>,
    pub timer: Timer,
}

/// 创建附着在实体上的伤害场
#[derive(EntityEvent, Debug)]
pub struct CommandAttachedFieldCreate {
    pub entity: Entity,
    /// 最终半径
    pub radius: f32,
    pub damage: f32,
    pub duration: f32,
    /// 半径起始值，Some(start) 表示从 start 增长到 radius，None 表示固定半径
    pub grow_from: Option<f32>,
    /// 半径增长持续时长（秒），不填则随 field duration 增长
    pub grow_duration: Option<f32>,
}

fn fixed_update(
    mut commands: Commands,
    q_missile: Query<(Entity, &Missile, &MissileState), Without<LinearMissile>>,
    q_transform: Query<&GlobalTransform>,
) {
    for (entity, missile, state) in q_missile.iter() {
        let Some(target_bone) = state.target_bone else {
            continue;
        };

        let Ok(target_transform) = q_transform.get(target_bone) else {
            continue;
        };

        let target_translation = target_transform.translation();
        commands.trigger(CommandMovement {
            entity,
            priority: 0,
            action: MovementAction::Start {
                way: MovementWay::Path(vec![target_translation]),
                speed: Some(missile.speed),
                source: MovementSource::Missile,
            },
        });
    }
}

fn linear_missile_collision(
    mut commands: Commands,
    mut q_missile: Query<(
        Entity,
        &Missile,
        &MissileState,
        &mut LinearMissile,
        &mut Transform,
    )>,
    q_targets: Query<(Entity, &Team, &Transform), (Without<LinearMissile>, Without<Death>)>,
    q_source_team: Query<&Team>,
    res_grid: Option<Res<ResourceGrid>>,
    assets_grid: Option<Res<Assets<ConfigNavigationGrid>>>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (missile_entity, missile, state, mut linear, mut transform) in q_missile.iter_mut() {
        let Some(destination) = state.destination else {
            continue;
        };

        let current = transform.translation;
        let diff = destination - current;
        let distance = diff.length();

        if distance < 1.0 {
            commands.entity(missile_entity).despawn();
            continue;
        }

        let step = (missile.speed * dt).min(distance);
        let direction = diff / distance;
        transform.translation = current + direction * step;
        // 旋转矩形条面朝移动方向（Cuboid 沿 Z 轴伸长）
        transform.rotation = Quat::from_rotation_arc(Vec3::Z, direction);

        // 粘性飞弹：检测地形碰撞，碰墙即锚定，不做实体碰撞
        if linear.sticky {
            if let (Some(res_grid), Some(assets_grid)) = (res_grid.as_ref(), assets_grid.as_ref()) {
                if let Some(grid) = assets_grid.get(&res_grid.0) {
                    let cell = grid.get_cell_xy_by_position(&transform.translation.xz());
                    if !grid.is_walkable_by_xy(cell) {
                        let hit_point = transform.translation;
                        commands.entity(missile_entity).despawn();
                        commands.spawn((
                            WallAnchor {
                                timer: Timer::from_seconds(WALL_ANCHOR_DURATION, TimerMode::Once),
                            },
                            Transform::from_translation(hit_point),
                        ));
                        commands.entity(state.source).trigger(|e| EventMissileHit {
                            source: e,
                            spell: missile.key.clone(),
                            point: hit_point,
                        });
                        continue;
                    }
                }
            }
            // 仍可走：继续飞行，不做实体碰撞
            continue;
        }

        let Ok(source_team) = q_source_team.get(state.source) else {
            continue;
        };

        for (target, team, target_transform) in q_targets.iter() {
            if team == source_team {
                continue;
            }
            if linear.hit_enemies.contains(&target) {
                continue;
            }

            let to_target = target_transform.translation - transform.translation;
            let dist = to_target.length();
            if dist < linear.width {
                linear.hit_enemies.push(target);
                commands.trigger(CommandDamageCreate {
                    entity: target,
                    source: state.source,
                    damage_type: DamageType::Physical,
                    amount: linear.damage,
                    tag: None,
                });
            }
        }
    }
}

/// 墙锚点生命周期：到期销毁
fn update_wall_anchors(
    mut commands: Commands,
    mut q: Query<(Entity, &mut WallAnchor)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut anchor) in q.iter_mut() {
        anchor.timer.tick(time.delta());
        if anchor.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn on_command_missile_create(
    trigger: On<CommandMissileCreate>,
    mut commands: Commands,
    res_assets_spell_object: Res<Assets<Spell>>,
    q_global_transform: Query<&GlobalTransform>,
    q_children: Query<&Children>,
    q_joint_target: Query<&Name>,
    q_team: Query<&Team>,
) {
    let entity = trigger.event_target();
    let opt_team = q_team.get(entity).ok().cloned();

    let spell_data = res_assets_spell_object
        .get(&trigger.spell)
        .and_then(|s| s.spell_data.as_ref());

    let speed = trigger
        .speed
        .or_else(|| spell_data.and_then(|d| d.missile_speed))
        .unwrap_or(1200.0);

    // 读取 missile spec 数据（start_bone, width, behaviors）
    let missile_spec = spell_data.and_then(|d| d.missile_spec.as_ref());
    let start_bone = missile_spec.and_then(|s| match &s.movement_component {
        MovementType::MovementTypeFixedSpeed(f) => f.start_bone_name.clone(),
    });
    let missile_width = missile_spec
        .and_then(|s| s.missile_width)
        .or(spell_data.and_then(|d| d.line_width));
    let _has_destroy_on_end = missile_spec
        .and_then(|s| {
            s.behaviors.as_ref().map(|b| {
                b.iter()
                    .any(|beh| matches!(beh, MissileBehavior::DestroyOnMovementComplete))
            })
        })
        .unwrap_or(true);

    let translation = match &start_bone {
        Some(bone_name) => find_bone_translation(
            entity,
            bone_name,
            &q_children,
            &q_joint_target,
            &q_global_transform,
        )
        .unwrap_or_else(|| {
            q_global_transform
                .get(entity)
                .map(|t| t.translation())
                .unwrap_or_default()
        }),
        None => q_global_transform
            .get(entity)
            .map(|t| t.translation())
            .unwrap_or_default(),
    };

    // 直线导弹
    if trigger.target.is_none() {
        let Some(destination) = trigger.destination else {
            return;
        };

        let mut cmd = commands.spawn((
            Missile {
                key: trigger.spell.clone(),
                speed,
            },
            MissileState {
                source: entity,
                target: None,
                target_bone: None,
                destination: Some(destination),
            },
            LinearMissile {
                width: missile_width.unwrap_or(100.0),
                damage: trigger.damage,
                hit_enemies: Vec::new(),
                sticky: trigger.sticky,
            },
            Transform::from_translation(translation),
            Movement { speed },
        ));

        if let Some(team) = opt_team.clone() {
            cmd.insert(team);
        }

        let missile_entity = cmd.id();

        commands.trigger(CommandMovement {
            entity: missile_entity,
            priority: 0,
            action: MovementAction::Start {
                way: MovementWay::Path(vec![destination]),
                speed: Some(speed),
                source: MovementSource::Missile,
            },
        });

        // 用矩形条显示导弹的碰撞宽度
        commands.entity(missile_entity).insert(DebugMissile {
            width: missile_width.unwrap_or(100.0),
            length: 100.0,
            color: Color::srgba(1.0, 0.3, 0.3, 0.6),
        });

        return;
    }

    // 追踪导弹
    let target = trigger.target.unwrap();

    let Ok(target_translation) = q_global_transform.get(target).map(|v| v.translation()) else {
        return;
    };

    let mut end_entity = target;
    if let Some(hit_bone_name) = spell_data.and_then(|d| d.hit_bone_name.clone()) {
        for child in q_children.iter_descendants(target) {
            let Ok(joint_target) = q_joint_target.get(child) else {
                continue;
            };
            if joint_target.to_string() == hit_bone_name {
                end_entity = child;
                break;
            }
        }
    }

    debug!("{} 发射导弹 {:?}", entity, trigger.spell);
    let mut cmd = commands.spawn((
        Missile {
            key: trigger.spell.clone(),
            speed,
        },
        MissileState {
            source: entity,
            target: Some(target),
            target_bone: Some(end_entity),
            destination: None,
        },
        Transform::from_translation(translation),
        Movement { speed },
    ));

    if let Some(team) = opt_team.clone() {
        cmd.insert(team);
    }

    let missile_entity = cmd.id();

    q_children.iter_descendants(entity);

    commands.trigger(CommandMovement {
        entity: missile_entity,
        priority: 0,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![target_translation]),
            speed: Some(speed),
            source: MovementSource::Missile,
        },
    });
    commands.entity(missile_entity).insert(DebugSphere {
        color: RED_500.into(),
        radius: 10.0,
    });
}

fn on_event_movement_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_missile: Query<&MissileState>,
    q_linear: Query<&LinearMissile>,
    q_damage: Query<&Damage>,
) {
    // 直线导弹到达终点：直接销毁（碰撞伤害已在 linear_missile_collision 中处理）
    if q_linear.get(trigger.entity).is_ok() {
        commands.entity(trigger.entity).despawn();
        return;
    }

    // 追踪导弹
    let Ok(state) = q_missile.get(trigger.entity) else {
        return;
    };

    commands.entity(trigger.entity).despawn();

    let Some(target) = state.target else {
        return;
    };

    if let Ok(damage) = q_damage.get(state.source) {
        debug!("{} 对 {} 造成伤害 {}", state.source, target, damage.0);
        commands.try_trigger(CommandDamageCreate {
            entity: target,
            source: state.source,
            damage_type: DamageType::Physical,
            amount: damage.0,
            tag: None,
        });
    }
}

fn on_command_attached_field_create(
    trigger: On<CommandAttachedFieldCreate>,
    mut commands: Commands,
) {
    let entity = trigger.event_target();
    let radius_end = trigger.radius;
    let radius_start = trigger.grow_from.unwrap_or(radius_end);
    let grow_duration = trigger.grow_duration.unwrap_or(trigger.duration);
    let field = commands
        .spawn((
            AttachedField {
                radius: radius_start,
                radius_start,
                radius_end,
                grow_duration,
                damage_amount: trigger.damage,
                hit_enemies: Vec::new(),
                timer: Timer::from_seconds(trigger.duration, TimerMode::Once),
            },
            Transform::default(),
            DebugArea {
                color: Color::srgba(0.3, 0.6, 1.0, 0.25),
            },
        ))
        .id();
    commands.entity(entity).add_child(field);
}

/// 每帧检查附着伤害场，对范围内的敌人造成伤害（每个敌人只一次）
/// 同时插值半径（从 radius_start 到 radius_end）并更新可视化圆盘大小
fn update_attached_fields(
    mut commands: Commands,
    mut q_fields: Query<(Entity, &mut AttachedField, &ChildOf, &mut Transform)>,
    q_parent_transform: Query<&Transform, Without<AttachedField>>,
    q_enemies: Query<(Entity, &Team, &Transform), (Without<AttachedField>, Without<Death>)>,
    q_parent_team: Query<&Team, Without<AttachedField>>,
    time: Res<Time<Fixed>>,
) {
    for (field_entity, mut field, child_of, mut transform) in q_fields.iter_mut() {
        field.timer.tick(time.delta());
        if field.timer.is_finished() {
            commands.entity(field_entity).despawn();
            continue;
        }

        // 插值半径：在 grow_duration 内从 radius_start 增长到 radius_end
        // grow_duration 到达后保持最大半径直到 field 销毁
        let grow_progress = if field.grow_duration > 0.0 {
            (field.timer.elapsed_secs() / field.grow_duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        field.radius = field.radius_start + (field.radius_end - field.radius_start) * grow_progress;

        // 缩放可视化圆盘（DebugArea mesh 是单位圆盘 radius=1，用 scale 控制可见大小）
        transform.scale = Vec3::new(field.radius, 1.0, field.radius);

        let parent_entity = child_of.0;
        let Ok(parent_transform) = q_parent_transform.get(parent_entity) else {
            continue;
        };
        let field_pos = parent_transform.translation;

        let Ok(team) = q_parent_team.get(parent_entity) else {
            continue;
        };

        for (enemy, enemy_team, enemy_transform) in q_enemies.iter() {
            if *enemy_team == *team {
                continue;
            }
            if field.hit_enemies.contains(&enemy) {
                continue;
            }

            let dist = enemy_transform.translation.distance(field_pos);
            if dist <= field.radius {
                field.hit_enemies.push(enemy);
                commands.entity(enemy).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source: parent_entity,
                    damage_type: DamageType::Physical,
                    amount: field.damage_amount,
                    tag: None,
                });
            }
        }
    }
}

fn find_bone_translation(
    entity: Entity,
    bone_name: &str,
    q_children: &Query<&Children>,
    q_joint_target: &Query<&Name>,
    q_global_transform: &Query<&GlobalTransform>,
) -> Option<Vec3> {
    for child in q_children.iter_descendants(entity) {
        let Ok(joint_target) = q_joint_target.get(child) else {
            continue;
        };
        if joint_target.to_string() == bone_name {
            return Some(q_global_transform.get(child).unwrap().translation());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use bevy::time::TimeUpdateStrategy;
    use lol_base::grid::{ConfigNavigationGrid, ConfigNavigationGridCell};
    use lol_base::hash_key::HashKey;

    use super::*;
    use crate::movement::PluginMovement;
    use crate::navigation::grid::ResourceGrid;
    use crate::navigation::navigation::PluginNavigaton;
    use crate::team::Team;

    const MISSILE_SPELL_KEY: u32 = 0x7001;

    fn spell_handle() -> Handle<Spell> {
        Handle::from(HashKey::<Spell>::from(MISSILE_SPELL_KEY))
    }

    #[derive(Resource, Default)]
    struct HitTrace(Vec<Vec3>);

    fn on_missile_hit(trigger: On<EventMissileHit>, mut trace: ResMut<HitTrace>) {
        trace.0.push(trigger.point);
    }

    /// 2x2 可走网格（cell_size 100，min 0,0），可走区域 x,y ∈ [0,200]；超出即视为墙体。
    fn app_with_grid() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginMissile);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
        app.init_asset::<Spell>();
        app.init_resource::<HitTrace>();
        app.add_observer(on_missile_hit);
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));

        let grid = ConfigNavigationGrid {
            min_position: Vec2::ZERO,
            cell_size: 100.0,
            x_len: 2,
            y_len: 2,
            cells: vec![
                vec![ConfigNavigationGridCell::default(); 2],
                vec![ConfigNavigationGridCell::default(); 2],
            ],
            ..Default::default()
        };
        let grid_handle = app
            .world_mut()
            .resource_mut::<Assets<ConfigNavigationGrid>>()
            .add(grid);
        app.insert_resource(ResourceGrid(grid_handle));
        app
    }

    fn count<T: Component>(app: &mut App) -> usize {
        let mut q = app.world_mut().query::<&T>();
        q.iter(app.world()).count()
    }

    #[test]
    fn sticky_missile_anchors_at_wall() {
        let mut app = app_with_grid();
        let spell = spell_handle();
        let caster = app
            .world_mut()
            .spawn((Team::Order, Transform::from_xyz(100.0, 0.0, 100.0)))
            .id();

        app.world_mut()
            .entity_mut(caster)
            .trigger(|e| CommandMissileCreate {
                entity: e,
                target: None,
                destination: Some(Vec3::new(1000.0, 0.0, 100.0)),
                spell,
                damage: 0.0,
                speed: Some(1200.0),
                particle_hash: None,
                sticky: true,
            });
        for _ in 0..15 {
            app.update();
        }

        let trace = app.world().resource::<HitTrace>();
        assert_eq!(
            trace.0.len(),
            1,
            "粘性飞弹碰墙应发一次 EventMissileHit，实际 {:?}",
            trace.0
        );
        let x = trace.0[0].x;
        assert!(
            (190.0..=300.0).contains(&x),
            "锚点应在可走区边界 x≈200 附近，实际 x = {x}"
        );
        assert_eq!(count::<LinearMissile>(&mut app), 0, "粘性飞弹应已销毁");
        assert_eq!(count::<WallAnchor>(&mut app), 1, "应生成一个 WallAnchor");
    }

    #[test]
    fn non_sticky_missile_ignores_wall() {
        let mut app = app_with_grid();
        let spell = spell_handle();
        let caster = app
            .world_mut()
            .spawn((Team::Order, Transform::from_xyz(100.0, 0.0, 100.0)))
            .id();

        app.world_mut()
            .entity_mut(caster)
            .trigger(|e| CommandMissileCreate {
                entity: e,
                target: None,
                destination: Some(Vec3::new(1000.0, 0.0, 100.0)),
                spell,
                damage: 0.0,
                speed: Some(1200.0),
                particle_hash: None,
                sticky: false,
            });
        for _ in 0..5 {
            app.update();
        }

        let trace = app.world().resource::<HitTrace>();
        assert!(
            trace.0.is_empty(),
            "非粘性飞弹不做地形检测，不应发 EventMissileHit，实际 {:?}",
            trace.0
        );
        assert_eq!(count::<WallAnchor>(&mut app), 0, "不应生成 WallAnchor");
    }
}
