use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::ecs::schedule::common_conditions::run_once;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode, PollType,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems};
use crossbeam_channel::{Receiver, Sender};
use image::{ImageBuffer, Rgba};

#[derive(Resource, Clone)]
pub struct SkillTestRenderConfig {
    pub output_dir: PathBuf,
    pub width: u32,
    pub height: u32,
    pub capture_every_nth_frame: u32,
    pub max_frames: Option<u32>,
    pub spawn_default_scene: bool,
    pub video_output: Option<SkillTestVideoOutput>,
    pub keep_frame_images: bool,
}

impl Default for SkillTestRenderConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("artifacts/test_renders"),
            width: 1280,
            height: 720,
            capture_every_nth_frame: 1,
            max_frames: None,
            spawn_default_scene: true,
            video_output: None,
            keep_frame_images: false,
        }
    }
}

#[derive(Clone)]
pub struct SkillTestVideoOutput {
    pub format: SkillTestVideoFormat,
    pub fps: u32,
    pub file_name: String,
}

impl SkillTestVideoOutput {
    fn output_path(&self, output_dir: &std::path::Path) -> PathBuf {
        output_dir.join(&self.file_name)
    }
}

fn frames_dir(output_dir: &std::path::Path) -> PathBuf {
    output_dir.join("frames")
}

#[derive(Clone)]
pub enum SkillTestVideoFormat {
    Gif,
    Mp4,
}

#[derive(Resource, Default)]
struct CapturePostProcessState {
    attempted: bool,
}

#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

#[derive(Resource, Default)]
struct CapturedFrameCount(u32);

#[derive(Resource, Default)]
struct CapturedFrameWriteIndex(u32);

#[derive(Resource, Clone)]
struct CaptureImageSize {
    width: u32,
    height: u32,
}

#[derive(Resource, Deref, DerefMut, Default, Clone)]
struct ImageCopiers(pub Vec<ImageCopier>);

#[derive(Component, Clone)]
struct ImageCopier {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    src_image: Handle<Image>,
}

#[derive(Default)]
pub struct PluginSkillTestRender;

impl Plugin for PluginSkillTestRender {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<SkillTestRenderConfig>() {
            app.insert_resource(SkillTestRenderConfig::default());
        }

        let (sender, receiver) = crossbeam_channel::unbounded();

        app.insert_resource(MainWorldReceiver(receiver));
        app.init_resource::<CapturedFrameCount>();
        app.init_resource::<CapturedFrameWriteIndex>();
        app.init_resource::<CapturePostProcessState>();
        app.add_systems(
            Update,
            setup_skill_test_render
                .run_if(resource_exists::<RenderDevice>)
                .run_if(run_once),
        );
        app.add_systems(
            Last,
            (write_captured_frames, run_post_process_after_capture),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(RenderWorldSender(sender))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(
                Render,
                (
                    image_copy_pass.after(RenderSystems::Render),
                    receive_image_from_buffer.after(image_copy_pass),
                ),
            );
    }
}

impl ImageCopier {
    fn new(src_image: Handle<Image>, size: Extent3d, render_device: &RenderDevice) -> Self {
        let pixel_size = TextureFormat::Rgba8UnormSrgb
            .block_copy_size(None)
            .unwrap_or(4) as usize;
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row(size.width as usize * pixel_size);

        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("skill-test-render-buffer"),
            size: padded_bytes_per_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            enabled: Arc::new(AtomicBool::new(true)),
            src_image,
        }
    }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

fn setup_skill_test_render(
    mut commands: Commands,
    config: Res<SkillTestRenderConfig>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    render_device: Res<RenderDevice>,
    q_camera: Query<Entity, With<Camera>>,
) {
    let _ = fs::create_dir_all(&config.output_dir);
    let _ = fs::create_dir_all(frames_dir(&config.output_dir));
    commands.insert_resource(CaptureImageSize {
        width: config.width,
        height: config.height,
    });

    let size = Extent3d {
        width: config.width,
        height: config.height,
        ..default()
    };

    let mut render_target_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    render_target_image.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
    let render_target_image_handle = images.add(render_target_image);

    commands.spawn(ImageCopier::new(
        render_target_image_handle.clone(),
        size,
        &render_device,
    ));

    if let Some(camera) = q_camera.iter().next() {
        commands.entity(camera).insert((
            Camera::default(),
            RenderTarget::Image(render_target_image_handle.into()),
        ));
    } else {
        commands.spawn((
            Camera3d::default(),
            RenderTarget::Image(render_target_image_handle.into()),
            Transform::from_xyz(-6.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }

    if !config.spawn_default_scene {
        return;
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.8, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(8.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.20, 0.24),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::default(),
        Name::new("SkillTestPlatform"),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.25, 0.25),
            emissive: Color::srgb(0.2, 0.02, 0.02).into(),
            ..default()
        })),
        Transform::from_xyz(1.5, 0.25, 0.0),
        Name::new("SkillTestDummy"),
    ));
}

fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ImageCopiers(
        image_copiers.iter().cloned().collect::<Vec<_>>(),
    ));
}

pub fn image_copy_pass(
    image_copiers: Res<ImageCopiers>,
    gpu_images: Res<RenderAssets<bevy::render::texture::GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for image_copier in image_copiers.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let Some(src_image) = gpu_images.get(&image_copier.src_image) else {
            continue;
        };

        let block_dimensions = src_image.texture_descriptor.format.block_dimensions();
        let block_size = src_image
            .texture_descriptor
            .format
            .block_copy_size(None)
            .unwrap();
        let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
            (src_image.texture_descriptor.size.width as usize / block_dimensions.0 as usize)
                * block_size as usize,
        );

        let mut encoder =
            render_device.create_command_encoder(&CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(
            src_image.texture.as_image_copy(),
            TexelCopyBufferInfo {
                buffer: &image_copier.buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZero::<u32>::new(padded_bytes_per_row as u32)
                            .unwrap()
                            .into(),
                    ),
                    rows_per_image: None,
                },
            },
            src_image.texture_descriptor.size,
        );

        render_queue.submit(std::iter::once(encoder.finish()));
    }
}

fn receive_image_from_buffer(
    image_copiers: Res<ImageCopiers>,
    render_device: Res<RenderDevice>,
    sender: Res<RenderWorldSender>,
) {
    for image_copier in image_copiers.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);
        let (map_sender, map_receiver) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |result| {
            let _ = map_sender.send(result);
        });

        render_device.poll(PollType::wait_indefinitely()).unwrap();
        let Ok(Ok(())) = map_receiver.recv() else {
            continue;
        };

        let _ = sender.send(buffer_slice.get_mapped_range().to_vec());
        image_copier.buffer.unmap();
    }
}

fn write_captured_frames(
    receiver: Res<MainWorldReceiver>,
    config: Res<SkillTestRenderConfig>,
    image_size: Res<CaptureImageSize>,
    mut capture_count: ResMut<CapturedFrameCount>,
    mut write_index: ResMut<CapturedFrameWriteIndex>,
) {
    for raw in receiver.try_iter() {
        let current_frame = capture_count.0;
        capture_count.0 += 1;

        if let Some(max_frames) = config.max_frames {
            if write_index.0 >= max_frames {
                continue;
            }
        }

        if current_frame % config.capture_every_nth_frame != 0 {
            continue;
        }

        let image = rgba_from_padded_buffer(&raw, image_size.width, image_size.height);
        let frame_path =
            frames_dir(&config.output_dir).join(format!("frame_{:06}.png", write_index.0));

        let _ = image.save(frame_path);
        write_index.0 += 1;
    }
}

fn run_post_process_after_capture(
    config: Res<SkillTestRenderConfig>,
    write_index: Res<CapturedFrameWriteIndex>,
    mut state: ResMut<CapturePostProcessState>,
) {
    if state.attempted {
        return;
    }

    let Some(max_frames) = config.max_frames else {
        return;
    };

    if write_index.0 < max_frames {
        return;
    }

    state.attempted = true;

    let Some(video_output) = &config.video_output else {
        return;
    };

    if !ffmpeg_exists() {
        warn!(
            "ffmpeg not found; skipped video export for {}",
            config.output_dir.display()
        );
        return;
    }

    let output_path = video_output.output_path(&config.output_dir);
    let frame_input = format!(
        "{}/frame_%06d.png",
        frames_dir(&config.output_dir)
            .strip_prefix(&config.output_dir)
            .unwrap_or(frames_dir(&config.output_dir).as_path())
            .display()
    );
    let status = match video_output.format {
        SkillTestVideoFormat::Gif => Command::new("ffmpeg")
            .args([
                "-y",
                "-framerate",
                &video_output.fps.to_string(),
                "-i",
                &frame_input,
                &video_output.file_name,
            ])
            .current_dir(&config.output_dir)
            .status(),
        SkillTestVideoFormat::Mp4 => Command::new("ffmpeg")
            .args([
                "-y",
                "-framerate",
                &video_output.fps.to_string(),
                "-i",
                &frame_input,
                "-pix_fmt",
                "yuv420p",
                "-vf",
                "pad=ceil(iw/2)*2:ceil(ih/2)*2",
                &video_output.file_name,
            ])
            .current_dir(&config.output_dir)
            .status(),
    };

    match status {
        Ok(status) if status.success() => {
            if !config.keep_frame_images {
                let _ = fs::remove_dir_all(frames_dir(&config.output_dir));
            }
            info!("exported capture video to {}", output_path.display());
        }
        Ok(status) => {
            warn!(
                "ffmpeg exited with status {status}; skipped usable video output for {}",
                output_path.display()
            );
        }
        Err(error) => {
            warn!(
                "failed to execute ffmpeg for {}: {error}",
                output_path.display()
            );
        }
    }
}

fn ffmpeg_exists() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn rgba_from_padded_buffer(raw: &[u8], width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let pixel_size = TextureFormat::Rgba8UnormSrgb
        .block_copy_size(None)
        .unwrap_or(4) as usize;
    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(width as usize * pixel_size);

    let mut rgba = Vec::with_capacity(width as usize * height as usize * pixel_size);
    for row in 0..height as usize {
        let row_start = row * padded_bytes_per_row;
        let row_end = row_start + width as usize * pixel_size;
        rgba.extend_from_slice(&raw[row_start..row_end]);
    }

    ImageBuffer::from_raw(width, height, rgba).expect("valid RGBA buffer")
}
