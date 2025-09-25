use bevy::{color::palettes, prelude::*};

use crate::core::damage::EventDamageCreate;
use crate::core::{Bounding, Health};
use crate::{system_debug, system_info};

#[derive(Default)]
pub struct PluginUI;

impl Plugin for PluginUI {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (init_health_bar, ui, update_damage_numbers));
        app.add_observer(on_damage_create);
    }
}

#[derive(Component)]
pub struct UIBind {
    pub entity: Entity,
    pub position: Vec3,
    pub offset: Vec2,
}

#[derive(Component)]
pub struct HealthBind(pub Entity);

/// 伤害数字组件 - 用于显示飘动的伤害数字
#[derive(Component)]
pub struct DamageNumber {
    /// 伤害数值
    pub damage: f32,
    /// 生存时间（秒）
    pub lifetime: f32,
    /// 最大生存时间
    pub max_lifetime: f32,
    /// 初始位置
    pub start_position: Vec3,
    /// 垂直速度
    pub velocity_y: f32,
    /// 重力加速度
    pub gravity: f32,
    /// 初始字体大小
    pub initial_font_size: f32,
    /// 最终字体大小
    pub final_font_size: f32,
}

pub fn init_health_bar(
    mut commands: Commands,
    q_health: Query<Entity, Added<Health>>,
    q_bounding: Query<&Bounding>,
) {
    let health_bar_count = q_health.iter().count();
    if health_bar_count > 0 {
        system_info!(
            "init_health_bar",
            "Creating health bars for {} new entities",
            health_bar_count
        );
    }

    for entity in q_health.iter() {
        system_debug!(
            "init_health_bar",
            "Creating health bar UI for entity {:?}",
            entity
        );

        commands
            .spawn((
                Node {
                    width: Val::Px(75.0),
                    height: Val::Px(5.0),
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                UIBind {
                    entity,
                    position: Vec3::ZERO
                        .with_y(q_bounding.get(entity).map(|v| v.height).unwrap_or(0.0)),
                    offset: Vec2::ZERO,
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        left: Val::Percent(-50.0),
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },))
                    .with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                ..Default::default()
                            },
                            BackgroundColor(Color::Srgba(palettes::css::BLACK)),
                            BorderRadius::all(Val::Px(5.0)),
                        ));
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                ..Default::default()
                            },
                            HealthBind(entity),
                            BackgroundColor(Color::Srgba(palettes::tailwind::GREEN_300)),
                            BorderRadius::all(Val::Px(5.0)),
                        ));
                    });
            });
    }
}

pub fn ui(
    mut commands: Commands,
    camera_info: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    global_transform: Query<&GlobalTransform>,
    mut q_node: Query<&mut Node>,
    q_health_bind: Query<(Entity, &HealthBind)>,
    q_health: Query<&Health>,
    q_ui_bind: Query<(Entity, &UIBind)>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();
    for (entity, ui_bind) in q_ui_bind.iter() {
        let Ok(mut node) = q_node.get_mut(entity) else {
            continue;
        };
        let Ok(bind_target) = global_transform.get(ui_bind.entity) else {
            commands.entity(entity).despawn();
            continue;
        };
        let Ok(viewport_position) = camera.world_to_viewport(
            camera_global_transform,
            bind_target.translation() + ui_bind.position,
        ) else {
            continue;
        };
        let viewport_position = viewport_position + ui_bind.offset;
        node.left = Val::Px(viewport_position.x);
        node.top = Val::Px(viewport_position.y);
    }

    for (entity, health_bind) in q_health_bind.iter() {
        let Ok(mut node) = q_node.get_mut(entity) else {
            continue;
        };
        let Ok(health) = q_health.get(health_bind.0) else {
            continue;
        };
        node.width = Val::Percent(health.value / health.max * 100.0);
    }
}

/// 监听伤害事件并创建伤害数字
pub fn on_damage_create(
    trigger: Trigger<EventDamageCreate>,
    mut commands: Commands,
    global_transform: Query<&GlobalTransform>,
) {
    let target_entity = trigger.target();
    let damage_result = &trigger.event().damage_result;

    // 只显示实际造成的伤害
    if damage_result.final_damage <= 0.0 {
        return;
    }

    system_debug!(
        "on_damage_create",
        "Creating damage number for entity {:?}, damage: {:.1}",
        target_entity,
        damage_result.final_damage
    );

    // 获取目标实体的位置
    let Ok(target_transform) = global_transform.get(target_entity) else {
        system_debug!(
            "on_damage_create",
            "Failed to get transform for entity {:?}",
            target_entity
        );
        return;
    };

    let world_position = target_transform.translation();

    // 创建伤害数字UI
    commands.spawn((
        Text::new(format!("{:.0}", damage_result.final_damage)),
        TextFont {
            font_size: 24.0,
            ..Default::default()
        },
        TextColor(Color::Srgba(palettes::css::RED)),
        Node {
            position_type: PositionType::Absolute,
            ..Default::default()
        },
        DamageNumber {
            damage: damage_result.final_damage,
            lifetime: 0.0,
            max_lifetime: 2.0, // 2秒生存时间
            start_position: world_position,
            velocity_y: 100.0, // 初始向上速度
            gravity: -200.0,   // 重力加速度
            initial_font_size: 24.0,
            final_font_size: 12.0,
        },
        UIBind {
            entity: target_entity,
            position: Vec3::new(0.0, 0.0, 0.0),
            offset: Vec2::new(0.0, 0.0),
        },
    ));
}

/// 更新伤害数字的动画效果
pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    camera_info: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut damage_numbers: Query<(
        Entity,
        &mut DamageNumber,
        &mut Node,
        &mut TextFont,
        &mut TextColor,
    )>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();
    let delta_time = time.delta_secs();

    for (entity, mut damage_number, mut node, mut text_font, mut text_color) in
        damage_numbers.iter_mut()
    {
        // 更新生存时间
        damage_number.lifetime += delta_time;

        // 如果超过生存时间，销毁实体
        if damage_number.lifetime >= damage_number.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // 计算动画进度 (0.0 到 1.0)
        let progress = damage_number.lifetime / damage_number.max_lifetime;

        // 更新垂直速度（重力影响）
        damage_number.velocity_y += damage_number.gravity * delta_time;

        // 计算当前位置
        let current_y_offset = damage_number.velocity_y * damage_number.lifetime
            + 0.5 * damage_number.gravity * damage_number.lifetime * damage_number.lifetime;

        let current_world_pos =
            damage_number.start_position + Vec3::new(0.0, current_y_offset, 0.0);

        // 转换到屏幕坐标
        if let Ok(viewport_position) =
            camera.world_to_viewport(camera_global_transform, current_world_pos)
        {
            node.left = Val::Px(viewport_position.x - 20.0); // 居中偏移
            node.top = Val::Px(viewport_position.y);
        }

        // 字体大小动画：从大到小
        let current_font_size = damage_number.initial_font_size
            + (damage_number.final_font_size - damage_number.initial_font_size) * progress;
        text_font.font_size = current_font_size;

        // 透明度动画：逐渐消失
        let alpha = 1.0 - progress;
        text_color.0 = text_color.0.with_alpha(alpha);
    }
}
