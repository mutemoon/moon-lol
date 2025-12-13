use std::collections::HashMap;

use bevy::prelude::*;
use league_core::{UiElementGroupButtonData, UiElementRegionData};
use lol_config::{HashKey, LoadHashKeyTrait};

use crate::core::ui::element::UIElementEntity;
use crate::UIElement;

#[derive(Resource, Default)]
pub struct UIButtonEntity {
    pub map: HashMap<u32, Entity>,
}

#[derive(Component)]
#[require(Interaction)]
pub struct UIButton {
    pub key: HashKey<UiElementGroupButtonData>,
}

#[derive(Event)]
pub struct CommandSpawnButton {
    pub entity: Option<Entity>,
    pub key: HashKey<UiElementGroupButtonData>,
}

#[derive(EntityEvent)]
pub struct CommandDespawnButton {
    pub entity: Entity,
}

pub fn startup_spawn_buttons(
    mut commands: Commands,
    res_assets_ui_element_group_button_data: Res<Assets<UiElementGroupButtonData>>,
    ) {
    for (key, ui_element_group_button_data) in
        res_assets_ui_element_group_button_data.iter().filter(|v| {
            v.1.name
                .contains("ClientStates/Gameplay/UX/LoL/PlayerFrame/")
        })
    {
        let is_enabled = ui_element_group_button_data.is_enabled.unwrap_or(false);
        if !is_enabled {
            continue;
        }

        commands.trigger(CommandSpawnButton {
            key: key.into(),
            entity: None,
        });
    }
}

pub fn on_command_spawn_button(
    trigger: On<CommandSpawnButton>,
    mut commands: Commands,
    res_assets_ui_element_group_button_data: Res<Assets<UiElementGroupButtonData>>,
    res_ui_region: Res<Assets<UiElementRegionData>>,
    ) {
    let key = trigger.key;
    let ui_element_group_button_data = res_assets_ui_element_group_button_data
        .load_hash(key)
        .unwrap();

    let hit_region = res_ui_region
        .load_hash(ui_element_group_button_data.hit_region_element)
        .unwrap();
    let bundle = (
        Node::default(),
        UIElement {
            key: ui_element_group_button_data.hit_region_element.to_string(),
            position: hit_region.position.clone().unwrap(),
            update_child: false,
        },
        UIButton { key },
    );

    if let Some(entity) = trigger.entity {
        commands.entity(entity).insert(bundle);
    } else {
        commands.spawn(bundle);
    }
}

pub fn on_command_despawn_button(
    trigger: On<CommandDespawnButton>,
    mut commands: Commands,
    q_ui_button: Query<&UIButton>,
    res_assets_ui_element_group_button_data: Res<Assets<UiElementGroupButtonData>>,
        res_ui_element_entity: Res<UIElementEntity>,
) {
    commands.entity(trigger.entity).despawn();

    let Ok(button) = q_ui_button.get(trigger.entity) else {
        return;
    };

    let ui_element_group_button_data = res_assets_ui_element_group_button_data
        .load_hash(button.key)
        .unwrap();

    for element in ui_element_group_button_data.elements.iter() {
        let Some(&element_entity) = res_ui_element_entity.map.get(element) else {
            continue;
        };
        commands.entity(element_entity).insert(Visibility::Hidden);
    }
}

pub fn update_button(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &UIButton), Changed<Interaction>>,
    res_assets_ui_element_group_button_data: Res<Assets<UiElementGroupButtonData>>,
        res_ui_element_entity: Res<UIElementEntity>,
) {
    for (interaction, button) in &mut interaction_query {
        let ui_element_group_button_data = res_assets_ui_element_group_button_data
            .load_hash(button.key)
            .unwrap();

        let interaction_entity = match *interaction {
            Interaction::Pressed => {
                debug!("按下 {}", ui_element_group_button_data.name);
                let Some(&clicked_entity) = ui_element_group_button_data
                    .clicked_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                else {
                    continue;
                };
                clicked_entity
            }
            Interaction::Hovered => {
                debug!("悬停 {}", ui_element_group_button_data.name);
                let &hover_entity = ui_element_group_button_data
                    .hover_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                    .unwrap();
                hover_entity
            }
            Interaction::None => {
                debug!("恢复 {}", ui_element_group_button_data.name);
                let &default_entity = ui_element_group_button_data
                    .default_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                    .unwrap();
                default_entity
            }
        };

        for element in ui_element_group_button_data.elements.iter() {
            let Some(&element_entity) = res_ui_element_entity.map.get(element) else {
                continue;
            };
            commands.entity(element_entity).insert(Visibility::Hidden);
        }

        commands
            .entity(interaction_entity)
            .insert(Visibility::Visible);
    }
}
