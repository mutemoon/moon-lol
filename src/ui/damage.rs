use bevy::color::palettes::css::{BLUE, RED, WHITE};
use bevy::prelude::*;

use crate::{DamageType, EventDamageCreate};

#[derive(Default)]
pub struct PluginUIDamage;

impl Plugin for PluginUIDamage {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_damage_numbers);
        app.add_observer(on_event_damage_create);
    }
}

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
    /// 最终字体大小
    pub final_scale: f32,
}

/// 监听伤害事件并创建伤害数字
fn on_event_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    global_transform: Query<&GlobalTransform>,
) {
    let target_entity = trigger.event_target();
    let damage_result = &trigger.damage_result;

    // 只显示实际造成的伤害
    if damage_result.final_damage <= 0.0 {
        return;
    }

    // 获取目标实体的位置
    let Ok(target_transform) = global_transform.get(target_entity) else {
        return;
    };

    let world_position = target_transform.translation();

    // 创建伤害数字UI
    commands.spawn((
        Text::new(format!("{:.0}", damage_result.final_damage)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::Srgba(match trigger.damage_type {
            DamageType::Physical => RED,
            DamageType::Magic => BLUE,
            DamageType::True => WHITE,
        })),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        DamageNumber {
            damage: damage_result.final_damage,
            lifetime: 0.0,
            max_lifetime: 1.0, // 2秒生存时间
            start_position: world_position,
            velocity_y: 250.0, // 初始向上速度
            gravity: -200.0,   // 重力加速度
            final_scale: 0.5,
        },
    ));
}

/// 更新伤害数字的动画效果
fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    camera_info: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut damage_numbers: Query<(
        Entity,
        &mut UiTransform,
        &mut DamageNumber,
        &mut Node,
        &mut TextColor,
    )>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();
    let delta_time = time.delta_secs();

    for (i, (entity, mut transform, mut damage_number, mut node, mut text_color)) in
        damage_numbers.iter_mut().enumerate()
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
            node.top = Val::Px(viewport_position.y + i as f32 * 20.);
        }

        // // 字体大小动画：从大到小
        let current_font_size = 1. - (1. - damage_number.final_scale) * progress;

        transform.scale = Vec2::splat(current_font_size);

        // 透明度动画：逐渐消失
        let alpha = 1.0 - progress;
        text_color.0 = text_color.0.with_alpha(alpha);
    }
}
