use bevy::asset::RecursiveDependencyLoadState;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::ui::{Overflow, OverflowAxis, ScrollPosition};
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base::particle::{ConfigVfx, ConfigVfxSystemDefinition};
use lol_render::camera::PluginCamera;
use lol_render::particle::{CommandParticleDespawn, CommandParticleSpawn, PluginParticle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            // 因为更新后的 Bevy 仅在设备启用 PASSTHROUGH_SHADERS 时才将 SpirV 走 wgpu passthrough（原样使用），
            // 否则回退 naga 解析而无法处理这些 DXC 编译的 League SPIR-V（报 Unable to find entry point 'main'），
            // 所以显式向设备请求该特性（在保留默认 TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES 的基础上追加）。
            render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                features: WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | WgpuFeatures::PASSTHROUGH_SHADERS,
                ..default()
            })),
            ..default()
        }))
        .add_plugins(PluginParticle)
        .add_plugins(PluginCamera)
        .init_resource::<PlayingVfx>()
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(
            Update,
            (
                on_hero_header_click,
                poll_hero_load,
                on_system_click,
                on_collapse_all,
                scroll_tree,
            ),
        )
        .run();
}

/// 记录当前正在播放的粒子 hash，切换时用于停止上一个
#[derive(Resource, Default)]
struct PlayingVfx(Option<u32>);

/// 粒子播放锚点（所有点击的粒子都在此实体上生成）
#[derive(Component)]
struct ParticleAnchor;

/// 左侧面板宽度（像素）
const PANEL_WIDTH: f32 = 320.0;

/// 左侧树的可滚动列表容器
#[derive(Component)]
struct VfxTreeList;

/// 左侧面板根节点（用于让相机识别指针悬停在 UI 上）
#[derive(Component)]
struct VfxTreePanel;

/// 一个英雄节点（树的第一层），挂在英雄列（Column）实体上
#[derive(Component)]
struct HeroNode {
    name: String,
    vfx_path: String,
    expanded: bool,
    loading: bool,
    loaded: bool,
    /// 第二层子节点的容器实体
    list: Entity,
    /// 表头文字实体（用于更新展开箭头）
    label: Entity,
    handle: Option<Handle<ConfigVfx>>,
}

/// 英雄表头按钮，指向对应的英雄列实体
#[derive(Component)]
struct HeroHeaderButton {
    hero: Entity,
}

/// 一个 ConfigVfxSystemDefinition 叶子按钮（树的第二层）
#[derive(Component)]
struct SystemButton {
    hash: u32,
}

/// "收起全部"按钮
#[derive(Component)]
struct CollapseAllButton;

fn main_panel_bg() -> Color {
    Color::srgba(0.05, 0.05, 0.06, 0.92)
}

/// 扫描 assets/characters 目录，列出所有带 skin0_vfx.ron 的英雄（名称升序）
fn enumerate_heroes() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let base = std::path::Path::new("assets/characters");
    let Ok(read_dir) = std::fs::read_dir(base) else {
        warn!("无法读取 assets/characters 目录");
        return out;
    };
    for entry in read_dir.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let vfx_fs = base.join(&name).join("skins").join("skin0_vfx.ron");
        if vfx_fs.is_file() {
            out.push((
                name.clone(),
                format!("characters/{name}/skins/skin0_vfx.ron"),
            ));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地面参照物
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(std_materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::default().with_translation(Vec3::NEG_Y * 10.0),
    ));

    // 粒子播放锚点，放在原点，供点击后生成粒子
    commands.spawn((ParticleAnchor, Transform::default()));
}

fn setup_ui(mut commands: Commands) {
    // 左侧固定面板
    let panel = commands
        .spawn((
            VfxTreePanel,
            Interaction::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(PANEL_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(main_panel_bg()),
        ))
        .id();

    // "收起全部"按钮
    let collapse = commands
        .spawn((
            Button,
            CollapseAllButton,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.22, 0.28)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text::new("收起全部"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::WHITE),
        ChildOf(collapse),
    ));

    // 可滚动的英雄列表
    let list = commands
        .spawn((
            VfxTreeList,
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Scroll,
                },
                ..default()
            },
            ScrollPosition::default(),
            ChildOf(panel),
        ))
        .id();

    let heroes = enumerate_heroes();
    info!("发现 {} 个英雄", heroes.len());

    for (name, vfx_path) in heroes {
        // 英雄列（表头 + 子列表）
        let hero = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ChildOf(list),
            ))
            .id();

        // 表头按钮
        let header = commands
            .spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(26.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::left(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.13, 0.13, 0.17)),
                ChildOf(hero),
            ))
            .id();
        let label = commands
            .spawn((
                Text::new(format!("▶ {name}")),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ChildOf(header),
            ))
            .id();

        // 第二层子列表（默认隐藏）
        let child_list = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    ..default()
                },
                ChildOf(hero),
            ))
            .id();

        commands.entity(header).insert(HeroHeaderButton { hero });
        commands.entity(hero).insert(HeroNode {
            name,
            vfx_path,
            expanded: false,
            loading: false,
            loaded: false,
            list: child_list,
            label,
            handle: None,
        });
    }
}

fn on_hero_header_click(
    q_btn: Query<(&Interaction, &HeroHeaderButton), (Changed<Interaction>, With<Button>)>,
    mut q_hero: Query<&mut HeroNode>,
    mut q_node: Query<&mut Node>,
    mut q_text: Query<&mut Text>,
    asset_server: Res<AssetServer>,
) {
    for (interaction, header) in &q_btn {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(mut hero) = q_hero.get_mut(header.hero) else {
            continue;
        };
        hero.expanded = !hero.expanded;

        let arrow = if hero.expanded { "▼" } else { "▶" };
        if let Ok(mut text) = q_text.get_mut(hero.label) {
            *text = Text::new(format!("{arrow} {}", hero.name));
        }

        if hero.expanded {
            if let Ok(mut node) = q_node.get_mut(hero.list) {
                node.display = Display::Flex;
            }
            if !hero.loaded && !hero.loading {
                hero.loading = true;
                // skin0_vfx.ron 为纯 serde RON，由 ConfigVfxLoader 直接加载为 ConfigVfx 资产
                let handle = asset_server.load::<ConfigVfx>(hero.vfx_path.clone());
                hero.handle = Some(handle);
            }
        } else if let Ok(mut node) = q_node.get_mut(hero.list) {
            node.display = Display::None;
        }
    }
}

fn poll_hero_load(
    mut q_hero: Query<&mut HeroNode>,
    asset_server: Res<AssetServer>,
    config_vfxs: Res<Assets<ConfigVfx>>,
    mut vfx_assets: ResMut<Assets<ConfigVfxSystemDefinition>>,
    mut commands: Commands,
) {
    for mut hero in &mut q_hero {
        if !hero.loading {
            continue;
        }
        let Some(handle) = hero.handle.clone() else {
            hero.loading = false;
            continue;
        };
        match asset_server.get_recursive_dependency_load_state(&handle) {
            Some(RecursiveDependencyLoadState::Loaded) => {}
            Some(RecursiveDependencyLoadState::Failed(err)) => {
                warn!("英雄 {} 的 vfx 配置加载失败: {err}", hero.name);
                hero.loading = false;
                continue;
            }
            _ => continue,
        }
        let Some(config_vfx) = config_vfxs.get(&handle) else {
            continue;
        };

        let list = hero.list;
        {
            for (&hash, def) in &config_vfx.systems {
                // 注入到 Assets，使点击时 on_command_particle_spawn 能按 hash 查到定义
                vfx_assets.add_hash(hash, def.clone());

                let btn = commands
                    .spawn((
                        Button,
                        SystemButton { hash },
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(20.0),
                            align_items: AlignItems::Center,
                            padding: UiRect::left(Val::Px(24.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.09, 0.09, 0.12)),
                        ChildOf(list),
                    ))
                    .id();
                commands.spawn((
                    Text::new(def.particle_name.clone()),
                    TextFont {
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.8, 0.92)),
                    ChildOf(btn),
                ));
            }
            info!(
                "英雄 {} 载入 {} 个粒子系统",
                hero.name,
                config_vfx.systems.len()
            );
        }

        hero.loading = false;
        hero.loaded = true;
    }
}

fn on_system_click(
    q_btn: Query<(&Interaction, &SystemButton), (Changed<Interaction>, With<Button>)>,
    q_anchor: Query<Entity, With<ParticleAnchor>>,
    mut playing: ResMut<PlayingVfx>,
    mut commands: Commands,
) {
    let Ok(anchor) = q_anchor.single() else {
        return;
    };
    for (interaction, sys) in &q_btn {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // 停止上一个粒子
        if let Some(prev) = playing.0.take() {
            commands
                .entity(anchor)
                .trigger(move |entity| CommandParticleDespawn {
                    entity,
                    vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(prev),
                });
        }
        let hash = sys.hash;
        commands
            .entity(anchor)
            .trigger(move |entity| CommandParticleSpawn {
                entity,
                vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(hash),
            });
        playing.0 = Some(hash);
        info!("播放粒子 hash={hash:08x}");
    }
}

fn on_collapse_all(
    q_btn: Query<&Interaction, (Changed<Interaction>, With<CollapseAllButton>)>,
    mut q_hero: Query<&mut HeroNode>,
    mut q_node: Query<&mut Node>,
    mut q_text: Query<&mut Text>,
) {
    let pressed = q_btn.iter().any(|i| *i == Interaction::Pressed);
    if !pressed {
        return;
    }
    for mut hero in &mut q_hero {
        if !hero.expanded {
            continue;
        }
        hero.expanded = false;
        if let Ok(mut node) = q_node.get_mut(hero.list) {
            node.display = Display::None;
        }
        if let Ok(mut text) = q_text.get_mut(hero.label) {
            *text = Text::new(format!("▶ {}", hero.name));
        }
    }
}

fn scroll_tree(
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut q_list: Query<&mut ScrollPosition, With<VfxTreeList>>,
) {
    // 因为只有指针在面板区域内时才应滚动列表（面板外的滚轮交给相机缩放），
    // 所以先判断光标是否落在面板矩形内，否则丢弃事件直接返回
    let over_panel = windows
        .iter()
        .find_map(|window| window.cursor_position())
        .map(|cursor| cursor.x <= PANEL_WIDTH)
        .unwrap_or(false);
    if !over_panel {
        for _ in wheel.read() {}
        return;
    }

    let mut delta = 0.0;
    for event in wheel.read() {
        delta += event.y;
    }
    if delta == 0.0 {
        return;
    }
    for mut scroll in &mut q_list {
        scroll.0.y = (scroll.0.y - delta * 24.0).max(0.0);
    }
}
