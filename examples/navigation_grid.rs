use bevy::prelude::*;
use lol_base::grid::ConfigNavigationGrid;
use lol_core::entities::barrack::PluginBarrack;
use lol_core::game::PluginGame;
use lol_core::navigation::grid::ResourceGrid;
use lol_render::camera::CameraState;
use lol_render::map::Map;
use moon_lol::PluginCore;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "navigation".to_string(),
                    resolution: (300, 300).into(),
                    position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore
                .build()
                .disable::<PluginBarrack>()
                .set(PluginGame {
                    scenes: vec!["games/attack.ron".to_string()],
                }),
        ))
        .add_systems(Update, on_key_space)
        .add_systems(Update, on_key_m)
        .run();
}

fn on_key_space(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_map: Query<&mut Visibility, With<Map>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut visibility in q_map.iter_mut() {
            visibility.toggle_visible_hidden();
        }
    }
}

fn on_key_m(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    res_grid: Option<Res<ResourceGrid>>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    mut camera: Query<&mut CameraState, With<Camera3d>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let Some(res_grid) = res_grid else {
            return;
        };
        let Some(grid) = assets_grid.get(&res_grid.0) else {
            return;
        };
        let center_pos = grid.get_map_center_position();

        if let Ok(mut camera_state) = camera.single_mut() {
            camera_state.position = center_pos;
            camera_state.scale = 10.0;
        }
    }
}
