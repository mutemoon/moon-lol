use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use bevy::{
    image::TextureFormatPixelInfo,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, Maintain,
            MapMode, TexelCopyBufferInfo, TexelCopyBufferLayout, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    window::ExitCondition,
    winit::WinitPlugin,
};
use crossbeam_channel::{Receiver, Sender};
use image::codecs::jpeg::JpegEncoder;

use moon_lol::{
    abilities::PluginAbilities,
    core::{Action, CameraInit, CommandAction, Controller, PluginCore},
    entities::{PluginBarrack, PluginEntities},
    server::PluginGymEnv,
};

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
    mut q_camera: Query<&mut Camera>,
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
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
    render_target_image.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
    let render_target_image_handle = images.add(render_target_image);

    let cpu_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0; 4],
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
    let cpu_image_handle = images.add(cpu_image);

    commands.spawn(ImageCopier::new(
        render_target_image_handle.clone(),
        size,
        &render_device,
    ));
    commands.spawn(ImageToSave(cpu_image_handle));

    let render_target = RenderTarget::Image(render_target_image_handle.into());

    let mut camera = q_camera.single_mut().unwrap();

    camera.target = render_target;
}

pub struct PluginImageCopy;

impl Plugin for PluginImageCopy {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        let render_app = app
            .insert_resource(MainWorldReceiver(r))
            .sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyDriver);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            .insert_resource(RenderWorldSender(s))
            .add_systems(ExtractSchedule, image_copy_extract)
            .add_systems(Render, receive_image_from_buffer.after(RenderSet::Render));
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

#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopy;

#[derive(Default)]
struct ImageCopyDriver;

impl render_graph::Node for ImageCopyDriver {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let image_copiers = world.get_resource::<ImageCopiers>().unwrap();
        let gpu_images = world
            .get_resource::<RenderAssets<bevy::render::texture::GpuImage>>()
            .unwrap();

        for image_copier in image_copiers.iter() {
            if !image_copier.enabled() {
                continue;
            }

            let src_image = gpu_images.get(&image_copier.src_image).unwrap();
            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());

            let block_dimensions = src_image.texture_format.block_dimensions();
            let block_size = src_image.texture_format.block_copy_size(None).unwrap();

            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src_image.size.width as usize / block_dimensions.0 as usize) * block_size as usize,
            );

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
                src_image.size,
            );
            println!("[bevy app] copy 结束");

            let render_queue = world.get_resource::<RenderQueue>().unwrap();
            render_queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
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

        render_device.poll(Maintain::wait()).panic_on_timeout();
        r.recv().expect("Failed to receive the map_async message");

        let _ = sender.send(buffer_slice.get_mapped_range().to_vec());
        image_copier.buffer.unmap();
    }
}

use rocket::{
    get,
    http::{ContentType, Status},
    launch, post, routes,
    serde::json::Json,
    State,
};

struct BevyAppState {
    sender: Sender<AppMsg>,
    image: Arc<Mutex<Option<Vec<u8>>>>,
}

enum AppMsg {
    Step(Action),
}

#[post("/step", data = "<action_json>")]
fn step(state: &State<BevyAppState>, action_json: Json<Action>) -> Status {
    println!("[STEP] 请求 POST /step");

    let action: Action = action_json.into_inner();

    match state.sender.send(AppMsg::Step(action)) {
        Ok(_) => {
            println!("[STEP] Action 已成功发送到 Bevy App。");
            Status::Accepted
        }
        Err(e) => {
            eprintln!("[STEP] 发送 Action 到 Bevy App 失败: {}", e);
            Status::InternalServerError
        }
    }
}

#[get("/render")]
fn render(state: &State<BevyAppState>) -> (ContentType, Vec<u8>) {
    println!("[STEP] /render 请求已收到");

    // 锁定、解包并克隆 Option<Vec<u8>>
    let image_option = state.image.lock().unwrap().clone();

    // 使用 match 来处理 Option
    match image_option {
        Some(image_data) => {
            // 如果有图像数据...
            println!(
                "[STEP] 找到图像 ({} 字节)，正在作为 JPEG 返回。",
                image_data.len()
            );

            // ...将其包装在 content::Raw 中并指定 JPEG 类型，
            // 然后将这一切包装在 Some() 中返回。
            (ContentType::JPEG, image_data)
        }
        None => {
            // 如果没有图像数据...
            println!("[STEP] 图像尚未准备好 (返回 404)。");

            // ...返回 None，Rocket 会自动发送 404 Not Found。
            panic!()
        }
    }
    // 你原来的 "update 完成" 打印语句不再需要，因为函数已经返回了
}

#[launch]
fn rocket() -> _ {
    let (tx, rx) = crossbeam_channel::bounded::<AppMsg>(1);

    // 2. 创建共享的 image 数据
    let shared_image = Arc::new(Mutex::new(None));

    let bevy_state = BevyAppState {
        sender: tx,
        image: shared_image.clone(),
    };

    thread::spawn(move || {
        let mut app = App::new();

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
        .add_plugins(PluginCore)
        .add_plugins(PluginEntities.build().disable::<PluginBarrack>())
        .add_plugins(PluginAbilities)
        .add_plugins(PluginGymEnv)
        .add_plugins(PluginImageCopy)
        .insert_resource(SceneController::new(1920, 1080))
        .add_systems(Startup, setup.after(CameraInit));

        app.finish();
        app.cleanup();

        loop {
            match rx.recv() {
                Ok(msg) => match msg {
                    AppMsg::Step(action) => {
                        let world = app.world_mut();
                        if let Ok((entity, _)) =
                            world.query::<(Entity, &Controller)>().single(world)
                        {
                            world
                                .commands()
                                .entity(entity)
                                .trigger(CommandAction { action });
                        }

                        app.update();
                        println!("[bevy app] update 结束");
                        // 假设这是在你的循环或函数内部
                        let world = app.world_mut();

                        // --- 移植自 update_and_stream_image 的逻辑 ---

                        // ----- 块 1: 检查通道 (不可变借用) -----
                        // 我们将结果放入一个 Option 中。
                        // `receiver` 的不可变借用在此块结束时被释放。
                        let image_data: Option<Vec<u8>> = {
                            let receiver = world.resource::<MainWorldReceiver>();
                            info!("[Rocket Loop] 尝试从 RenderWorld 接收图像...");
                            // 使用 .ok() 将 Result 转换为 Option，非阻塞
                            receiver.recv().ok()
                        };

                        // 仅在成功接收到数据时才继续
                        if let Some(image_data) = image_data {
                            if image_data.is_empty() {
                                info!("[Rocket Loop] 收到空的图像数据。");
                                // 如果在循环中，你可能想用 continue;
                                return;
                            }

                            info!(
                                "[Rocket Loop] 收到 {} 字节的原始图像数据。正在处理...",
                                image_data.len()
                            );

                            // ----- 块 2: 获取要保存的图像 ID (第一个可变借用) -----
                            // 假设你只关心第一个实体 (根据你原来的 `break;`)
                            // `images_to_save_query` 的可变借用在此块结束时被释放。
                            let image_to_save_id = {
                                let mut images_to_save_query = world.query::<&ImageToSave>();
                                // 获取第一个匹配实体的 ID
                                images_to_save_query.iter(world).next().map(|img| img.id())
                            };

                            // 仅在找到要保存的实体时才继续
                            if let Some(image_id) = image_to_save_id {
                                // ----- 块 3: 修改图像资源 (第二个可变借用) -----
                                // 现在以前的借用都已释放，我们可以安全地获取 `Assets<Image>`。
                                let mut images = world.resource_mut::<Assets<Image>>();

                                // 你的原始逻辑中，这里可能会 panic 如果 ID 无效。
                                // 最好使用 get_mut 并处理 Option。
                                let Some(img_bytes) = images.get_mut(image_id) else {
                                    warn!("[Rocket Loop] 找不到 ImageToSave ID 对应的 Asset。");
                                    return; // 或 continue
                                };

                                // ... 你的图像处理逻辑 ...
                                let row_bytes = img_bytes.width() as usize
                                    * img_bytes.texture_descriptor.format.pixel_size();
                                let aligned_row_bytes =
                                    RenderDevice::align_copy_bytes_per_row(row_bytes);

                                if row_bytes == aligned_row_bytes {
                                    // 注意：image_data 在这里被移动 (move)
                                    img_bytes.data = Some(image_data);
                                } else {
                                    img_bytes.data = Some(
                image_data // `image_data` 在这里也被移动
                    .chunks(aligned_row_bytes)
                    .take(img_bytes.height() as usize)
                    .flat_map(|row| &row[..row_bytes.min(row.len())])
                    .cloned()
                    .collect(),
            );
                                }

                                let img = match img_bytes.clone().try_into_dynamic() {
                                    Ok(img) => img.to_rgba8(),
                                    Err(e) => {
                                        warn!("[Rocket Loop] 转换图像失败: {e:?}");
                                        return; // 或 continue
                                    }
                                };

                                let mut jpeg_bytes = Vec::new();
                                let mut encoder =
                                    JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);

                                info!("[Rocket Loop] 正在将图像编码为 JPEG...");

                                match encoder.encode_image(&img) {
                                    Ok(_) => {
                                        info!("[Rocket Loop] JPEG 编码成功 ({} 字节)。存入 shared_image...",jpeg_bytes.len());
                                        *shared_image.lock().unwrap() = Some(jpeg_bytes);
                                    }
                                    Err(e) => {
                                        warn!("[Rocket Loop] JPEG 编码失败: {e}");
                                    }
                                }
                                // `break;`不再需要，因为我们只处理了 `image_to_save_id`
                            } else {
                                info!("[Rocket Loop] 收到图像数据，但未找到 ImageToSave 实体。");
                            }
                        } else {
                            info!("[Rocket Loop] 未收到图像数据。");
                        }
                        // --- 移植逻辑结束 ---
                    }
                },
                Err(_) => {}
            }
        }
    });

    rocket::build()
        .manage(bevy_state)
        .mount("/", routes![step, render])
}
