use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};

#[derive(Component)]
pub struct CursorAnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub increment: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl CursorAnimationConfig {
    pub fn new(first: usize, last: usize, increment: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            increment,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

pub struct PluginCursor;

impl Plugin for PluginCursor {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_setup_cursor);
        // app.add_systems(Update, update_cursor_animation);
    }
}

fn startup_setup_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Single<Entity, With<Window>>,
) {
    commands
        .entity(*window)
        .insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: asset_server.load("ASSETS/UX/Cursors/hand1.tga"),
            texture_atlas: None,
            flip_x: false,
            flip_y: false,
            rect: None,
            hotspot: (10, 10),
        })));
}

// fn update_cursor_animation(
//     time: Res<Time>,
//     mut query: Query<(&mut CursorAnimationConfig, &mut CursorIcon)>,
// ) {
//     for (mut config, mut cursor_icon) in &mut query {
//         let CursorIcon::Custom(CustomCursor::Image(ref mut image)) = *cursor_icon else {
//             continue;
//         };

//         config.frame_timer.tick(time.delta());

//         let Some(atlas) = image.texture_atlas.as_mut() else {
//             continue;
//         };

//         if config.frame_timer.is_finished() {
//             atlas.index += config.increment;

//             if atlas.index > config.last_sprite_index {
//                 atlas.index = config.first_sprite_index;
//             }

//             config.frame_timer = CursorAnimationConfig::timer_from_fps(config.fps);
//         }
//     }
// }
