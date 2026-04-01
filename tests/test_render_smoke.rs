use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use moon_lol::{PluginSkillTestRenderSuite, SkillTestRenderConfig};

#[test]
fn skill_test_render_writes_frames() {
    let output_dir = PathBuf::from("artifacts/test_render_smoke");
    let _ = fs::remove_dir_all(&output_dir);

    let mut app = App::new();
    app.insert_resource(SkillTestRenderConfig {
        output_dir: output_dir.clone(),
        width: 320,
        height: 180,
        capture_every_nth_frame: 1,
        max_frames: Some(3),
        spawn_default_scene: true,
    });
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
    app.add_plugins(PluginSkillTestRenderSuite);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(16)));

    app.finish();
    app.cleanup();

    for _ in 0..10 {
        app.update();
    }

    let entries = fs::read_dir(&output_dir)
        .unwrap_or_else(|e| panic!("failed to read output dir {output_dir:?}: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect::<Vec<_>>();

    assert!(
        !entries.is_empty(),
        "expected rendered frames in {output_dir:?}, found none"
    );
}
