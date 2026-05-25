use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use lol_base::hash_key::LoadHashKeyTrait;
use lol_base::ui::LOLEnumData;
use lol_base::ui_components::{UIElement, UIElementChild};

use super::init::{
    AnimAssets, DesaturateAssets, FillPercentageAssets, IconAssets, RegionAssets, TextAssets,
};

pub fn update_element_layout(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity: Entity,
    ui_element: &UIElement,
    window_size: Vec2,
    icon_assets: &IconAssets,
    anim_assets: &AnimAssets,
    desaturate_assets: &DesaturateAssets,
    region_assets: &RegionAssets,
    text_assets: &TextAssets,
    fill_percentage_assets: &FillPercentageAssets,
    q_node: &mut Query<&mut Node>,
    q_children: &Query<&Children>,
    q_image: &Query<&ImageNode>,
) {
    let Ok(mut node) = q_node.get_mut(entity) else {
        return;
    };

    let mut temp_texture_data: Option<LOLEnumData> = None;

    let (ui_position, texture_data, name, layer) = match ui_element {
        UIElement::Icon(handle) => {
            let Some(icon_data) = icon_assets.load_hash(handle) else {
                return;
            };
            (
                &icon_data.position,
                &icon_data.texture_data,
                Some(&icon_data.name),
                icon_data.layer.unwrap_or(0) as i32,
            )
        }
        UIElement::Animation(handle) => {
            let Some(anim_data) = anim_assets.load_hash(handle) else {
                return;
            };
            (
                &anim_data.position,
                &anim_data.texture_data,
                Some(&anim_data.name),
                anim_data.layer.unwrap_or(0) as i32,
            )
        }
        UIElement::Desaturate(handle) => {
            let Some(desaturate_data) = desaturate_assets.load_hash(handle) else {
                return;
            };
            (
                &desaturate_data.position,
                &desaturate_data.texture_data,
                Some(&desaturate_data.name),
                desaturate_data.layer.unwrap_or(0) as i32,
            )
        }
        UIElement::Region(handle) => {
            let Some(region_data) = region_assets.load_hash(handle) else {
                return;
            };
            (
                region_data
                    .position
                    .as_ref()
                    .unwrap_or(&lol_base::ui::LOLEnumUiPosition::UiPositionFullScreen),
                &None,
                Some(&region_data.name),
                0,
            )
        }
        UIElement::Text(handle) => {
            let Some(text_data) = text_assets.load_hash(handle) else {
                return;
            };

            (
                &text_data.position,
                &None,
                Some(&text_data.name),
                text_data.layer.unwrap_or(0) as i32,
            )
        }
        UIElement::FillPercentage(handle) => {
            let Some(fill_percentage_data) = fill_percentage_assets.load_hash(handle) else {
                return;
            };
            temp_texture_data = Some(LOLEnumData::AtlasData(
                fill_percentage_data.texture_data.clone(),
            ));
            (
                &fill_percentage_data.position,
                &temp_texture_data,
                Some(&fill_percentage_data.name),
                fill_percentage_data.layer as i32,
            )
        }
    };

    use lol_base::ui::LOLEnumUiPosition;

    let (ui_rect, anchors) = match ui_position {
        LOLEnumUiPosition::UiPositionRect(rect) => {
            (rect.ui_rect.as_ref().unwrap(), rect.anchors.as_ref())
        }
        LOLEnumUiPosition::UiPositionPolygon(polygon) => {
            (polygon.ui_rect.as_ref().unwrap(), Some(&polygon.anchors))
        }
        _ => {
            warn!("{name:?} 渲染器暂不支持的 UI 位置类型 {ui_position:?}，跳过矩形更新",);
            return;
        }
    };

    use lol_base::ui::LOLEnumAnchor;

    let anchor = match anchors {
        Some(anchors) => match anchors {
            LOLEnumAnchor::AnchorSingle(anchor) => anchor.anchor,
            _ => {
                warn!("{name:?} 渲染器暂不支持的锚点类型: {anchors:?}",);
                Vec2::ZERO
            }
        },
        None => Vec2::ZERO,
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

    let mut scale_y = window_size.y / source_resolution_height as f32;
    if ui_position.disable_resolution_downscale() {
        scale_y = 1.0;
    }

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

    commands
        .entity(entity)
        .insert((ZIndex(layer), super::OriginalPosition(position_new)));

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

    if let UIElement::Text(_) = ui_element {
        node.justify_content = JustifyContent::Center;
        node.align_items = AlignItems::Center;
        commands
            .entity(child)
            .insert(TextLayout::new_with_no_wrap());
        return;
    }

    // 更新子实体的 ImageNode
    if !q_image.contains(child) {
        if let Some(texture_data) = texture_data {
            init_child_image_node(commands, asset_server, child, texture_data, name);
        }
    }

    if let Ok(mut child_node) = q_node.get_mut(child) {
        child_node.width = Val::Px(size_new.x);
        child_node.height = Val::Px(size_new.y);
    } else {
        commands.entity(child).insert(Node {
            width: Val::Px(size_new.x),
            height: Val::Px(size_new.y),
            ..default()
        });
    }
}

pub fn update_on_add_ui_element(
    mut commands: Commands,
    mut q_element: Query<
        (
            Entity,
            &UIElement,
            &InheritedVisibility,
            Option<&super::UIButton>,
        ),
        Or<(
            Changed<UIElement>,
            Changed<Visibility>,
            Changed<InheritedVisibility>,
        )>,
    >,
    mut q_node: Query<&mut Node>,
    q_children: Query<&Children>,
    q_image: Query<&ImageNode>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    icon_assets: Res<IconAssets>,
    anim_assets: Res<AnimAssets>,
    desaturate_assets: Res<DesaturateAssets>,
    region_assets: Res<RegionAssets>,
    text_assets: Res<TextAssets>,
    fill_percentage_assets: Res<FillPercentageAssets>,
    asset_server: Res<AssetServer>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };
    let window_size = vec2(window.width(), window.height());

    for (entity, ui_element, inherited_visibility, button) in q_element.iter_mut() {
        if inherited_visibility.get() || button.is_some() {
            update_element_layout(
                &mut commands,
                &asset_server,
                entity,
                ui_element,
                window_size,
                &icon_assets,
                &anim_assets,
                &desaturate_assets,
                &region_assets,
                &text_assets,
                &fill_percentage_assets,
                &mut q_node,
                &q_children,
                &q_image,
            );
        }

        commands.entity(entity).observe(
            |event: On<Pointer<Click>>,
             q_ui_element: Query<&UIElement>,
             icon_assets: Res<IconAssets>,
             anim_assets: Res<AnimAssets>,
             desaturate_assets: Res<DesaturateAssets>,
             region_assets: Res<RegionAssets>,
             text_assets: Res<TextAssets>,
             fill_percentage_assets: Res<FillPercentageAssets>| {
                let ui_element = q_ui_element.get(event.entity).unwrap();
                let name = match ui_element {
                    UIElement::Icon(handle) => {
                        icon_assets.load_hash(*handle).map(|v| v.name.as_str())
                    }
                    UIElement::Animation(handle) => {
                        anim_assets.load_hash(*handle).map(|v| v.name.as_str())
                    }
                    UIElement::Desaturate(handle) => desaturate_assets
                        .load_hash(*handle)
                        .map(|v| v.name.as_str()),
                    UIElement::Region(handle) => {
                        region_assets.load_hash(*handle).map(|v| v.name.as_str())
                    }
                    UIElement::Text(handle) => {
                        text_assets.load_hash(*handle).map(|v| v.name.as_str())
                    }
                    UIElement::FillPercentage(handle) => fill_percentage_assets
                        .load_hash(*handle)
                        .map(|v| v.name.as_str()),
                };
                let name = name.unwrap_or("dynamic");
                println!("点击了 {}", name);
            },
        );
    }
}

pub fn update_on_window_resized(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut resize_reader: MessageReader<WindowResized>,
    mut q_element: Query<(Entity, &UIElement, &InheritedVisibility)>,
    mut q_node: Query<&mut Node>,
    q_children: Query<&Children>,
    q_image: Query<&ImageNode>,
    icon_assets: Res<IconAssets>,
    anim_assets: Res<AnimAssets>,
    desaturate_assets: Res<DesaturateAssets>,
    region_assets: Res<RegionAssets>,
    text_assets: Res<TextAssets>,
    fill_percentage_assets: Res<FillPercentageAssets>,
) {
    for e in resize_reader.read() {
        let window_size = vec2(e.width, e.height);
        for (entity, ui_element, inherited_visibility) in q_element.iter_mut() {
            if inherited_visibility.get() {
                update_element_layout(
                    &mut commands,
                    &asset_server,
                    entity,
                    ui_element,
                    window_size,
                    &icon_assets,
                    &anim_assets,
                    &desaturate_assets,
                    &region_assets,
                    &text_assets,
                    &fill_percentage_assets,
                    &mut q_node,
                    &q_children,
                    &q_image,
                );
            }
        }
    }
}

fn init_child_image_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    child: Entity,
    texture_data: &LOLEnumData,
    name: Option<&String>,
) {
    match texture_data {
        LOLEnumData::AtlasData(atlas_data) => {
            let Some(m_texture_uv) = atlas_data.m_texture_uv else {
                commands.entity(child).insert(ImageNode {
                    image: asset_server.load(&atlas_data.m_texture_name),
                    ..default()
                });
                return;
            };
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
        LOLEnumData::LooseUiTextureData(loose_data) => {
            commands.entity(child).insert(ImageNode {
                image: asset_server.load(&loose_data.texture_name),
                ..default()
            });
        }
        _ => {
            warn!("{name:?} 渲染器暂不支持的纹理数据类型: {texture_data:?}",);
        }
    }
}
