use bevy::prelude::*;
use lol_base::hash_key::LoadHashKeyTrait;
use lol_base::ui::{LOLUiElementGroupButtonData, LOLUiElementGroupButtonState};
use lol_base::ui_components::{UIButton, UIButtonEntities, UIElement};

use crate::ui::element::{UIElementEntity, UIState};

#[derive(Default)]
pub struct PluginUIButton;

impl Plugin for PluginUIButton {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_button_entities, update_button, update_visibility)
                .chain()
                .run_if(in_state(UIState::Loaded)),
        );
    }
}

fn update_visibility(
    mut commands: Commands,
    q_ui_button: Query<(&UIButtonEntities, &InheritedVisibility), Changed<InheritedVisibility>>,
) {
    for (entities, visibility) in q_ui_button.iter() {
        if entities.default.is_none() {
            continue;
        }
        if visibility.get() {
            commands
                .entity(entities.default.unwrap())
                .insert(Visibility::Inherited);
            continue;
        }
        for &element_entity in &entities.all {
            commands.entity(element_entity).insert(Visibility::Hidden);
        }
    }
}

pub fn update_button(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &UIButtonEntities), Changed<Interaction>>,
) {
    for (interaction, entities) in &mut interaction_query {
        let interaction_entity = match *interaction {
            Interaction::Pressed => entities.clicked,
            Interaction::Hovered => entities.hover,
            Interaction::None => entities.default,
        };

        let Some(interaction_entity) = interaction_entity else {
            continue;
        };

        for &element_entity in &entities.all {
            commands.entity(element_entity).insert(Visibility::Hidden);
        }

        commands
            .entity(interaction_entity)
            .insert(Visibility::Inherited);
    }
}

fn update_button_entities(
    mut commands: Commands,
    q_button: Query<(Entity, &UIButton), Without<UIButtonEntities>>,
    button_assets: Res<Assets<LOLUiElementGroupButtonData>>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    for (entity, button) in &q_button {
        let Some(button_data) = button_assets.load_hash(button.0) else {
            warn!("未找到按钮 {:?} 的 UIButtonData", entity);
            continue;
        };

        let get_state = |state: &Option<LOLUiElementGroupButtonState>| {
            let hash = state.as_ref()?.display_element_list.as_ref()?.get(0)?;
            res_ui_element_entity.map.get(hash).copied()
        };

        let default = get_state(&button_data.default_state_elements);
        let hover = get_state(&button_data.hover_state_elements);
        let clicked = get_state(&button_data.clicked_state_elements);

        let mut all = Vec::new();
        if let Some(e) = default {
            all.push(e);
        }
        if let Some(e) = hover {
            all.push(e);
        }
        if let Some(e) = clicked {
            all.push(e);
        }

        // 初始隐藏所有状态
        for &e in &all {
            commands.entity(e).insert(Visibility::Hidden);
        }

        // 默认显示 default 状态
        if let Some(e) = default {
            commands.entity(e).insert(Visibility::Inherited);
        }

        commands.entity(entity).insert((
            UIElement::Region(button_data.hit_region_element),
            Pickable::default(),
            UIButtonEntities {
                default,
                hover,
                clicked,
                all,
            },
        ));
    }
}
