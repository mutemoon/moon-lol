use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::image::TextureFormatPixelInfo;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode, PollType,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{Extract, Render, RenderApp, RenderSystems};
use bevy::time::TimeUpdateStrategy;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use crossbeam_channel::{Receiver, Sender};
use image::codecs::jpeg::JpegEncoder;
use lol_champions::fiora::passive::Vital;
use lol_core::action::{Action, CommandAction};
use lol_core::attack::AttackState;
use lol_core::life::Health;
use lol_render::camera::CameraInit;
use lol_render::controller::Controller;
use moon_lol::PluginCore;
use rocket::http::{ContentType, Method, Status};
use rocket::serde::json::Json;
use rocket::{State, get, launch, post, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct AttackTarget;

#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

#[derive(Resource, Deref, DerefMut, Default, Clone)]
struct ImageCopiers(pub Vec<ImageCopier>);

#[derive(Resource, Default, Debug)]
struct SceneController {
    width: u32,
    height: u32,
}

#[derive(Component, Deref, DerefMut)]
struct ImageToSave(Handle<Image>);

#[derive(Component, Clone)]
struct ImageCopier {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    src_image: Handle<Image>,
}

impl SceneController {
    pub fn new(width: u32, height: u32) -> SceneController {
        SceneController { width, height }
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    scene_controller: Res<SceneController>,
    render_device: Res<RenderDevice>,
    mut q_render_target: Query<&mut RenderTarget>,
) {
    let size = Extent3d {
        width: scene_controller.width,
        height: scene_controller.height,
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

    let cpu_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let cpu_image_handle = images.add(cpu_image);

    commands.spawn(ImageCopier::new(
        render_target_image_handle.clone(),
        size,
        &render_device,
    ));
    commands.spawn(ImageToSave(cpu_image_handle));

    let mut render_target = q_render_target.single_mut().unwrap();
    *render_target = RenderTarget::Image(render_target_image_handle.into());
}

pub struct PluginImageCopy;

impl Plugin for PluginImageCopy {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        let render_app = app
            .insert_resource(MainWorldReceiver(r))
            .sub_app_mut(RenderApp);

        render_app
            .insert_resource(RenderWorldSender(s))
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
    pub fn new(
        src_image: Handle<Image>,
        size: Extent3d,
        render_device: &RenderDevice,
    ) -> ImageCopier {
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: padded_bytes_per_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ImageCopier {
            buffer: cpu_buffer,
            src_image,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ImageCopiers(
        image_copiers.iter().cloned().collect::<Vec<_>>(),
    ));
}

// Removed ImageCopy and ImageCopyDriver as RenderGraph is replaced by systems.

fn image_copy_pass(
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
        info!("copy 结束");

        render_queue.submit(std::iter::once(encoder.finish()));
    }
}

fn receive_image_from_buffer(
    image_copiers: Res<ImageCopiers>,
    render_device: Res<RenderDevice>,
    sender: Res<RenderWorldSender>,
) {
    for image_copier in image_copiers.0.iter() {
        if !image_copier.enabled() {
            continue;
        }

        let buffer_slice = image_copier.buffer.slice(..);
        let (s, r) = crossbeam_channel::bounded(1);

        buffer_slice.map_async(MapMode::Read, move |res| match res {
            Ok(r) => s.send(r).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        render_device.poll(PollType::wait_indefinitely()).unwrap();
        r.recv().expect("Failed to receive the map_async message");

        let _ = sender.send(buffer_slice.get_mapped_range().to_vec());
        image_copier.buffer.unmap();
    }
}

struct BevyAppState {
    sender: Sender<AppMsg>,
    image: Arc<Mutex<Option<Vec<u8>>>>,
    observe: Arc<Mutex<Option<Observe>>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Observe {
    time: f32,
    myself: ObserveMyself,
    minions: ObserveMinion,
}

#[derive(Clone, Serialize, Deserialize)]
struct ObserveMinion {
    entity: Entity,
    position: Vec2,
    health: f32,
    vital: Vital,
}

#[derive(Clone, Serialize, Deserialize)]
struct ObserveMyself {
    position: Vec2,
    attack_state: Option<AttackState>,
}

enum AppMsg {
    Step(Option<Action>),
}

#[post("/step", data = "<action_json>")]
fn step(state: &State<BevyAppState>, action_json: Option<Json<Action>>) -> Status {
    let action = action_json.map(|v| v.into_inner());

    match state.sender.send(AppMsg::Step(action)) {
        Ok(_) => {
            info!("Action 已成功发送到 Bevy App。");
            Status::Accepted
        }
        Err(e) => {
            info!("发送 Action 到 Bevy App 失败: {}", e);
            Status::InternalServerError
        }
    }
}

#[get("/render")]
fn render(state: &State<BevyAppState>) -> Result<(ContentType, Vec<u8>), Status> {
    let image_option = state.image.lock().unwrap().clone();

    if let Some(image_data) = image_option {
        info!("找到图像 ({} 字节)，正在作为 JPEG 返回。", image_data.len());

        Ok((ContentType::JPEG, image_data))
    } else {
        info!("图像尚未准备好 (返回 404)。");

        Err(Status::NotFound)
    }
}

#[get("/observe")]
fn observe(state: &State<BevyAppState>) -> Result<Json<Observe>, Status> {
    let observe_option = state.observe.lock().unwrap().clone();

    if let Some(observe_data) = observe_option {
        Ok(Json(observe_data))
    } else {
        info!("[OBSERVE] Observe 数据尚未准备好 (返回 404)。");
        Err(Status::NotFound)
    }
}

#[launch]
fn rocket() -> _ {
    let (tx, rx) = crossbeam_channel::bounded::<AppMsg>(1);

    let shared_image = Arc::new(Mutex::new(None));
    let shared_observe = Arc::new(Mutex::new(None));

    let bevy_state = BevyAppState {
        sender: tx,
        image: shared_image.clone(),
        observe: shared_observe.clone(),
    };

    thread::spawn(move || {
        let mut app = App::new();

        let fixed_update_timestep = Time::<Fixed>::default().timestep();

        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(PluginCore.build())
        .add_plugins(PluginImageCopy)
        .insert_resource(TimeUpdateStrategy::ManualDuration(fixed_update_timestep))
        .insert_resource(SceneController::new(1920, 1080))
        .add_systems(Startup, setup.after(CameraInit));

        app.finish();
        app.cleanup();

        app.update();
        receive_and_encode_image_async(app.world_mut(), shared_image.clone());
        app.update();
        receive_and_encode_image_async(app.world_mut(), shared_image.clone());
        app.update();
        receive_and_encode_image_async(app.world_mut(), shared_image.clone());

        if let Some(observe) = get_observe(app.world_mut()) {
            *shared_observe.lock().unwrap() = Some(observe);
        };

        loop {
            let Ok(msg) = rx.recv() else {
                continue;
            };

            let AppMsg::Step(action) = msg;

            if let Some(action) = action {
                let world = app.world_mut();
                if let Ok((entity, _)) = world.query::<(Entity, &Controller)>().single(world) {
                    world.commands().trigger(CommandAction { entity, action });
                }
            }

            app.update();
            info!("update 结束");

            let world = app.world_mut();

            receive_and_encode_image_async(world, shared_image.clone());

            if let Some(observe) = get_observe(world) {
                *shared_observe.lock().unwrap() = Some(observe);
            };
        }
    });

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Options]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allowed_headers(rocket_cors::AllowedHeaders::all())
        .allow_credentials(true);

    rocket::build()
        .attach(cors.to_cors().expect("CORS fairing 创建失败"))
        .manage(bevy_state)
        .mount("/", routes![step, render, observe])
}

fn receive_and_encode_image_async(world: &mut World, shared_image: Arc<Mutex<Option<Vec<u8>>>>) {
    let receiver = world.resource::<MainWorldReceiver>();
    info!("尝试从 RenderWorld 接收图像...");

    let Ok(image_data) = receiver.try_recv() else {
        info!("本帧未收到图像数据。");
        return;
    };

    if image_data.is_empty() {
        info!("收到空的图像数据。");
        return;
    }

    info!(
        "收到 {} 字节的原始图像数据。正在派发到后台线程处理...",
        image_data.len()
    );

    let scene_controller = world.resource::<SceneController>();
    let width = scene_controller.width;
    let height = scene_controller.height;
    let format = TextureFormat::Rgba8UnormSrgb;

    thread::spawn(move || {
        info!("[后台线程] 开始处理图像...");

        let row_bytes = width as usize * format.pixel_size().unwrap();
        let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);

        let final_image_data = if row_bytes == aligned_row_bytes {
            image_data
        } else {
            info!("[后台线程] 正在去除图像行填充...");
            image_data
                .chunks(aligned_row_bytes)
                .take(height as usize)
                .flat_map(|row| &row[..row_bytes.min(row.len())])
                .cloned()
                .collect()
        };

        let Some(img) = image::RgbaImage::from_raw(width, height, final_image_data) else {
            warn!("[后台线程] 无法从原始数据创建 RgbaImage。");
            return;
        };

        let mut jpeg_bytes = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);

        info!("[后台线程] 正在将图像编码为 JPEG...");
        if encoder.encode_image(&img).is_err() {
            warn!("[后台线程] JPEG 编码失败");
            return;
        }
        info!("[后台线程] 编码结束。正在更新 Mutex...");

        *shared_image.lock().unwrap() = Some(jpeg_bytes);
        info!("[后台线程] 写入最新游戏图像。");
    });
}

fn get_observe(world: &mut World) -> Option<Observe> {
    let Ok((_entity, transform, attack_state, _controller)) = world
        .query::<(Entity, &Transform, Option<&AttackState>, &Controller)>()
        .single(world)
    else {
        return None;
    };

    let myself = ObserveMyself {
        position: transform.translation.xz(),
        attack_state: attack_state.cloned(),
    };

    let Ok((target_entity, target_transform, health, vital, _)) = world
        .query::<(Entity, &Transform, &Health, &Vital, &AttackTarget)>()
        .single(world)
    else {
        return None;
    };

    let minions = ObserveMinion {
        entity: target_entity,
        position: target_transform.translation.xz(),
        health: health.value,
        vital: vital.clone(),
    };

    let time = world.resource::<Time>().elapsed_secs();

    Some(Observe {
        time,
        myself,
        minions,
    })
}
