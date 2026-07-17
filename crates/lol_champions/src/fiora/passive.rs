use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use lol_core::base::buff::BuffOf;
use lol_core::base::direction::{Direction, is_in_direction};
use lol_core::buffs::common_buffs::{BuffMoveSpeed, BuffSelfHeal};
use lol_core::damage::{CommandDamageCreate, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::PassiveSkillOf;
use lol_core::team::Team;
use rand::random;
use serde::{Deserialize, Serialize};

use crate::fiora::Fiora;
use crate::fiora::r::BuffFioraR;

const VITAL_DISTANCE: f32 = 1000.0;
pub(crate) const FIORA_PASSIVE_ACTIVE_DURATION: f32 = 1.7;
pub(crate) const FIORA_PASSIVE_DURATION: f32 = 4.0;
const VITAL_TIMEOUT: f32 = 1.5;

// ── Vital 扇形视觉指示器参数 ──
/// 扇形半径（略大于英雄碰撞半径，使其在英雄脚下清晰可见）。
const VITAL_SECTOR_RADIUS: f32 = 110.0;
/// 扇形张角（与 `is_in_direction` 的 90° 象限判定一致）。
const VITAL_SECTOR_ANGLE_DEG: f32 = 90.0;
/// 扇形弧段数。
const VITAL_SECTOR_SEGMENTS: usize = 16;
/// 视觉离地高度，避免与地面 Z-fighting。
const VITAL_SECTOR_Y: f32 = 0.3;
/// 扇形颜色（剑姬要害的浅蓝色，半透明）。
const VITAL_SECTOR_COLOR: Color = Color::srgba(0.4, 0.85, 1.0, 0.45);

// ── 击破要害移速加成 ──
/// 被动击破要害后的移速加成比例（wiki：8%）。
pub(crate) const FIORA_PASSIVE_MS_PERCENT: f32 = 0.08;
/// 被动击破要害后移速加成持续时间（wiki：1.5s）。
pub(crate) const FIORA_PASSIVE_MS_DURATION: f32 = 1.5;
/// 被动击破要害后的治疗量。
///
/// wiki 仅说明「治疗菲奥娜」未给数值，ron `passive_heal_amount` 公式为空，
/// 此处为占位常量，待真实数值就绪后替换。
pub(crate) const FIORA_PASSIVE_HEAL: f32 = 40.0;

#[derive(Resource, Default)]
pub struct FioraVitalLastDirection {
    pub entity_to_last_direction: HashMap<Entity, Direction>,
}

#[derive(Component, Default)]
pub struct AbilityFioraPassive;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Vital {
    pub direction: Direction,
    pub active_timer: Timer,
    pub remove_timer: Timer,
    pub timeout_red_triggered: bool,
}

impl Vital {
    pub fn new(direction: Direction, add_duration: f32, active_duration: f32) -> Self {
        Self {
            direction,
            active_timer: Timer::from_seconds(add_duration, TimerMode::Once),
            remove_timer: Timer::from_seconds(active_duration, TimerMode::Once),
            timeout_red_triggered: false,
        }
    }
}

impl Vital {
    pub fn is_active(&self) -> bool {
        self.active_timer.is_finished()
    }
}

pub fn update_add_vital(
    mut commands: Commands,
    q_target_without_vital: Query<(Entity, &Transform, &Team), (With<Champion>, Without<Vital>)>,
    q_skill_of_with_ability: Query<&PassiveSkillOf, With<AbilityFioraPassive>>,
    q_transform_team: Query<(&Transform, &Team)>,
    q_buff_fiora_r: Query<&BuffOf, With<BuffFioraR>>,
    mut last_direction: ResMut<FioraVitalLastDirection>,
) {
    for skill_of in q_skill_of_with_ability.iter() {
        let entity = skill_of.0;

        let Ok((transform, team)) = q_transform_team.get(entity) else {
            continue;
        };

        for (target_entity, target_transform, target_team) in q_target_without_vital.iter() {
            if target_entity == entity {
                continue;
            }

            if target_team == team {
                continue;
            }

            let distance = target_transform
                .translation
                .xz()
                .distance(transform.translation.xz());

            if distance > VITAL_DISTANCE {
                continue;
            }

            let mut found = false;
            for buff_of in q_buff_fiora_r.iter() {
                if buff_of.get() == target_entity {
                    found = true;
                }
            }
            if found {
                continue;
            }

            let direction = match last_direction.entity_to_last_direction.get(&target_entity) {
                Some(direction) => match direction {
                    Direction::Up | Direction::Right => {
                        if random::<bool>() {
                            Direction::Left
                        } else {
                            Direction::Down
                        }
                    }
                    Direction::Left | Direction::Down => {
                        if random::<bool>() {
                            Direction::Up
                        } else {
                            Direction::Right
                        }
                    }
                },
                None => {
                    if random::<bool>() {
                        Direction::Up
                    } else {
                        Direction::Left
                    }
                }
            };

            last_direction
                .entity_to_last_direction
                .insert(target_entity, direction.clone());

            commands.entity(target_entity).insert(Vital::new(
                direction.clone(),
                FIORA_PASSIVE_ACTIVE_DURATION,
                FIORA_PASSIVE_DURATION,
            ));
        }
    }
}

pub fn update_remove_vital(
    mut commands: Commands,
    mut q_target_with_vital: Query<
        (Entity, &Transform, &Team, &mut Vital),
        (With<Champion>, With<Vital>),
    >,
    q_skill_of_with_ability: Query<&PassiveSkillOf, With<AbilityFioraPassive>>,
    q_transform_team: Query<(&Transform, &Team)>,
    time: Res<Time<Fixed>>,
) {
    for skill_of in q_skill_of_with_ability.iter() {
        let entity = skill_of.0;

        let Ok((fiora_transform, fiora_team)) = q_transform_team.get(entity) else {
            continue;
        };

        for (target_entity, target_transform, target_team, mut vital) in
            q_target_with_vital.iter_mut()
        {
            if target_team == fiora_team {
                continue;
            }

            let distance = target_transform
                .translation
                .xz()
                .distance(fiora_transform.translation.xz());

            if distance > VITAL_DISTANCE {
                commands.entity(target_entity).remove::<Vital>();
                continue;
            }

            if !vital.is_active() {
                vital.active_timer.tick(time.delta());
                continue;
            }

            if !vital.timeout_red_triggered && vital.remove_timer.remaining_secs() <= VITAL_TIMEOUT
            {
                vital.timeout_red_triggered = true;
            }

            vital.remove_timer.tick(time.delta());

            if vital.remove_timer.is_finished() {
                commands.entity(target_entity).remove::<Vital>();
            }
        }
    }
}

/// 监听伤害事件并创建伤害数字
pub fn on_passive_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_target_with_vital: Query<(&Transform, &Team, &Health, &Vital)>,
    q_transform: Query<(&Transform, &Team)>,
    mut last_direction: ResMut<FioraVitalLastDirection>,
) {
    let target_entity = trigger.event_target();
    if q_fiora.get(trigger.source).is_err() {
        return; // 只有菲奥娜本人能击破要害
    }
    let Ok((transform, team)) = q_transform.get(trigger.source) else {
        return;
    };

    let Ok((target_transform, target_team, hp, vital)) = q_target_with_vital.get(target_entity)
    else {
        return;
    };

    if target_team == team {
        return;
    }

    if !vital.is_active() {
        return;
    }

    let source_position = transform.translation.xz();
    let target_position = target_transform.translation.xz();

    if !is_in_direction(source_position, target_position, &vital.direction) {
        return;
    }

    let distance = source_position.distance(target_position);

    if distance > VITAL_DISTANCE {
        return;
    }

    let direction = match last_direction.entity_to_last_direction.get(&target_entity) {
        Some(direction) => match direction {
            Direction::Up | Direction::Right => {
                if random::<bool>() {
                    Direction::Left
                } else {
                    Direction::Down
                }
            }
            Direction::Left | Direction::Down => {
                if random::<bool>() {
                    Direction::Up
                } else {
                    Direction::Right
                }
            }
        },
        None => {
            if random::<bool>() {
                Direction::Up
            } else {
                Direction::Left
            }
        }
    };

    last_direction
        .entity_to_last_direction
        .insert(target_entity, direction.clone());

    commands.entity(target_entity).try_insert(Vital::new(
        direction.clone(),
        FIORA_PASSIVE_ACTIVE_DURATION,
        FIORA_PASSIVE_DURATION,
    ));

    commands.trigger(CommandDamageCreate {
        entity: target_entity,
        source: trigger.source,
        damage_type: DamageType::True,
        amount: hp.max * 0.05,
        tag: None,
    });

    // 击破要害：治疗菲奥娜 + 8% 移速（1.5s），均走通用 buff 原语
    commands
        .entity(trigger.source)
        .with_related::<BuffOf>(BuffSelfHeal::new(FIORA_PASSIVE_HEAL));
    commands
        .entity(trigger.source)
        .with_related::<BuffOf>(BuffMoveSpeed::new(
            FIORA_PASSIVE_MS_PERCENT,
            FIORA_PASSIVE_MS_DURATION,
        ));
}

// ── Vital 扇形视觉指示器 ──
//
// 被动原本只有逻辑（在敌人身上挂 `Vital` 标记一个要害方向），没有任何视觉表现。
// 这里为每个 `Vital` 同步生成一个平铺在地面、指向要害方向的半透明扇形，
// 随 Vital 的出现 / 方向刷新 / 消失而创建 / 更新 / 回收。
//
// 视觉系统直接以 `Vital` 组件为唯一驱动源，与「Vital 如何被挂上」解耦：
// 无头模式下不创建任何 Mesh 资源（`Assets<Mesh>` 不可用），仅维护标记实体与
// Transform，因此视觉的生命周期与朝向仍可在无头测试中断言。

/// 一个 Vital 扇形视觉指示器，`target` 指向持有 `Vital` 的敌方英雄。
#[derive(Component, Clone)]
pub struct FioraVitalVisual {
    pub target: Entity,
}

/// Vital 朝向 → 扇形实体的 Y 轴旋转。
///
/// 扇形网格默认朝 +Z，按要害方向旋转到对应象限。方向语义与 `is_in_direction`
/// 一致：要害方向表示攻击者应从哪一侧接近，扇形即指向该侧。
fn vital_direction_rotation(direction: &Direction) -> Quat {
    use std::f32::consts::{FRAC_PI_2, PI};
    match direction {
        Direction::Up => Quat::IDENTITY,                      // +Z
        Direction::Right => Quat::from_rotation_y(FRAC_PI_2), // +X
        Direction::Down => Quat::from_rotation_y(PI),         // -Z
        Direction::Left => Quat::from_rotation_y(-FRAC_PI_2), // -X
    }
}

/// 构造一个平铺在地面（XZ 平面，法线 +Y）、朝 +Z 的扇形（饼形）网格。
fn build_vital_sector_mesh(radius: f32, angle_deg: f32, segments: usize) -> Mesh {
    let half = angle_deg.to_radians() / 2.0;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    // 中心顶点
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    // 弧上顶点，从 -half（左侧）到 +half（右侧）
    for i in 0..=segments {
        let t = -half + 2.0 * half * (i as f32 / segments as f32);
        positions.push([radius * t.sin(), 0.0, radius * t.cos()]);
        normals.push([0.0, 1.0, 0.0]);
    }
    // 三角扇：(center, next, current) 使正面朝上 (+Y)
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 3);
    for i in 0..segments {
        indices.extend_from_slice(&[0, (i + 2) as u32, (i + 1) as u32]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// 缓存的扇形 Mesh / 材质句柄，避免每次生成都重建资源。
/// （用两个 `Local` 而非结构体，避免在 `pub` 系统签名里暴露私有类型。）

/// 每 tick 将 Vital 与扇形视觉对账：缺则补、方向 / 位置变则更、Vital 消失则回收。
pub fn update_vital_visuals(
    mut commands: Commands,
    q_vital: Query<(Entity, &GlobalTransform, &Vital), With<Champion>>,
    mut q_visual: Query<(Entity, &FioraVitalVisual, &mut Transform)>,
    mut local_mesh: Local<Option<Handle<Mesh>>>,
    mut local_material: Local<Option<Handle<StandardMaterial>>>,
    opt_meshes: Option<ResMut<Assets<Mesh>>>,
    opt_materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    // 目标 → (世界坐标, 要害方向)
    let vital_map: HashMap<Entity, (Vec3, Direction)> = q_vital
        .iter()
        .map(|(e, t, v)| (e, (t.translation(), v.direction.clone())))
        .collect();

    // 更新已存在的视觉；目标已无 Vital 的视觉加入回收列表
    let mut existing_targets: HashSet<Entity> = HashSet::new();
    let mut to_despawn: Vec<(Entity, Entity)> = Vec::new();
    for (visual_entity, visual, mut transform) in q_visual.iter_mut() {
        if let Some((pos, direction)) = vital_map.get(&visual.target) {
            transform.translation = Vec3::new(pos.x, VITAL_SECTOR_Y, pos.z);
            transform.rotation = vital_direction_rotation(direction);
            existing_targets.insert(visual.target);
        } else {
            to_despawn.push((visual_entity, visual.target));
        }
    }
    for (entity, target) in to_despawn {
        commands.entity(entity).despawn();
        info!("已销毁目标 {:?} 的剑姬要害视觉", target);
    }

    // 仅在渲染模式（资源可用）下 lazily 创建并缓存 Mesh / 材质
    let (mesh_handle, material_handle) =
        if let (Some(mut meshes), Some(mut materials)) = (opt_meshes, opt_materials) {
            if local_mesh.is_none() {
                *local_mesh = Some(meshes.add(build_vital_sector_mesh(
                    VITAL_SECTOR_RADIUS,
                    VITAL_SECTOR_ANGLE_DEG,
                    VITAL_SECTOR_SEGMENTS,
                )));
            }
            if local_material.is_none() {
                *local_material = Some(materials.add(StandardMaterial {
                    base_color: VITAL_SECTOR_COLOR,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }));
            }
            (local_mesh.clone(), local_material.clone())
        } else {
            (None, None)
        };

    // 为尚无视觉的 Vital 生成指示器
    for (target, transform, vital) in q_vital.iter() {
        if existing_targets.contains(&target) {
            continue;
        }
        let pos = transform.translation();
        let mut cmd = commands.spawn((
            FioraVitalVisual { target },
            Transform {
                translation: Vec3::new(pos.x, VITAL_SECTOR_Y, pos.z),
                rotation: vital_direction_rotation(&vital.direction),
                scale: Vec3::ONE,
            },
            GlobalTransform::default(),
        ));
        if let (Some(mesh), Some(material)) = (mesh_handle.as_ref(), material_handle.as_ref()) {
            cmd.insert((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));
        }
        info!(
            "已生成目标 {:?} 的剑姬要害视觉，朝向 {:?}",
            target, vital.direction
        );
    }
}

/// 把 `AbilityFioraPassive` 标记挂到 Fiora 的被动技能实体上。
///
/// `update_add_vital` 通过 `With<AbilityFioraPassive>` 过滤被动技能实体，但该标记
/// 此前从未被挂上，导致 Vital 从不生成、被动形同未启用。这里在 Fiora 英雄的
/// 被动技能实体上补挂该标记（幂等），使被动真正生效。
pub fn attach_fiora_passive_ability(
    mut commands: Commands,
    q_passive: Query<(Entity, &PassiveSkillOf), Without<AbilityFioraPassive>>,
    q_fiora: Query<(), With<Fiora>>,
) {
    for (skill_entity, passive_of) in q_passive.iter() {
        if q_fiora.get(passive_of.0).is_ok() {
            commands.entity(skill_entity).insert(AbilityFioraPassive);
        }
    }
}
