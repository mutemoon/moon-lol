mod init;
pub mod layout;
mod tree;

use std::collections::HashMap;

use bevy::prelude::*;
pub use init::{
    AnimAssets, ButtonAssets, DesaturateAssets, IconAssets, RegionAssets, SceneAssets,
    startup_load_ui_data,
};
use lol_base::hash::hash_bin;
use lol_base::hash_key::HashKey;
use lol_base::ui::{
    LOLHeroFloatingInfoBarData, LOLStructureFloatingInfoBarData, LOLUiElementEffectAnimationData,
    LOLUiElementEffectDesaturateData, LOLUiElementEffectFillPercentageData,
    LOLUiElementEffectInstancedData, LOLUiElementGroupButtonData, LOLUiElementIconData,
    LOLUiElementRegionData, LOLUiElementTextData, LOLUiSceneData, LOLUnitFloatingInfoBarData,
};
use lol_base::ui_components::UIButton;
pub use lol_base::ui_components::UIElement;
pub use tree::save_ui_tree_to_json;

pub struct PluginUIElement;

impl Plugin for PluginUIElement {
    fn build(&self, app: &mut App) {
        app.init_state::<UIState>();
        app.init_resource::<UIElementEntity>();
        app.init_asset::<LOLUiSceneData>();
        app.init_asset::<LOLUiElementIconData>();
        app.init_asset::<LOLUiElementEffectAnimationData>();
        app.init_asset::<LOLUiElementEffectDesaturateData>();
        app.init_asset::<LOLUiElementEffectFillPercentageData>();
        app.init_asset::<LOLUiElementEffectInstancedData>();
        app.init_asset::<LOLUiElementGroupButtonData>();
        app.init_asset::<LOLUiElementRegionData>();
        app.init_asset::<LOLUiElementTextData>();
        app.init_asset::<LOLUnitFloatingInfoBarData>();
        app.init_asset::<LOLHeroFloatingInfoBarData>();
        app.init_asset::<LOLStructureFloatingInfoBarData>();

        app.add_systems(Startup, startup_load_ui_data);
        app.add_systems(
            Update,
            (
                layout::update_on_window_resized,
                layout::update_on_add_ui_element,
            )
                .run_if(in_state(UIState::Loaded)),
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
    pub flip: bool,
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

#[derive(Component, Debug, Clone, Copy)]
pub struct OriginalPosition(pub Vec2);

impl UIElementEntity {
    pub fn get_by_string(&self, key: &str) -> Option<&Entity> {
        self.map.get(&hash_bin(key))
    }

    pub fn get_entity<T: TypePath>(&self, key: &HashKey<T>) -> Entity {
        self.map.get(&key.0.0).copied().unwrap()
    }

    pub fn add(&mut self, key: u32, entity: Entity) {
        self.map.insert(key, entity);
    }

    pub fn get(&self, key: u32) -> Entity {
        self.map.get(&key).copied().unwrap()
    }
}

fn on_command_update_ui_element(
    trigger: On<CommandUpdateUIElement>,
    q_children: Query<&Children>,
    mut q_node: Query<&mut Node>,
    q_original_position: Query<&OriginalPosition>,
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

    let target_size = standard_size * value;
    {
        let Ok(mut target_node) = q_node.get_mut(target_entity) else {
            return;
        };
        match size_type {
            SizeType::Width => {
                target_node.width = Val::Px(target_size);
            }
            SizeType::Height => {
                target_node.height = Val::Px(target_size);
            }
        }
    }

    if trigger.flip {
        let original_top = q_original_position
            .get(entity)
            .map(|o| o.0.y)
            .unwrap_or(0.0);
        match size_type {
            SizeType::Width => {
                let Ok(mut child_node) = q_node.get_mut(children[0]) else {
                    return;
                };
                child_node.left = Val::Px(standard_size - target_size);
            }
            SizeType::Height => {
                {
                    let Ok(mut target_node) = q_node.get_mut(target_entity) else {
                        return;
                    };
                    target_node.top = Val::Px(original_top + standard_size - target_size);
                }
                {
                    let Ok(mut child_node) = q_node.get_mut(children[0]) else {
                        return;
                    };
                    child_node.top = Val::Px(-standard_size + target_size);
                }
            }
        }
    }
}
