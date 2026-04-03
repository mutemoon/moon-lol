use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use moon_lol::{
    attach_skill_test_actor, Action, Fiora, PluginBarrack, PluginCore, PluginMap, PluginMinion,
    PluginResource, PluginSkillTestRender, PluginTurret, PluginUI, SkillTestRenderConfig,
    SkillTestScript, SkillTestScriptCursor, SkillTestScriptStep, SkillTestVideoFormat,
    SkillTestVideoOutput,
};

fn main() {
    let output_dir = PathBuf::from("artifacts/fiora_render_test");
    let _ = std::fs::remove_dir_all(&output_dir);

    let mut app = App::new();
    app.insert_resource(SkillTestRenderConfig {
        output_dir,
        width: 1280,
        height: 720,
        capture_every_nth_frame: 1,
        max_frames: Some(180),
        spawn_default_scene: false,
        video_output: Some(SkillTestVideoOutput {
            format: SkillTestVideoFormat::Mp4,
            fps: 60,
            file_name: "fiora_render_test.mp4".to_owned(),
        }),
        keep_frame_images: false,
    });
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        16,
    )));
    app.insert_resource(make_fiora_script());
    app.init_resource::<SkillTestScriptCursor>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            visible: false,
            ..default()
        }),
        ..default()
    }));
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

    for _ in 0..220 {
        app.update();
    }
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

fn make_fiora_script() -> SkillTestScript {
    SkillTestScript::new(vec![
        SkillTestScriptStep::set_skill_points(0, 4),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(0)),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(1)),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(2)),
        SkillTestScriptStep::action(0, Action::SkillLevelUp(3)),
        SkillTestScriptStep::action(
            15,
            Action::Skill {
                index: 0,
                point: Vec2::new(170.0, 0.0),
            },
        ),
        SkillTestScriptStep::action(
            45,
            Action::Skill {
                index: 1,
                point: Vec2::new(170.0, 0.0),
            },
        ),
        SkillTestScriptStep::action(
            80,
            Action::Skill {
                index: 2,
                point: Vec2::new(170.0, 0.0),
            },
        ),
        SkillTestScriptStep::action(
            115,
            Action::Skill {
                index: 0,
                point: Vec2::new(170.0, 0.0),
            },
        ),
        SkillTestScriptStep::action(
            145,
            Action::Skill {
                index: 3,
                point: Vec2::new(170.0, 0.0),
            },
        ),
    ])
}
