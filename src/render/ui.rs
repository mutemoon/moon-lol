// UI-related functionality
use crate::combat::Health;
use crate::{system_debug, system_info};
use bevy::{color::palettes, prelude::*};

pub struct PluginUI;

impl Plugin for PluginUI {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (init_health_bar, ui));
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

pub fn init_health_bar(mut commands: Commands, q_health: Query<Entity, Added<Health>>) {
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
                    position: Vec3::ZERO,
                    offset: Vec2::new(0.0, -50.0),
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
