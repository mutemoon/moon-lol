use bevy::color::palettes::css::{BLUE, GOLD, RED, WHITE};
use bevy::prelude::*;
use lol_core::base::gold::EventGoldGain;
use lol_core::base::level::EventExperienceGain;
use lol_core::damage::{DamageType, EventDamageCreate};
use rand::Rng;

use crate::camera::CameraState;
use crate::controller::SelfPlayer;

/// 浮动数字组件 - 通用飘字动画
#[derive(Component)]
pub struct FloatingNumber {
    /// 当前生存时间
    pub lifetime: f32,
    /// 最大生存时间（秒）
    pub max_lifetime: f32,
    /// 初始世界坐标
    pub start_position: Vec3,
    /// 当前 X 轴偏移
    pub current_x_offset: f32,
    /// 当前 Y 轴偏移
    pub current_y_offset: f32,
    /// 水平速度
    pub velocity_x: f32,
    /// 垂直速度
    pub velocity_y: f32,
    /// 重力加速度
    pub gravity: f32,
    /// 最终缩放比例
    pub final_scale: f32,
}

#[derive(Default)]
pub struct PluginUIFloatingNumber;

impl Plugin for PluginUIFloatingNumber {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_floating_numbers);
        app.add_observer(on_event_damage_create);
        app.add_observer(on_event_gold_gain);
        app.add_observer(on_event_experience_gain);
    }
}

/// 更新浮动数字的动画效果（位置、缩放、透明度）
fn update_floating_numbers(
    mut commands: Commands,
    time: Res<Time>,
    camera_info: Single<(&Camera, &GlobalTransform), With<CameraState>>,
    mut floating_numbers: Query<(
        Entity,
        &mut UiTransform,
        &mut FloatingNumber,
        &mut Node,
        &mut TextColor,
    )>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();
    let delta_time = time.delta_secs();

    for (entity, mut transform, mut number, mut node, mut text_color) in floating_numbers.iter_mut()
    {
        number.lifetime += delta_time;

        if number.lifetime >= number.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = number.lifetime / number.max_lifetime;

        // 重力影响垂直速度
        number.velocity_y += number.gravity * delta_time;
        number.current_x_offset += number.velocity_x * delta_time;
        number.current_y_offset += number.velocity_y * delta_time;

        let current_world_pos = number.start_position
            + Vec3::new(number.current_x_offset, number.current_y_offset, 0.0);

        // 转换到屏幕坐标
        if let Ok(viewport_position) =
            camera.world_to_viewport(camera_global_transform, current_world_pos)
        {
            node.left = Val::Px(viewport_position.x - 20.0);
            node.top = Val::Px(viewport_position.y);
        }

        // 缩放动画：从大到小
        let current_font_size = 1.0 - (1.0 - number.final_scale) * progress;
        transform.scale = Vec2::splat(current_font_size);

        // 透明度动画：逐渐消失
        let alpha = 1.0 - progress;
        text_color.0 = text_color.0.with_alpha(alpha);
    }
}

/// 监听伤害事件并创建伤害数字
///
/// 数字大小根据伤害值自适应：小伤害小字，大伤害大字
fn on_event_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    global_transform: Query<&GlobalTransform>,
    q_champion: Query<(), With<lol_core::entities::champion::Champion>>,
) {
    // 只显示英雄受到伤害的数字（过滤打到小兵等非英雄目标）
    let target_entity = trigger.event_target();
    if q_champion.get(target_entity).is_err() {
        return;
    }

    if trigger.damage_result.final_damage <= 0.0 {
        return;
    }

    let damage = trigger.damage_result.final_damage;

    let Ok(target_transform) = global_transform.get(target_entity) else {
        return;
    };

    let world_position = target_transform.translation();

    // 伤害越大字号越大（14px~36px），小兵小伤害小字，英雄技能大伤害大字
    let font_size = (14.0 + damage * 0.05).clamp(14.0, 36.0);

    commands.spawn((
        Text::new(format!("{:.0}", damage)),
        TextFont {
            font_size: FontSize::Px(font_size),
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
        FloatingNumber {
            lifetime: 0.0,
            max_lifetime: 1.0,
            start_position: world_position,
            current_x_offset: 0.0,
            current_y_offset: 0.0,
            velocity_x: rand::rng().random_range(-100.0..100.0),
            velocity_y: 500.0,
            gravity: -1500.0,
            final_scale: 0.5,
        },
    ));
}

/// 监听金币增益事件并创建金币飘字
///
/// 只在玩家（SelfPlayer）击杀时显示，位置在死者死亡位置
fn on_event_gold_gain(
    trigger: On<EventGoldGain>,
    mut commands: Commands,
    q_self_player: Query<(), With<SelfPlayer>>,
) {
    let killer_entity = trigger.event_target();

    // 只显示自己收获的金币
    if q_self_player.get(killer_entity).is_err() {
        return;
    }

    commands.spawn((
        Text::new(format!("+{:.0}", trigger.amount)),
        TextFont {
            font_size: FontSize::Px(33.0),
            ..default()
        },
        TextColor(Color::Srgba(GOLD)),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        FloatingNumber {
            lifetime: 0.0,
            max_lifetime: 1.2,
            start_position: trigger.world_position,
            current_x_offset: 0.0,
            current_y_offset: 0.0,
            velocity_x: rand::rng().random_range(-50.0..50.0),
            velocity_y: 400.0,
            gravity: -1200.0,
            final_scale: 0.6,
        },
    ));
}

/// 监听经验增益事件并创建经验飘字
fn on_event_experience_gain(
    trigger: On<EventExperienceGain>,
    mut commands: Commands,
    global_transform: Query<&GlobalTransform>,
) {
    let entity = trigger.event_target();
    let Ok(target_transform) = global_transform.get(entity) else {
        return;
    };

    commands.spawn((
        Text::new(format!("+{} EXP", trigger.amount)),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::Srgba(WHITE)),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        FloatingNumber {
            lifetime: 0.0,
            max_lifetime: 1.2,
            start_position: target_transform.translation(),
            current_x_offset: 0.0,
            current_y_offset: 0.0,
            velocity_x: rand::rng().random_range(-50.0..50.0),
            velocity_y: 350.0,
            gravity: -1000.0,
            final_scale: 0.6,
        },
    ));
}
