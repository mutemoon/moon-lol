use std::collections::HashMap;

use bevy::prelude::*;
use league_utils::hash_bin;
use serde::{Deserialize, Serialize};

use league_core::{
    UiElementEffectAnimationData, UiElementGroupButtonData, UiElementIconData, UiElementRegionData,
};

#[derive(Resource, Clone, Default, Serialize, Deserialize)]
pub struct ConfigUi {
    pub ui_elements: HashMap<u32, UiElementIconData>,
    pub ui_animations: HashMap<u32, UiElementEffectAnimationData>,
    pub ui_button_group: HashMap<u32, UiElementGroupButtonData>,
    pub ui_region: HashMap<u32, UiElementRegionData>,
}

impl ConfigUi {
    pub fn get_ui_element_by_path(&self, path: &str) -> Option<&UiElementIconData> {
        self.ui_elements.get(&hash_bin(path))
    }

    pub fn get_ui_animation_by_path(&self, path: &str) -> Option<&UiElementEffectAnimationData> {
        self.ui_animations.get(&hash_bin(path))
    }

    pub fn get_ui_button_group_by_path(&self, path: &str) -> Option<&UiElementGroupButtonData> {
        self.ui_button_group.get(&hash_bin(path))
    }
}
