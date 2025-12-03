use bevy::prelude::*;

use lol_config::ConfigUi;

use crate::{core::ui::element::UIElementEntity, UIElement};

#[derive(Component)]
#[require(Interaction)]
pub struct UIButton {
    pub key: u32,
}

pub fn startup_spawn_buttons(mut commands: Commands, res_config_ui: Res<ConfigUi>) {
    for (&key, ui_button_group) in res_config_ui.ui_button_group.iter().filter(|v| {
        v.1.name
            .contains("ClientStates/Gameplay/UX/LoL/PlayerFrame/")
    }) {
        let is_enabled = ui_button_group.is_enabled.unwrap_or(false);
        if !is_enabled {
            continue;
        }

        let hit_region = res_config_ui
            .ui_region
            .get(&ui_button_group.hit_region_element)
            .unwrap();
        commands.spawn((
            Node::default(),
            UIElement {
                key: ui_button_group.hit_region_element.to_string(),
                position: hit_region.position.clone().unwrap(),
                update_child: false,
            },
            UIButton { key },
        ));
    }
}

pub fn update_button(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &UIButton), Changed<Interaction>>,
    res_config_ui: Res<ConfigUi>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    for (interaction, button) in &mut interaction_query {
        let ui_button_group = res_config_ui.ui_button_group.get(&button.key).unwrap();

        let interaction_entity = match *interaction {
            Interaction::Pressed => {
                info!("按下 {}", ui_button_group.name);
                let Some(&clicked_entity) = ui_button_group
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
                info!("悬停 {}", ui_button_group.name);
                let &hover_entity = ui_button_group
                    .hover_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                    .unwrap();
                hover_entity
            }
            Interaction::None => {
                info!("恢复 {}", ui_button_group.name);
                let &default_entity = ui_button_group
                    .default_state_elements
                    .as_ref()
                    .and_then(|v| v.display_element_list.as_ref())
                    .and_then(|v| v.get(0))
                    .and_then(|v| res_ui_element_entity.map.get(v))
                    .unwrap();
                default_entity
            }
        };

        for element in ui_button_group.elements.iter() {
            let &element_entity = res_ui_element_entity.map.get(element).unwrap();
            commands.entity(element_entity).insert(Visibility::Hidden);
        }

        commands
            .entity(interaction_entity)
            .insert(Visibility::Visible);
    }
}
