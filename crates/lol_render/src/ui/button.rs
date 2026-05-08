use bevy::prelude::*;
use lol_base::ui::{LOLUiElementGroupButtonData, LOLUiHandles};
use lol_base::ui_components::{UIButton, UIElement};

use crate::ui::element::{UIElementEntity, UIState};

#[derive(Default)]
pub struct PluginUIButton;

impl Plugin for PluginUIButton {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            startup_spawn_buttons.run_if(in_state(UIState::Loaded).and_then(run_once)),
        );
        app.add_systems(Update, update_button.run_if(in_state(UIState::Loaded)));
        app.add_observer(on_command_spawn_button);
        app.add_observer(on_command_despawn_button);
    }
}

#[derive(Event)]
pub struct CommandSpawnButton {
    pub entity: Option<Entity>,
    pub hash: u32,
}

#[derive(EntityEvent)]
pub struct CommandDespawnButton {
    pub entity: Entity,
}

fn startup_spawn_buttons(
    mut commands: Commands,
    res_ui_handles: Res<LOLUiHandles>,
    button_assets: Res<Assets<LOLUiElementGroupButtonData>>,
) {
    for (&hash, handle) in res_ui_handles.button_handles.iter() {
        let Some(button_data) = button_assets.get(handle) else {
            continue;
        };
        let is_enabled = button_data.is_enabled.unwrap_or(false);
        if !is_enabled {
            continue;
        }

        commands.trigger(CommandSpawnButton { hash, entity: None });
    }
}

fn on_command_spawn_button(
    trigger: On<CommandSpawnButton>,
    mut commands: Commands,
    res_ui_handles: Res<LOLUiHandles>,
    button_assets: Res<Assets<LOLUiElementGroupButtonData>>,
) {
    let hash = trigger.hash;
    let Some(button_handle) = res_ui_handles.button_handles.get(&hash) else {
        return;
    };
    let Some(button_data) = button_assets.get(button_handle) else {
        return;
    };

    let Some(icon_handle) = res_ui_handles.icon_handles.get(&button_data.hit_region_element) else {
        warn!("未找到按钮 {} 的 hit_region_element {}", button_data.name, button_data.hit_region_element);
        return;
    };

    let bundle = (
        UIElement::Handle(icon_handle.clone()),
        UIButton(button_handle.clone()),
        Pickable::default(),
    );

    if let Some(entity) = trigger.entity {
        commands.entity(entity).insert(bundle);
    } else {
        commands.spawn(bundle);
    }
}

fn on_command_despawn_button(
    trigger: On<CommandDespawnButton>,
    mut commands: Commands,
    q_ui_button: Query<&UIButton>,
    button_assets: Res<Assets<LOLUiElementGroupButtonData>>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    let Ok(button) = q_ui_button.get(trigger.entity) else {
        return;
    };

    let Some(button_data) = button_assets.get(&button.0) else {
        return;
    };

    for element in button_data.elements.iter() {
        let Some(&element_entity) = res_ui_element_entity.map.get(element) else {
            continue;
        };
        commands.entity(element_entity).insert(Visibility::Hidden);
    }

    commands.entity(trigger.entity).despawn();
}

fn update_button(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &UIButton), Changed<Interaction>>,
    button_assets: Res<Assets<LOLUiElementGroupButtonData>>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    for (interaction, button) in &mut interaction_query {
        let Some(button_data) = button_assets.get(&button.0) else {
            continue;
        };

        let interaction_entity = match *interaction {
            Interaction::Pressed => {
                debug!("按下 {}", button_data.name);
                let Some(&clicked_entity) = button_data
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
                debug!("进入 {}", button_data.name);
                let Some(&hover_entity) = button_data
                    .hover_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                else {
                    continue;
                };
                hover_entity
            }
            Interaction::None => {
                debug!("离开 {}", button_data.name);
                let Some(&default_entity) = button_data
                    .default_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                else {
                    continue;
                };
                default_entity
            }
        };

        for element in button_data.elements.iter() {
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
