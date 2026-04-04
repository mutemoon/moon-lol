use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use lol_core_render::test_render::{PluginSkillTestRenderSuite, SkillTestRenderConfig};

fn main() {
    let output_dir = PathBuf::from("artifacts/skill_test_render_smoke");
    let _ = std::fs::remove_dir_all(&output_dir);

    let mut app = App::new();
    app.insert_resource(SkillTestRenderConfig {
        output_dir,
        width: 640,
        height: 360,
        capture_every_nth_frame: 1,
        max_frames: Some(8),
        spawn_default_scene: true,
        video_output: None,
        keep_frame_images: false,
    });
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            visible: false,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(PluginSkillTestRenderSuite);
    app.finish();
    app.cleanup();

    for _ in 0..24 {
        app.update();
    }
}
