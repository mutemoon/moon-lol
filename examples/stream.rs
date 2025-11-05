use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use bevy::{
    app::AppExit,
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
use rand::Rng as _;
use tungstenite::protocol::Message;

use lol_config::{ConfigGame, ConfigNavigationGrid};
use lol_core::Team;

use moon_lol::{
    abilities::PluginAbilities,
    core::{spawn_skin_entity, Action, CameraInit, Controller, Focus, Health, PluginCore},
    entities::{spawn_fiora, PluginBarrack, PluginEntities},
};

#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

#[derive(Resource, Deref)]
struct WsImageSender(Sender<Vec<u8>>);

#[derive(Resource, Deref, DerefMut, Default, Clone)]
struct ImageCopiers(pub Vec<ImageCopier>);

#[derive(Resource, Default, Debug)]
struct SceneController {
    state: SceneState,
    name: String,
    width: u32,
    height: u32,
}

#[derive(Default, Debug)]
enum SceneState {
    #[default]
    BuildScene,
    Render(u32),
}

#[derive(Component, Deref, DerefMut)]
struct ImageToSave(Handle<Image>);

#[derive(Component, Clone)]
struct ImageCopier {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    src_image: Handle<Image>,
}

struct AppConfig {
    width: u32,
    height: u32,
}

fn main() {
    let config = AppConfig {
        width: 1920,
        height: 1080,
    };

    let (ws_tx, ws_rx) = crossbeam_channel::unbounded::<Vec<u8>>();

    thread::spawn(move || {
        ws_server_thread(ws_rx);
    });

    let mut app = App::new();
    app.insert_resource(SceneController::new(config.width, config.height))
        .insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)))
        .insert_resource(WsImageSender(ws_tx))
        .add_plugins(
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
        .add_plugins(ImageCopyPlugin)
        .add_plugins(CaptureFramePlugin)
        .init_resource::<SceneController>()
        .add_systems(Startup, setup.after(CameraInit));

    app.finish();
    app.cleanup();

    loop {
        let start_time = std::time::Instant::now();

        app.update();

        info!("(Runner) 帧更新完成。");

        let mut app_exit_events = app.world_mut().resource_mut::<Events<AppExit>>();
        if app_exit_events.drain().next().is_some() {
            info!("(Runner) 收到退出信号，关闭。");
            break;
        }

        let elapsed = start_time.elapsed();
        let sleep_duration = Duration::from_millis(50).saturating_sub(elapsed);
        if !sleep_duration.is_zero() {
            info!("(Runner) 睡眠 {} 毫秒...", sleep_duration.as_millis());
            thread::sleep(sleep_duration);
        }
    }
}

fn ws_server_thread(image_receiver: Receiver<Vec<u8>>) {
    let server = TcpListener::bind("127.0.0.1:9001").expect("无法启动 WebSocket 服务器");
    info!("[WS Server] WebSocket 服务器已启动，监听: ws://127.0.0.1:9001");

    let clients: Arc<Mutex<HashMap<i32, tungstenite::WebSocket<std::net::TcpStream>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut next_client_id = 0;

    let clients_for_broadcast = clients.clone();

    thread::spawn(move || {
        for image_data in image_receiver {
            let mut clients_guard = clients_for_broadcast.lock().unwrap();
            let mut disconnected_clients = Vec::new();

            if clients_guard.is_empty() {
                info!("[WS Broadcaster] 收到图像，但没有客户端连接。");
                continue;
            }

            info!(
                "[WS Broadcaster] 收到 {} 字节的图像。广播给 {} 个客户端。",
                image_data.len(),
                clients_guard.len()
            );

            for (id, client) in clients_guard.iter_mut() {
                let msg = Message::Binary(image_data.clone());
                if let Err(e) = client.send(msg) {
                    warn!(
                        "[WS Broadcaster] 发送失败 (客户端 {}): {}。标记为断开连接。",
                        id, e
                    );
                    disconnected_clients.push(*id);
                }
            }

            for id in disconnected_clients {
                clients_guard.remove(&id);
                info!("[WS Broadcaster] 客户端 {} 已移除。", id);
            }
        }
        info!("[WS Broadcaster] Bevy 通道已关闭。广播线程退出。");
    });

    for stream in server.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                warn!("[WS Server] 接受连接失败: {}", e);
                continue;
            }
        };

        let peer_addr = stream.peer_addr().unwrap();
        info!("[WS Server] 收到新客户端连接: {}", peer_addr);

        let clients_clone = clients.clone();
        let client_id = next_client_id;
        next_client_id += 1;

        thread::spawn(move || match tungstenite::accept(stream) {
            Ok(websocket) => {
                info!("[WS Client {}] WebSocket 握手成功。", client_id);
                {
                    let mut clients_guard = clients_clone.lock().unwrap();
                    clients_guard.insert(client_id, websocket);
                    info!("[WS Client {}] 已添加到客户端列表。", client_id);
                }

                info!("[WS Client {}] 客户端处理线程完成。", client_id);
            }
            Err(e) => {
                warn!("[WS Client {}] WebSocket 握手失败: {}", client_id, e);
            }
        });
    }
}

impl SceneController {
    pub fn new(width: u32, height: u32) -> SceneController {
        SceneController {
            state: SceneState::BuildScene,
            name: String::from(""),
            width,
            height,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<SceneController>,
    render_device: Res<RenderDevice>,
    mut q_camera: Query<&mut Camera>,
) {
    let render_target = setup_render_target(
        &mut commands,
        &mut images,
        &render_device,
        &mut scene_controller,
        1,
        "main_scene".into(),
    );

    let mut camera = q_camera.single_mut().unwrap();

    camera.target = render_target;
}

pub struct ImageCopyPlugin;

impl Plugin for ImageCopyPlugin {
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

fn setup_render_target(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    render_device: &Res<RenderDevice>,
    scene_controller: &mut ResMut<SceneController>,
    pre_roll_frames: u32,
    scene_name: String,
) -> RenderTarget {
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
        render_device,
    ));
    commands.spawn(ImageToSave(cpu_image_handle));

    scene_controller.state = SceneState::Render(pre_roll_frames);
    scene_controller.name = scene_name;
    RenderTarget::Image(render_target_image_handle.into())
}

pub struct CaptureFramePlugin;

impl Plugin for CaptureFramePlugin {
    fn build(&self, app: &mut App) {
        info!("Adding CaptureFramePlugin");
        app.add_systems(PostUpdate, update_and_stream_image);
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

fn update_and_stream_image(
    images_to_save: Query<&ImageToSave>,
    receiver: Res<MainWorldReceiver>,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<SceneController>,
    ws_sender: Res<WsImageSender>,
) {
    if let SceneState::Render(n) = scene_controller.state {
        if n < 1 {
            info!("[Bevy Update] 预热结束。尝试从 RenderWorld 接收图像...");

            let mut image_data = Vec::new();
            while let Ok(data) = receiver.try_recv() {
                image_data = data;
            }
            if image_data.is_empty() {
                info!("[Bevy Update] 未收到图像数据。");
            } else {
                info!(
                    "[Bevy Update] 收到 {} 字节的原始图像数据。正在处理...",
                    image_data.len()
                );
                for image in images_to_save.iter() {
                    let img_bytes = images.get_mut(image.id()).unwrap();
                    let row_bytes = img_bytes.width() as usize
                        * img_bytes.texture_descriptor.format.pixel_size();
                    let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);

                    if row_bytes == aligned_row_bytes {
                        img_bytes.data = Some(image_data.clone());
                    } else {
                        img_bytes.data = Some(
                            image_data
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
                            warn!("[Bevy Update] 转换图像失败: {e:?}");
                            continue;
                        }
                    };

                    let mut jpeg_bytes = Vec::new();

                    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 80);

                    info!("[Bevy Update] 正在将图像编码为 JPEG...");
                    match encoder.encode_image(&img) {
                        Ok(_) => {
                            info!(
                                "[Bevy Update] JPEG 编码成功 ({} 字节)。发送到 WebSocket...",
                                jpeg_bytes.len()
                            );

                            if let Err(e) = ws_sender.send(jpeg_bytes) {
                                warn!("[Bevy Update] 发送图像到 WS 线程失败: {e}");
                            }
                        }
                        Err(e) => {
                            warn!("[Bevy Update] JPEG 编码失败: {e}");
                        }
                    }
                }
            }
        } else {
            scene_controller.state = SceneState::Render(n - 1);
        }
    }
}

#[derive(Default)]
struct PluginGymEnv;

impl Plugin for PluginGymEnv {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fiora_player);
        app.add_systems(FixedUpdate, spawn_target);
        app.add_systems(FixedUpdate, drive_random_agent);
    }
}

#[derive(Component)]
struct AttackTarget;

#[derive(Component)]
struct RandomAgent {
    timer: Timer,
}

fn setup_fiora_player(
    mut commands: Commands,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    config_game: Res<ConfigGame>,
    asset_server: Res<AssetServer>,
    grid: Res<ConfigNavigationGrid>,
) {
    virtual_time.set_relative_speed(1.0);

    let center = grid.get_map_center_position();

    for (_, team, skin) in config_game.legends.iter() {
        let agent = spawn_skin_entity(
            &mut commands,
            &mut res_animation_graph,
            &asset_server,
            Transform::from_translation(center + vec3(-100.0, 0.0, 100.0)),
            &skin,
        );

        spawn_fiora(&mut commands, agent);

        commands.entity(agent).insert((
            team.clone(),
            Controller::default(),
            Focus,
            Pickable::IGNORE,
        ));

        commands.entity(agent).insert(RandomAgent {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        });
    }
}

fn spawn_target(
    mut commands: Commands,
    q_t: Query<&AttackTarget>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    res_navigation_grid: Res<ConfigNavigationGrid>,
    config_game: Res<ConfigGame>,
) {
    if q_t.single().is_ok() {
        return;
    }

    for (_, _, skin) in config_game.legends.iter() {
        let map_center_position = res_navigation_grid.get_map_center_position();

        let target = spawn_skin_entity(
            &mut commands,
            &mut res_animation_graph,
            &asset_server,
            Transform::from_translation(map_center_position + vec3(100.0, 0.0, -100.0)),
            &skin,
        );

        spawn_fiora(&mut commands, target);

        commands.entity(target).insert((
            Team::Chaos,
            Health {
                value: 6000.0,
                max: 6000.0,
            },
            AttackTarget,
        ));
    }
}

fn drive_random_agent(
    mut commands: Commands,
    mut agents: Query<(Entity, &mut RandomAgent, &Transform)>,
    q_target: Query<Entity, With<AttackTarget>>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut agent, transform) in agents.iter_mut() {
        agent.timer.tick(time.delta());
        if !agent.timer.just_finished() {
            continue;
        }

        let mut rng = rand::rng();
        let choice = rng.random_range(0..3);

        let action = match choice {
            0 => Action::Attack(q_target.single().unwrap()),
            1 => {
                let angle = rng.random_range(0.0f32..std::f32::consts::TAU);
                let radius = rng.random_range(50.0f32..200.0f32);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                Action::Move(transform.translation.xz() + offset)
            }
            2 => Action::Stop,
            _ => Action::Stop,
        };

        commands
            .entity(entity)
            .trigger(moon_lol::core::CommandAction { action });
    }
}
