mod init;
pub mod layout;
mod tree;

use std::collections::HashMap;

use bevy::prelude::*;
pub use init::{
    AnimAssets, ButtonAssets, DesaturateAssets, IconAssets, RegionAssets, SceneAssets,
    UiFileAssets, UiLoadProgress, poll_ui_load, startup_load_ui_data,
};
use lol_base::hash::hash_bin;
use lol_base::hash_key::HashKey;
use lol_base_render::ui::{
    LOLHeroFloatingInfoBarData, LOLStructureFloatingInfoBarData, LOLUiElementEffectAnimationData,
    LOLUiElementEffectDesaturateData, LOLUiElementEffectFillPercentageData,
    LOLUiElementEffectInstancedData, LOLUiElementGroupButtonData, LOLUiElementIconData,
    LOLUiElementRegionData, LOLUiElementTextData, LOLUiFile, LOLUiSceneData,
    LOLUnitFloatingInfoBarData,
};
use lol_base_render::ui_components::UIButton;
pub use lol_base_render::ui_components::UIElement;
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
        app.init_asset::<LOLUiFile>();
        app.init_asset_loader::<crate::loaders::ui::UiFileLoader>();
        app.init_resource::<UiLoadProgress>();

        app.init_resource::<lol_base_render::ui::LOLPlayerFrameViewController>();
        app.init_resource::<lol_base_render::ui::LOLFloatingInfoBarViewController>();
        app.init_resource::<lol_base_render::ui::LOLPlayerInventoryViewController>();
        app.init_resource::<lol_base_render::ui::LOLLolGameStateViewController>();

        app.register_type::<lol_base_render::ui::LOLPlayerFrameViewController>();
        app.register_type::<lol_base_render::ui::LOLFloatingInfoBarViewController>();
        app.register_type::<lol_base_render::ui::LOLPlayerInventoryViewController>();
        app.register_type::<lol_base_render::ui::LOLLolGameStateViewController>();

        app.register_type::<lol_base_render::ui::LOLAbilitiesUiData>();
        app.register_type::<lol_base_render::ui::LOLSpellPipsUiData>();
        app.register_type::<lol_base_render::ui::LOLSpellRankPipsUiData>();
        app.register_type::<lol_base_render::ui::LOLSpellSlotDetailedUiDefinition>();
        app.register_type::<lol_base_render::ui::LOLSpellSlotBuffTimerData>();
        app.register_type::<lol_base_render::ui::LOLCooldownGemUiData>();
        app.register_type::<lol_base_render::ui::LOLCooldownEffectUiData>();
        app.register_type::<lol_base_render::ui::LOLPlayerPortraitUiData>();
        app.register_type::<lol_base_render::ui::LOLHudAbilityResourceThresholdIndicator>();
        app.register_type::<lol_base_render::ui::LOLUiElementMeterSkin>();
        app.register_type::<lol_base_render::ui::LOLHealthMeter>();
        app.register_type::<lol_base_render::ui::LOLAbilityResourceBarData>();
        app.register_type::<lol_base_render::ui::LOLEnumResourceMeter>();
        app.register_type::<lol_base_render::ui::LOLResourceMeterGroupData>();
        app.register_type::<lol_base_render::ui::LOLResourceMeterSkinData>();
        app.register_type::<lol_base_render::ui::LOLResourceMeterIconData>();
        app.register_type::<lol_base_render::ui::LOLHudPlayerResourceBars>();
        app.register_type::<lol_base_render::ui::LOLUiLevelUp>();
        app.register_type::<lol_base_render::ui::LOLSpellLevelUpUiData>();
        app.register_type::<lol_base_render::ui::LOLStatPageViewController>();
        app.register_type::<lol_base_render::ui::LOLStatPageCategoryData>();
        app.register_type::<lol_base_render::ui::LOLItemSlotDetailedUiData>();
        app.register_type::<lol_base_render::ui::LOLHudShopButton>();
        app.register_type::<lol_base_render::ui::LOLDrawAreaList>();
        app.register_type::<lol_base_render::ui::LOLEnumUiMetric>();
        app.register_type::<lol_base_render::ui::LOLUiClashTeam>();
        app.register_type::<lol_base_render::ui::LOLUiMetricClash>();
        app.register_type::<lol_base_render::ui::LOLUiMetricCreepScore>();
        app.register_type::<lol_base_render::ui::LOLUiMetricFps>();
        app.register_type::<lol_base_render::ui::LOLUiMetricGameTime>();
        app.register_type::<lol_base_render::ui::LOLUiMetricKda>();
        app.register_type::<lol_base_render::ui::LOLUiMetricLatencyText>();
        app.register_type::<lol_base_render::ui::LOLUiMetricTeamKills>();
        app.register_type::<lol_base_render::ui::LOLUiMetricTeamScoreMeters>();
        app.register_type::<lol_base_render::ui::LOLUnk0x5ab5b20f>();
        app.register_type::<lol_base_render::ui::LOLUnk0x767adcf7>();
        app.register_type::<lol_base_render::ui::LOLUnk0xa8c6f5f0>();
        app.register_type::<lol_base_render::ui::LOLUnk0x7a19656>();
        app.register_type::<lol_base_render::ui::LOLUnk0xb8a49c96>();
        app.register_type::<lol_base_render::ui::LOLUnk0xf43ad1ce>();
        app.register_type::<lol_base_render::ui::LOLUnk0xb62c8675>();
        app.register_type::<lol_base_render::ui::LOLUnk0xe228ce4a>();
        app.register_type::<lol_base_render::ui::LOLEnumUiPosition>();
        app.register_type::<lol_base_render::ui::LOLUiPositionPolygon>();
        app.register_type::<lol_base_render::ui::LOLUiPositionRect>();
        app.register_type::<lol_base_render::ui::LOLUiElementRect>();
        app.register_type::<lol_base_render::ui::LOLEnumAnchor>();
        app.register_type::<lol_base_render::ui::LOLAnchorDouble>();
        app.register_type::<lol_base_render::ui::LOLAnchorSingle>();
        app.register_type::<lol_base_render::ui::LOLEnumData>();
        app.register_type::<lol_base_render::ui::LOLAtlasData3SliceH>();
        app.register_type::<lol_base_render::ui::LOLAtlasData3SliceV>();
        app.register_type::<lol_base_render::ui::LOLAtlasData9Slice>();
        app.register_type::<lol_base_render::ui::LOLLooseUiTextureData3SliceH>();
        app.register_type::<lol_base_render::ui::LOLLooseUiTextureData3SliceV>();
        app.register_type::<lol_base_render::ui::LOLLooseUiTextureData9Slice>();
        app.register_type::<lol_base_render::ui::LOLAtlasData>();
        app.register_type::<lol_base_render::ui::LOLLooseUiTextureData>();
        app.register_type::<lol_base_render::ui::LOLHeroFloatingInfoBarData>();
        app.register_type::<lol_base_render::ui::LOLHeroFloatingInfoBorderData>();
        app.register_type::<lol_base_render::ui::LOLHeroFloatingInfoBorderDefenseIconData>();
        app.register_type::<lol_base_render::ui::LOLHeroFloatingInfoBorderDefenseIconThresholdData>();
        app.register_type::<lol_base_render::ui::LOLHeroFloatingInfoBorderTypeData>();
        app.register_type::<lol_base_render::ui::LOLHealthBarData>();
        app.register_type::<lol_base_render::ui::LOLHealthBarExtraBarsData>();
        app.register_type::<lol_base_render::ui::LOLHealthBarFadeData>();
        app.register_type::<lol_base_render::ui::LOLHealthBarTextData>();
        app.register_type::<lol_base_render::ui::LOLEnumHealthBarTickStyle>();
        app.register_type::<lol_base_render::ui::LOLHealthBarTickStyleHero>();
        app.register_type::<lol_base_render::ui::LOLHealthBarTickStyleTftCompanion>();
        app.register_type::<lol_base_render::ui::LOLHealthBarTickStyleUnit>();
        app.register_type::<lol_base_render::ui::LOLMicroTicksPerStandardTickData>();
        app.register_type::<lol_base_render::ui::LOLBarTypeMap>();
        app.register_type::<lol_base_render::ui::LOLStructureFloatingInfoBarData>();
        app.register_type::<lol_base_render::ui::LOLUnitFloatingInfoBarData>();
        app.register_type::<lol_base_render::ui::LOLUiElementIconData>();
        app.register_type::<lol_base_render::ui::LOLUiElementRegionData>();
        app.register_type::<lol_base_render::ui::LOLUiElementGroupButtonData>();
        app.register_type::<lol_base_render::ui::LOLUiElementGroupButtonState>();
        app.register_type::<lol_base_render::ui::LOLUiElementEffectAnimationData>();
        app.register_type::<lol_base_render::ui::LOLUiElementEffectDesaturateData>();
        app.register_type::<lol_base_render::ui::LOLUiElementEffectInstancedData>();
        app.register_type::<lol_base_render::ui::LOLUiElementEffectFillPercentageData>();
        app.register_type::<lol_base_render::ui::LOLUiElementTextData>();
        app.register_type::<lol_base_render::ui::LOLUiSceneData>();

        app.register_type::<HashKey<lol_base_render::ui::LOLUiSceneData>>();
        app.register_type::<HashKey<lol_base_render::ui::LOLUiElementIconData>>();
        app.register_type::<HashKey<lol_base_render::ui::LOLUiElementRegionData>>();
        app.register_type::<HashKey<lol_base_render::ui::LOLUiElementEffectDesaturateData>>();
        app.register_type::<HashKey<lol_base_render::ui::LOLUiElementGroupButtonData>>();
        app.register_type::<HashKey<lol_base_render::ui::LOLUiElementTextData>>();

        app.add_systems(Startup, startup_load_ui_data);
        app.add_systems(Update, poll_ui_load.run_if(in_state(UIState::Loading)));
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
        self.map.get(&key.0).copied().unwrap()
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
