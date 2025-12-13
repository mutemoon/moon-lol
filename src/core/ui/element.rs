use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use league_core::{EnumAnchor, EnumData, EnumUiPosition, UiElementIconData};
use league_utils::hash_bin;

use crate::CommandLoadPropBin;

#[derive(Resource, Default)]
pub struct UIElementEntity {
    pub map: HashMap<u32, Entity>,
}

impl UIElementEntity {
    pub fn get_by_string(&self, key: &str) -> Option<&Entity> {
        self.map.get(&hash_bin(key))
    }
}

#[derive(Component)]
pub struct UIElement {
    pub key: String,
    pub position: EnumUiPosition,
    pub update_child: bool,
}

#[derive(States, Default, Debug, Hash, Eq, Clone, PartialEq)]
pub enum UIState {
    #[default]
    Loading,
    Loaded,
}

pub fn startup_load_ui(mut commands: Commands) {
    let paths = vec!["clientstates/gameplay/ux/lol/playerframe/uibase".to_string()];

    commands.trigger(CommandLoadPropBin { paths });
}

pub fn update_spawn_ui_element(
    mut commands: Commands,
    mut res_ui_element_entity: ResMut<UIElementEntity>,
    res_asset_server: Res<AssetServer>,
    res_assets_ui_element_icon_data: Res<Assets<UiElementIconData>>,
    ) {
    if res_assets_ui_element_icon_data.iter().count() == 0 {
        return;
    }

    for (_, ui) in res_assets_ui_element_icon_data.iter().filter(|v| {
        v.1.name
            .contains("ClientStates/Gameplay/UX/LoL/PlayerFrame/")
    }) {
        let Some(entity) = spawn_ui_element(&mut commands, &res_asset_server, ui) else {
            continue;
        };

        commands.entity(entity).insert(Visibility::Hidden);

        if let Some(&enabled) = ui.enabled.as_ref() {
            if enabled {
                commands.entity(entity).insert(Visibility::Visible);
            }
        }

        if ui.name.ends_with("_BorderAvailable") {
            commands.entity(entity).insert(Visibility::Visible);
        }

        res_ui_element_entity.map.insert(hash_bin(&ui.name), entity);
    }

    // commands.trigger(CommandUiAnimationStart {
    //     key: "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/LevelUpFxIn/LevelUp0_ButtonIn".to_string(),
    // });
    commands.set_state(UIState::Loaded);
}

pub fn spawn_ui_element(
    commands: &mut Commands,
    res_asset_server: &Res<AssetServer>,
    ui: &UiElementIconData,
) -> Option<Entity> {
    spawn_ui_atom(
        commands,
        res_asset_server,
        &ui.name,
        &ui.position,
        &ui.layer,
        &ui.texture_data,
    )
}

pub fn spawn_ui_atom(
    commands: &mut Commands,
    res_asset_server: &Res<AssetServer>,
    key: &String,
    position: &EnumUiPosition,
    layer: &Option<u32>,
    texture_data: &Option<EnumData>,
) -> Option<Entity> {
    let entity = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                overflow: Overflow::hidden(),
                ..default()
            },
            ZIndex(layer.unwrap_or(0) as i32),
            Pickable::IGNORE,
            UIElement {
                key: key.clone(),
                position: position.clone(),
                update_child: true,
            },
        ))
        .observe(
            |event: On<Pointer<Click>>, q_ui_element: Query<&UIElement>| {
                let ui_element = q_ui_element.get(event.entity).unwrap();
                println!("点击了 {}", ui_element.key);
            },
        )
        .id();

    let child_entity = commands.spawn((Node::default(), Pickable::IGNORE)).id();

    if let Some(texture_data) = texture_data {
        if let EnumData::AtlasData(atlas_data) = texture_data {
            if let Some(m_texture_uv) = atlas_data.m_texture_uv {
                commands.entity(child_entity).insert(ImageNode {
                    image: res_asset_server.load(format!("{}#srgb", atlas_data.m_texture_name)),
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
    };

    commands.entity(entity).add_child(child_entity);

    Some(entity)
}

fn apply_rect_position(node: &mut Node, ui_element: &UIElement, window_size: Vec2) -> Option<Vec2> {
    let EnumUiPosition::UiPositionRect(ref position) = ui_element.position else {
        return None;
    };

    let Some(ui_rect) = &position.ui_rect else {
        return None;
    };

    let Some(anchors) = &position.anchors else {
        return None;
    };

    let EnumAnchor::AnchorSingle(anchor) = anchors else {
        return None;
    };
    let anchor = anchor.anchor;

    let Some(position) = ui_rect.position else {
        return None;
    };

    let Some(size) = ui_rect.size else {
        return None;
    };

    let Some(source_resolution_width) = ui_rect.source_resolution_width else {
        return None;
    };

    let Some(source_resolution_height) = ui_rect.source_resolution_height else {
        return None;
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

    Some(size_new)
}

fn update_element_layout(
    entity: Entity,
    ui_element: &UIElement,
    window_size: Vec2,
    q_node: &mut Query<&mut Node>,
    q_children: &Query<&Children>,
) {
    let Some(size_new) = ({
        let Ok(mut node) = q_node.get_mut(entity) else {
            return;
        };
        apply_rect_position(&mut node, ui_element, window_size)
    }) else {
        return;
    };

    if !ui_element.update_child {
        return;
    }

    let Ok(children) = q_children.get(entity) else {
        return;
    };

    let Some(&child) = children.first() else {
        return;
    };

    let Ok(mut child_node) = q_node.get_mut(child) else {
        return;
    };

    child_node.width = Val::Px(size_new.x);
    child_node.height = Val::Px(size_new.y);
}

pub fn update_on_add_ui_element(
    mut q_element: Query<(Entity, &UIElement), Added<UIElement>>,
    q_children: Query<&Children>,
    mut q_node: Query<&mut Node>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };
    let window_size = vec2(window.width(), window.height());

    for (entity, ui_element) in q_element.iter_mut() {
        update_element_layout(entity, ui_element, window_size, &mut q_node, &q_children);
    }
}

pub fn update_ui_element(
    mut q_element: Query<(Entity, &UIElement)>,
    q_children: Query<&Children>,
    mut q_node: Query<&mut Node>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    for e in resize_reader.read() {
        let window_size = vec2(e.width, e.height);
        for (entity, ui_element) in q_element.iter_mut() {
            update_element_layout(entity, ui_element, window_size, &mut q_node, &q_children);
        }
    }
}
