use super::{Movement, MovementDestination};
use crate::render::{MAP_HEIGHT, MAP_WIDTH};
use crate::{system_debug, system_info, system_warn};
use bevy::{app::App, math::vec2, prelude::*};
use vleue_navigator::{
    prelude::{ObstacleSource, PrimitiveObstacle},
    NavMesh,
};

#[derive(Component, Default)]
pub struct Navigator;

#[derive(Component, Default)]
pub struct Obstacle;

#[derive(Resource)]
pub struct GlobalNavMesh(NavMesh);

pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedPreUpdate, update);
    }
}

fn setup(
    mut commands: Commands,
    cachable_obstacles: Query<
        (&GlobalTransform, &PrimitiveObstacle),
        (With<Obstacle>, Without<Movement>),
    >,
) {
    let obstacle_count = cachable_obstacles.iter().count();
    system_info!(
        "navigation_setup",
        "Setting up navigation mesh with {} obstacles",
        obstacle_count
    );

    let polygons: Vec<_> = cachable_obstacles
        .iter()
        .flat_map(|(global_transform, obstacle)| {
            obstacle.get_polygons(
                global_transform,
                &Transform::default(),
                (global_transform.forward(), 0.0),
            )
        })
        .collect();

    system_debug!(
        "navigation_setup",
        "Generated {} polygons from obstacles",
        polygons.len()
    );

    let navmesh = NavMesh::from_edge_and_obstacles(
        vec![
            vec2(0.0, 0.0),
            vec2(MAP_WIDTH, 0.0),
            vec2(MAP_WIDTH, MAP_HEIGHT),
            vec2(0.0, MAP_HEIGHT),
        ],
        polygons,
    );

    system_info!(
        "navigation_setup",
        "Navigation mesh created successfully with bounds {}x{}",
        MAP_WIDTH,
        MAP_HEIGHT
    );
    commands.insert_resource(GlobalNavMesh(navmesh));
}

fn update(
    mut commands: Commands,
    mut query_navigator: Query<(Entity, &MovementDestination, &mut Transform), With<Navigator>>,
    navmeshes: Res<GlobalNavMesh>,
) {
    let navmesh = &navmeshes.0;
    let navigator_count = query_navigator.iter().count();

    if navigator_count > 0 {
        system_debug!(
            "update_navigator",
            "Updating navigation for {} entities",
            navigator_count
        );
    }

    let mut path_found_count = 0;
    let mut path_failed_count = 0;

    for (entity, movement_destination, transform) in query_navigator.iter_mut() {
        let target = movement_destination.0;
        let start = transform.translation.xz();

        if start == target {
            continue;
        }

        system_debug!(
            "update_navigator",
            "Finding path from ({:.1}, {:.1}) to ({:.1}, {:.1})",
            start.x,
            start.y,
            target.x,
            target.y
        );

        let Some(path) = navmesh.path(start, target) else {
            system_warn!(
                "update_navigator",
                "Failed to find path from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                start.x,
                start.y,
                target.x,
                target.y
            );
            path_failed_count += 1;
            continue;
        };

        let path = path.path;
        if !path.is_empty() {
            let next_waypoint = path[0];
            system_debug!(
                "update_navigator",
                "Path found with {} waypoints, next waypoint: ({:.1}, {:.1})",
                path.len(),
                next_waypoint.x,
                next_waypoint.y
            );
            commands
                .entity(entity)
                .insert(MovementDestination(next_waypoint));
            path_found_count += 1;
        }
    }

    if path_found_count > 0 || path_failed_count > 0 {
        system_info!(
            "update_navigator",
            "Navigation update complete: {} paths found, {} paths failed",
            path_found_count,
            path_failed_count
        );
    }
}
