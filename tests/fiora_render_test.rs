use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use lol_core::action::Action;
use lol_core::entities::barrack::PluginBarrack;
use lol_core::entities::champions::fiora::Fiora;
use lol_core::entities::minion::PluginMinion;
use lol_core::entities::turret::PluginTurret;
use lol_core::map::PluginMap;
use lol_core::resource::PluginResource;
use lol_core_render::test_render::{
    attach_skill_test_actor, PluginSkillTestRender, SkillTestRenderConfig, SkillTestScript,
    SkillTestScriptStep, SkillTestVideoFormat, SkillTestVideoOutput,
};
use lol_core_render::ui::PluginUI;
use moon_lol::PluginCore;

#[test]
fn fiora_q_writes_video() {
    run_fiora_case(
        "fiora_q_writes_video",
        120,
        vec![SkillTestScriptStep::action(
            20,
            Action::Skill {
                index: 0,
                point: Vec2::new(170.0, 0.0),
            },
        )],
    );
}

#[test]
fn fiora_w_writes_video() {
    run_fiora_case(
        "fiora_w_writes_video",
        120,
        vec![SkillTestScriptStep::action(
            20,
            Action::Skill {
                index: 1,
                point: Vec2::new(170.0, 0.0),
            },
        )],
    );
}

#[test]
fn fiora_e_writes_video() {
    run_fiora_case(
        "fiora_e_writes_video",
        120,
        vec![SkillTestScriptStep::action(
            20,
            Action::Skill {
                index: 2,
                point: Vec2::new(170.0, 0.0),
            },
        )],
    );
}

#[test]
fn fiora_r_writes_video() {
    run_fiora_case(
        "fiora_r_writes_video",
        140,
        vec![SkillTestScriptStep::action(
            20,
            Action::Skill {
                index: 3,
                point: Vec2::new(170.0, 0.0),
            },
        )],
    );
}

fn run_fiora_case(test_name: &str, max_frames: u32, steps: Vec<SkillTestScriptStep>) {
    if std::env::var("MOON_LOL_RUN_RENDER_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping render test: set MOON_LOL_RUN_RENDER_TESTS=1 to enable");
        return;
    }
    if skip_due_to_missing_gpu(|| run_fiora_case_inner(test_name, max_frames, steps)) {
        return;
    }
}

fn run_fiora_case_inner(test_name: &str, max_frames: u32, mut steps: Vec<SkillTestScriptStep>) {
    let output_dir = PathBuf::from(format!("artifacts/tests/{test_name}"));
    let _ = fs::remove_dir_all(&output_dir);

    let mut app = App::new();
    app.insert_resource(SkillTestRenderConfig {
        output_dir: output_dir.clone(),
        width: 1280,
        height: 720,
        capture_every_nth_frame: 1,
        max_frames: Some(max_frames),
        spawn_default_scene: false,
        video_output: Some(SkillTestVideoOutput {
            format: SkillTestVideoFormat::Mp4,
            fps: 60,
            file_name: format!("{test_name}.mp4"),
        }),
        keep_frame_images: false,
    });
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    app.insert_resource(make_script(steps.as_mut_slice()));
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
    app.add_plugins(PluginSkillTestRender);
    app.add_plugins(
        PluginCore
            .build()
            .set(PluginResource {
                game_config_path: "games/fiora_render.ron".to_owned(),
            })
            .disable::<PluginBarrack>()
            .disable::<PluginMap>()
            .disable::<PluginMinion>()
            .disable::<PluginTurret>()
            .disable::<PluginUI>(),
    );
    app.add_systems(Startup, setup_stage);
    app.add_systems(Update, attach_skill_test_actor::<Fiora>);
    app.finish();
    app.cleanup();

    for _ in 0..(max_frames + 40) {
        app.update();
    }

    assert!(
        output_dir.join(format!("{test_name}.mp4")).is_file(),
        "expected capture video in {}",
        output_dir.display()
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
                eprintln!("skipping render test: no GPU available");
                true
            } else {
                std::panic::resume_unwind(payload);
            }
        }
    }
}

fn make_script(skill_steps: &mut [SkillTestScriptStep]) -> SkillTestScript {
    let mut steps = vec![
        SkillTestScriptStep::set_skill_points(0, 4),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(0)),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(1)),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(2)),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(3)),
    ];
    steps.extend_from_slice(skill_steps);
    SkillTestScript::new(steps)
}

fn setup_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.7, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(12.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.18, 0.22),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Name::new("RenderTestPlatform"),
    ));
}
