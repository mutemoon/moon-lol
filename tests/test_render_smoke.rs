use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use moon_lol::{PluginSkillTestRenderSuite, SkillTestRenderConfig};

#[test]
fn skill_test_render_writes_frames() {
    if std::env::var("MOON_LOL_RUN_RENDER_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping render smoke test: set MOON_LOL_RUN_RENDER_TESTS=1 to enable");
        return;
    }
    if skip_due_to_missing_gpu(skill_test_render_writes_frames_inner) {
        return;
    }
}

fn skill_test_render_writes_frames_inner() {
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
        video_output: None,
        keep_frame_images: false,
    });
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
    app.add_plugins(PluginSkillTestRenderSuite);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));

    app.finish();
    app.cleanup();

    for _ in 0..10 {
        app.update();
    }

    let frames_dir = output_dir.join("frames");
    let entries = fs::read_dir(&frames_dir)
        .unwrap_or_else(|e| panic!("failed to read output dir {frames_dir:?}: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect::<Vec<_>>();

    assert!(
        !entries.is_empty(),
        "expected rendered frames in {frames_dir:?}, found none"
    );
}

fn skip_due_to_missing_gpu(run: impl FnOnce()) -> bool {
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(()) => false,
        Err(payload) => {
            let message = if let Some(message) = payload.downcast_ref::<String>() {
                message.as_str()
            } else if let Some(message) = payload.downcast_ref::<&str>() {
                message
            } else {
                ""
            };

            if message.contains("Unable to find a GPU") {
                eprintln!("skipping render smoke test: no GPU available");
                true
            } else {
                std::panic::resume_unwind(payload);
            }
        }
    }
}
