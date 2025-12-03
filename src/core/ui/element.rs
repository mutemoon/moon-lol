use std::collections::HashMap;

use bevy::{prelude::*, window::WindowResized};

use league_core::{
    UiElementEffectAnimationDataTextureData, UiElementIconData, UiElementIconDataPosition,
    UiPositionRectAnchors,
};
use league_utils::hash_bin;
use lol_config::ConfigUi;

use crate::CommandUiAnimationStart;

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
    pub position: UiElementIconDataPosition,
    pub update_child: bool,
}

pub fn startup_spawn_ui_element(
    mut commands: Commands,
    res_config_ui: Res<ConfigUi>,
    res_asset_server: Res<AssetServer>,
    mut res_ui_element_entity: ResMut<UIElementEntity>,
) {
    for (key, ui) in res_config_ui.ui_elements.iter().filter(|v| {
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

        res_ui_element_entity.map.insert(key.clone(), entity);
    }

    commands.trigger(CommandUiAnimationStart {
        key: "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/LevelUpFxIn/LevelUp0_ButtonIn".to_string(),
    });
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
    position: &UiElementIconDataPosition,
    layer: &Option<u32>,
    texture_data: &Option<UiElementEffectAnimationDataTextureData>,
) -> Option<(Entity)> {
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
        if let UiElementEffectAnimationDataTextureData::AtlasData(atlas_data) = texture_data {
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

pub fn update_resize_system(
    mut q_element: Query<(Entity, &UIElement)>,
    q_children: Query<&Children>,
    mut q_node: Query<&mut Node>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    for e in resize_reader.read() {
        for (entity, ui_element) in q_element.iter_mut() {
            let mut node = q_node.get_mut(entity).unwrap();

            let UiElementIconDataPosition::UiPositionRect(ref position) = ui_element.position
            else {
                continue;
            };

            let Some(ui_rect) = &position.ui_rect else {
                continue;
            };

            let Some(anchors) = &position.anchors else {
                continue;
            };

            let UiPositionRectAnchors::AnchorSingle(anchor) = anchors else {
                continue;
            };
            let anchor = anchor.anchor;

            let Some(position) = ui_rect.position else {
                continue;
            };

            let Some(size) = ui_rect.size else {
                continue;
            };

            let Some(source_resolution_width) = ui_rect.source_resolution_width else {
                continue;
            };

            let Some(source_resolution_height) = ui_rect.source_resolution_height else {
                continue;
            };

            let scale_y = e.height / source_resolution_height as f32;

            let canvas_size_old = vec2(
                source_resolution_width as f32,
                source_resolution_height as f32,
            );
            let canvas_size_new = vec2(e.width, e.height);

            let anchor_old = canvas_size_old * anchor;
            let anchor_new = canvas_size_new * anchor;

            let position_new = (position - anchor_old) * scale_y + anchor_new;
            let size_new = size * scale_y;

            node.left = Val::Px(position_new.x);
            node.top = Val::Px(position_new.y);

            node.width = Val::Px(size_new.x);
            node.height = Val::Px(size_new.y);

            if ui_element.update_child {
                let children = q_children.get(entity).unwrap();
                let mut child_node = q_node.get_mut(children[0]).unwrap();

                child_node.width = Val::Px(size_new.x);
                child_node.height = Val::Px(size_new.y);
            }
        }
    }
}
