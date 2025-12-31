mod animation;
mod button;
mod damage;
mod element;
mod player;
mod skill;

pub use animation::*;
use bevy::prelude::*;
pub use button::*;
pub use damage::*;
pub use element::*;
pub use player::*;
pub use skill::*;

use crate::{Bounding, Health};

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
                update_skill_level_up_button.run_if(in_state(UIState::Loaded)),
                (update_player_skill_icon, update_player_icon)
                    .run_if(in_state(UIState::Loaded).and(run_once)),
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
