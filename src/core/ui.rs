mod animation;
mod button;
mod element;
mod player;
mod skill;

pub use animation::*;
use bevy::color::palettes::css::{BLUE, RED, WHITE};
use bevy::prelude::*;
pub use button::*;
pub use element::*;
pub use player::*;
pub use skill::*;

use crate::{Bounding, DamageType, EventDamageCreate, Health};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UIStartupSystems {
    SpawnUIElement,
}

#[derive(Default)]
pub struct PluginUI;

impl Plugin for PluginUI {
    fn build(&self, app: &mut App) {
        app.init_state::<UIState>();

        app.init_resource::<UIElementEntity>();
        app.init_resource::<UIButtonEntity>();
        app.init_resource::<SkillLevelUpButton>();

        app.add_systems(Startup, startup_load_ui);
        app.add_systems(Startup, startup_spawn_buttons);
        app.add_systems(
            Update,
            (
                update_spawn_ui_element.run_if(in_state(UIState::Loading)),
                init_health_bar,
                update_ui_bind,
                update_health,
                update_level,
                update_player_health,
                update_player_health_fade,
                update_damage_numbers,
                update_player_ability_resource,
                update_skill_icon,
                update_skill_level_up_button,
                update_player_icon.run_if(in_state(UIState::Loaded).and(run_once)),
                update_ui_animation,
                update_ui_element,
                update_on_add_ui_element,
                update_button,
            ),
        );

        app.add_observer(on_event_damage_create);
        app.add_observer(on_command_update_ui_element);
        app.add_observer(on_command_ui_animation_start);
        app.add_observer(on_command_spawn_button);
        app.add_observer(on_command_despawn_button);
    }
}

#[derive(EntityEvent, Debug)]
pub struct CommandUpdateUIElement {
    pub entity: Entity,
    pub size_type: SizeType,
    pub value: f32,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum SizeType {
    Width,
    Height,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum NodeType {
    Parent,
    Child,
}

#[derive(Component)]
pub struct UIBind {
    pub entity: Entity,
    pub position: Vec3,
    pub offset: Vec2,
}

#[derive(Component)]
pub struct HealthBind(pub Entity);

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
pub struct HealthBar {
    pub bar_type: HealthBarType,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HealthBarType {
    #[default]
    Minion,
    Champion,
    Turret,
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

fn init_health_bar(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    q_added_health_bar: Query<(Entity, &HealthBar, &Bounding), Added<Bounding>>,
) {
    for (entity, health_bar, bounding) in q_added_health_bar.iter() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UIBind {
                    entity,
                    position: Vec3::ZERO.with_y(bounding.height),
                    offset: Vec2::ZERO,
                },
            ))
            .with_children(|parent| match health_bar.bar_type {
                HealthBarType::Minion => {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(74.0),
                                height: Val::Px(7.0),
                                left: Val::Px(-37.0),
                                top: Val::Px(-20.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            ImageNode {
                                image: res_asset_server.load("spotlighthealthbaratlas.tex#srgb"),
                                rect: Some(Rect::new(2.0, 503.0, 68.0, 510.0)),
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((Node {
                                    width: Val::Px(72.0),
                                    height: Val::Px(5.0),
                                    left: Val::Px(1.0),
                                    top: Val::Px(1.0),
                                    ..default()
                                },))
                                .with_child((
                                    Node {
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    ImageNode {
                                        image: res_asset_server
                                            .load("spotlighthealthbaratlas.tex#srgb"),
                                        rect: Some(Rect::new(147.0, 4.0, 258.0, 15.0)),
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                    HealthBind(entity),
                                ));
                        });
                }
                HealthBarType::Champion => {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(136.0),
                                height: Val::Px(29.0),
                                left: Val::Px(-68.0),
                                top: Val::Px(-60.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            ImageNode {
                                image: res_asset_server.load("spotlighthealthbaratlas.tex#srgb"),
                                rect: Some(Rect::new(3.0, 2.0, 139.0, 31.0)),
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((Node {
                                    width: Val::Px(104.0),
                                    height: Val::Px(11.0),
                                    left: Val::Px(28.0),
                                    top: Val::Px(7.0),
                                    ..default()
                                },))
                                .with_child((
                                    Node {
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    ImageNode {
                                        image: res_asset_server
                                            .load("spotlighthealthbaratlas.tex#srgb"),
                                        rect: Some(Rect::new(147.0, 4.0, 258.0, 15.0)),
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                    HealthBind(entity),
                                ));
                        });
                }
                HealthBarType::Turret => {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(192.0),
                                height: Val::Px(44.0),
                                left: Val::Px(-96.0),
                                top: Val::Px(-22.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            ImageNode {
                                image: res_asset_server.load("spotlighthealthbaratlas.tex#srgb"),
                                rect: Some(Rect::new(6.0, 256.0, 198.0, 290.0)),
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((Node {
                                    width: Val::Px(168.0),
                                    height: Val::Px(13.0),
                                    left: Val::Px(12.0),
                                    top: Val::Px(7.0),
                                    ..default()
                                },))
                                .with_child((
                                    Node {
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    ImageNode {
                                        image: res_asset_server
                                            .load("spotlighthealthbaratlas.tex#srgb"),
                                        rect: Some(Rect::new(147.0, 4.0, 258.0, 15.0)),
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                        image_mode: NodeImageMode::Stretch,
                                        ..default()
                                    },
                                    HealthBind(entity),
                                ));
                        });
                }
            });
    }
}

fn update_ui_bind(
    mut commands: Commands,
    camera_info: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    q_global_transform: Query<&GlobalTransform>,
    mut q_ui_bind: Query<(Entity, &mut Node, &UIBind)>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();

    for (entity, mut node, ui_bind) in q_ui_bind.iter_mut() {
        let Ok(bind_target) = q_global_transform.get(ui_bind.entity) else {
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

        if viewport_position.x < 0.0 || viewport_position.y < 0.0 {
            commands.entity(entity).insert(Visibility::Hidden);
            continue;
        } else {
            commands.entity(entity).insert(Visibility::Visible);
            node.left = Val::Px(viewport_position.x);
            node.top = Val::Px(viewport_position.y);
        }
    }
}

fn update_health(
    mut commands: Commands,
    mut q_health_bind: Query<(Entity, &mut Node, &HealthBind)>,
    q_health: Query<(&Health, &HealthBar)>,
) {
    for (entity, mut node, health_bind) in q_health_bind.iter_mut() {
        let Ok((health, health_bar)) = q_health.get(health_bind.0) else {
            continue;
        };

        node.width = Val::Percent(health.value / health.max * 100.0);

        // 英雄血条需要生成每 100 点血的标记
        if health_bar.bar_type == HealthBarType::Champion {
            commands.entity(entity).despawn_children();
            commands.entity(entity).with_children(|parent| {
                let health_indicator_num = (health.value / 100.0) as usize;
                let health_bar_width = ((100.0 / health.max) * 104.0).floor();
                for i in 0..health_indicator_num {
                    parent.spawn((
                        Node {
                            width: Val::Px(1.0),
                            height: Val::Px(6.0),
                            left: Val::Px(health_bar_width * (i + 1) as f32),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::BLACK),
                    ));
                }
            });
        }
    }
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
        &mut Transform,
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

        transform.scale = Vec3::splat(current_font_size);

        // 透明度动画：逐渐消失
        let alpha = 1.0 - progress;
        text_color.0 = text_color.0.with_alpha(alpha);
    }
}

fn on_command_update_ui_element(
    trigger: On<CommandUpdateUIElement>,
    q_children: Query<&Children>,
    mut q_node: Query<&mut Node>,
) {
    let entity = trigger.entity;
    let size_type = trigger.size_type;
    let value = trigger.value;
    let node_type = trigger.node_type;

    let Ok(children) = q_children.get(entity) else {
        return;
    };

    let Ok(child_node) = q_node.get(children[0]) else {
        return;
    };

    let (target_entity, standard_size) = match node_type {
        NodeType::Parent => {
            let size = match size_type {
                SizeType::Width => {
                    if let Val::Px(width) = child_node.width {
                        width
                    } else {
                        return;
                    }
                }
                SizeType::Height => {
                    if let Val::Px(height) = child_node.height {
                        height
                    } else {
                        return;
                    }
                }
            };
            (entity, size)
        }
        NodeType::Child => {
            let Ok(parent_node) = q_node.get(entity) else {
                return;
            };
            let size = match size_type {
                SizeType::Width => {
                    if let Val::Px(width) = parent_node.width {
                        width
                    } else {
                        return;
                    }
                }
                SizeType::Height => {
                    if let Val::Px(height) = parent_node.height {
                        height
                    } else {
                        return;
                    }
                }
            };
            (children[0], size)
        }
    };

    let Ok(mut target_node) = q_node.get_mut(target_entity) else {
        return;
    };

    match size_type {
        SizeType::Width => target_node.width = Val::Px(standard_size * value),
        SizeType::Height => target_node.height = Val::Px(standard_size * value),
    }
}
