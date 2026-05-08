use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use league_utils::hash_bin;
use lol_base::ui::{
    LOLEnumAnchor, LOLEnumData, LOLEnumUiPosition, LOLUiElementEffectAnimationData,
    LOLUiElementGroupButtonData, LOLUiElementIconData, LOLUiElementRegionData, LOLUiFile,
    LOLUiHandles, LOLUiPaths, LOLUiScenes,
};
use lol_base::ui_components::{UIElement, UIElementChild};

pub struct PluginUIElement;

impl Plugin for PluginUIElement {
    fn build(&self, app: &mut App) {
        app.init_state::<UIState>();
        app.init_resource::<UIElementEntity>();
        app.init_resource::<LOLUiScenes>();
        app.init_resource::<LOLUiHandles>();
        app.init_asset::<LOLUiElementIconData>();
        app.init_asset::<LOLUiElementEffectAnimationData>();
        app.init_asset::<LOLUiElementGroupButtonData>();
        app.init_asset::<LOLUiElementRegionData>();

        app.add_systems(Startup, startup_load_ui_data);
        app.add_systems(
            Update,
            (update_on_window_resized, update_on_add_ui_element).run_if(in_state(UIState::Loaded)),
        );

        app.add_observer(on_command_update_ui_element);
    }
}

#[derive(States, Default, Debug, Hash, Eq, Clone, PartialEq)]
pub enum UIState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Resource, Default)]
pub struct UIElementEntity {
    pub map: HashMap<u32, Entity>,
}

#[derive(EntityEvent, Debug)]
pub struct CommandUpdateUIElement {
    pub entity: Entity,
    pub size_type: SizeType,
    pub value: f32,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum SizeType {
    Width,
    Height,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum NodeType {
    Parent,
    Child,
}

impl UIElementEntity {
    pub fn get_by_string(&self, key: &str) -> Option<&Entity> {
        self.map.get(&hash_bin(key))
    }
}

fn startup_load_ui_data(
    mut commands: Commands,
    mut res_ui_scenes: ResMut<LOLUiScenes>,
    mut res_ui_handles: ResMut<LOLUiHandles>,
    mut res_ui_element_entity: ResMut<UIElementEntity>,
    mut icon_assets: ResMut<Assets<LOLUiElementIconData>>,
    mut anim_assets: ResMut<Assets<LOLUiElementEffectAnimationData>>,
    mut button_assets: ResMut<Assets<LOLUiElementGroupButtonData>>,
    mut region_assets: ResMut<Assets<LOLUiElementRegionData>>,
) {
    let ui_paths = LOLUiPaths::default();
    let ron_paths = vec![
        ui_paths.player_frame_ron(),
        ui_paths.floating_info_bars_ron(),
    ];

    for path in ron_paths {
        let Ok(content) = std::fs::read_to_string(format!("assets/{}", path)) else {
            warn!("无法读取 UI 数据文件: {}", path);
            continue;
        };

        let Ok(data) = ron::from_str::<LOLUiFile>(&content) else {
            warn!("无法解析 UI 数据文件: {}", path);
            continue;
        };

        for (hash, scene_data) in data.scenes {
            res_ui_scenes.scenes.insert(hash, scene_data);
        }

        // 合并数据并 spawn 实体
        for (hash, icon_data) in data.elements {
            let handle = icon_assets.add(icon_data.clone());
            res_ui_handles.icon_handles.insert(hash, handle.clone());

            let is_visible = determine_visibility(&icon_data, &res_ui_scenes);
            let entity = commands
                .spawn((
                    ZIndex(icon_data.layer.unwrap_or(0) as i32),
                    UIElement::Handle(handle),
                    if is_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                ))
                .id();
            res_ui_element_entity.map.insert(hash, entity);
        }

        for (hash, anim_data) in data.animation_elements {
            let handle = anim_assets.add(anim_data.clone());
            res_ui_handles.animation_handles.insert(hash, handle);
        }

        for (hash, button_data) in data.button_elements {
            let handle = button_assets.add(button_data.clone());
            res_ui_handles.button_handles.insert(hash, handle);
        }

        for (hash, region_data) in data.region_elements {
            let handle = region_assets.add(region_data.clone());
            res_ui_handles.region_handles.insert(hash, handle);
        }
    }

    info!(
        "UI 元素初始化完成，一共 {} 个实体",
        res_ui_element_entity.map.len()
    );
    commands.set_state(UIState::Loaded);
}

fn determine_visibility(icon_data: &LOLUiElementIconData, res_ui_scenes: &LOLUiScenes) -> bool {
    if let Some(scene) = res_ui_scenes.scenes.get(&icon_data.scene) {
        if !scene.enabled {
            return false;
        }
    }
    icon_data.enabled
}

fn update_element_layout(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity: Entity,
    ui_element: &UIElement,
    window_size: Vec2,
    icon_assets: &Assets<LOLUiElementIconData>,
    q_node: &mut Query<&mut Node>,
    q_children: &Query<&Children>,
) {
    let Ok(mut node) = q_node.get_mut(entity) else {
        return;
    };

    let (position, texture_data) = match ui_element {
        UIElement::Handle(handle) => {
            let Some(icon_data) = icon_assets.get(handle) else {
                return;
            };
            (&icon_data.position, &icon_data.texture_data)
        }
        UIElement::Data {
            position,
            texture_data,
        } => (position, texture_data),
    };

    let LOLEnumUiPosition::UiPositionRect(position_rect) = position;

    let Some(ui_rect) = &position_rect.ui_rect else {
        return;
    };

    let anchor = match &position_rect.anchors {
        Some(LOLEnumAnchor::AnchorSingle(anchor)) => anchor.anchor,
        _ => Vec2::ZERO,
    };

    let Some(position) = ui_rect.position else {
        return;
    };

    let Some(size) = ui_rect.size else {
        return;
    };

    let Some(source_resolution_width) = ui_rect.source_resolution_width else {
        return;
    };

    let Some(source_resolution_height) = ui_rect.source_resolution_height else {
        return;
    };

    let scale_y = window_size.y / source_resolution_height as f32;

    let canvas_size_old = vec2(
        source_resolution_width as f32,
        source_resolution_height as f32,
    );
    let canvas_size_new = window_size;

    let anchor_old = canvas_size_old * anchor;
    let anchor_new = canvas_size_new * anchor;

    let position_new = (position - anchor_old) * scale_y + anchor_new;
    let size_new = size * scale_y;

    node.left = Val::Px(position_new.x);
    node.top = Val::Px(position_new.y);

    node.width = Val::Px(size_new.x);
    node.height = Val::Px(size_new.y);

    let child = if let Ok(children) = q_children.get(entity) {
        if let Some(&child) = children.first() {
            child
        } else {
            let child = commands.spawn(UIElementChild).id();
            commands.entity(entity).add_child(child);
            child
        }
    } else {
        let child = commands.spawn(UIElementChild).id();
        commands.entity(entity).add_child(child);
        child
    };

    // 更新子实体的 ImageNode
    if let Some(texture_data) = texture_data {
        let LOLEnumData::AtlasData(atlas_data) = texture_data;
        if let Some(m_texture_uv) = atlas_data.m_texture_uv {
            commands.entity(child).insert(ImageNode {
                image: asset_server.load(&atlas_data.m_texture_name),
                rect: Some(Rect::new(
                    m_texture_uv.x,
                    m_texture_uv.y,
                    m_texture_uv.z,
                    m_texture_uv.w,
                )),
                ..default()
            });
        }
    }

    if let Ok(mut child_node) = q_node.get_mut(child) {
        child_node.width = Val::Px(size_new.x);
        child_node.height = Val::Px(size_new.y);
    }
}

fn update_on_add_ui_element(
    mut commands: Commands,
    mut q_element: Query<(Entity, &UIElement), Added<UIElement>>,
    mut q_node: Query<&mut Node>,
    q_children: Query<&Children>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    icon_assets: Res<Assets<LOLUiElementIconData>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };
    let window_size = vec2(window.width(), window.height());

    for (entity, ui_element) in q_element.iter_mut() {
        update_element_layout(
            &mut commands,
            &asset_server,
            entity,
            ui_element,
            window_size,
            &icon_assets,
            &mut q_node,
            &q_children,
        );

        commands.entity(entity).observe(
            |event: On<Pointer<Click>>,
             q_ui_element: Query<&UIElement>,
             icon_assets: Res<Assets<LOLUiElementIconData>>| {
                let ui_element = q_ui_element.get(event.entity).unwrap();
                let icon_data = match ui_element {
                    UIElement::Handle(handle) => icon_assets.get(handle),
                    UIElement::Data { .. } => None, // Data variant doesn't have name anymore
                };
                let name = icon_data.map(|v| v.name.as_str()).unwrap_or("dynamic");
                println!("点击了 {}", name);
            },
        );
    }
}

fn update_on_window_resized(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut resize_reader: MessageReader<WindowResized>,
    mut q_element: Query<(Entity, &UIElement)>,
    mut q_node: Query<&mut Node>,
    q_children: Query<&Children>,
    icon_assets: Res<Assets<LOLUiElementIconData>>,
) {
    for e in resize_reader.read() {
        let window_size = vec2(e.width, e.height);
        for (entity, ui_element) in q_element.iter_mut() {
            update_element_layout(
                &mut commands,
                &asset_server,
                entity,
                ui_element,
                window_size,
                &icon_assets,
                &mut q_node,
                &q_children,
            );
        }
    }
}

fn on_command_update_ui_element(
    trigger: On<CommandUpdateUIElement>,
    q_children: Query<&Children>,
    mut q_node: Query<&mut Node>,
) {
    let entity = trigger.entity;
    let size_type = trigger.size_type;
    let value = trigger.value;
    let node_type = trigger.node_type;

    let Ok(children) = q_children.get(entity) else {
        return;
    };

    let Ok(child_node) = q_node.get(children[0]) else {
        return;
    };

    let (target_entity, standard_size) = match node_type {
        NodeType::Parent => {
            let size = match size_type {
                SizeType::Width => {
                    if let Val::Px(width) = child_node.width {
                        width
                    } else {
                        return;
                    }
                }
                SizeType::Height => {
                    if let Val::Px(height) = child_node.height {
                        height
                    } else {
                        return;
                    }
                }
            };
            (entity, size)
        }
        NodeType::Child => {
            let Ok(parent_node) = q_node.get(entity) else {
                return;
            };
            let size = match size_type {
                SizeType::Width => {
                    if let Val::Px(width) = parent_node.width {
                        width
                    } else {
                        return;
                    }
                }
                SizeType::Height => {
                    if let Val::Px(height) = parent_node.height {
                        height
                    } else {
                        return;
                    }
                }
            };
            (children[0], size)
        }
    };

    let Ok(mut target_node) = q_node.get_mut(target_entity) else {
        return;
    };

    match size_type {
        SizeType::Width => target_node.width = Val::Px(standard_size * value),
        SizeType::Height => target_node.height = Val::Px(standard_size * value),
    }
}
