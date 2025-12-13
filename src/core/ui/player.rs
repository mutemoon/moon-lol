use bevy::prelude::*;use lol_config::LoadHashKeyTrait;
use league_core::SkinCharacterDataProperties;

use crate::{
    AbilityResource, CommandUpdateUIElement, Controller, Health, Level, NodeType, SizeType, Skin,
    UIElementEntity,
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HealthFade {
    pub value: f32,
    pub max: f32,
}

pub fn update_level(
    mut commands: Commands,
    res_ui_element_entity: Res<UIElementEntity>,
    q_level: Query<&Level, With<Controller>>,
) {
    let Some(&entity) = res_ui_element_entity.get_by_string("ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/Player_Frame/PlayerExp_BarTexture") else {
        return;
    };

    let Ok(level) = q_level.single() else {
        return;
    };

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Height,
        value: level.experience as f32 / level.experience_to_next_level as f32,
        node_type: NodeType::Parent,
    });
}

pub fn update_player_health(
    mut commands: Commands,
    res_ui_element_entity: Res<UIElementEntity>,
    q_health: Query<&Health, With<Controller>>,
) {
    let value = if let Ok(health) = q_health.single() {
        health.value as f32 / health.max as f32
    } else {
        0.0
    };

    // Update Green Bar (Immediate)
    let key = "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerResourceBars/PlayerHPMeter/PlayerHP_BarTextureGreen";
    let Some(&entity) = res_ui_element_entity.get_by_string(key) else {
        return;
    };

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Width,
        value,
        node_type: NodeType::Parent,
    });
}

pub fn update_player_health_fade(
    mut commands: Commands,
    time: Res<Time>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_health: Query<&Health, With<Controller>>,
    mut q_health_fade: Query<&mut HealthFade>,
) {
    let health_data = q_health.single().ok();

    // Update Fade Bar (Animated)
    let key = "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerResourceBars/PlayerHPMeter/PlayerHP_BarFade";
    let Some(&entity) = res_ui_element_entity.get_by_string(key) else {
        return;
    };

    let mut final_value;
    let final_max;

    if let Ok(mut health_fade) = q_health_fade.get_mut(entity) {
        let (target_val, current_max) = if let Some(health) = health_data {
            health_fade.max = health.max;
            (health.value, health.max)
        } else {
            (0.0, health_fade.max)
        };

        if target_val < health_fade.value {
            health_fade.value -= 100.0 * time.delta_secs() * 2.0;
            if health_fade.value < target_val {
                health_fade.value = target_val;
            }
        } else {
            health_fade.value = target_val;
        }
        final_value = health_fade.value;
        final_max = current_max;
    } else {
        if let Some(health) = health_data {
            commands.entity(entity).insert(HealthFade {
                value: health.value,
                max: health.max,
            });
            final_value = health.value;
            final_max = health.max;
        } else {
            commands.entity(entity).insert(HealthFade {
                value: 0.0,
                max: 1.0,
            });
            final_value = 0.0;
            final_max = 1.0;
        }
    }

    if final_max == 0.0 {
        final_value = 0.0;
    } else {
        final_value /= final_max;
    }

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Width,
        value: final_value,
        node_type: NodeType::Parent,
    });
}

pub fn update_player_ability_resource(
    mut commands: Commands,
    res_ui_element_entity: Res<UIElementEntity>,
    q_ability_resource: Query<&AbilityResource, With<Controller>>,
) {
    let key = "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerResourceBars/PlayerPARMeter/PlayerPar_BarTextureBlue";
    let Some(&entity) = res_ui_element_entity.get_by_string(key) else {
        return;
    };

    let Ok(ability_resource) = q_ability_resource.single() else {
        return;
    };

    commands.entity(entity).insert(Visibility::Visible);

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Width,
        value: ability_resource.value as f32 / ability_resource.max as f32,
        node_type: NodeType::Parent,
    });
}

pub fn update_player_icon(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut q_image_node: Query<&mut ImageNode>,
    q_skin: Query<&Skin, With<Controller>>,
    q_children: Query<&Children>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
        res_ui_element_entity: Res<UIElementEntity>,
) {
    let key = "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/Player_Frame/PlayerIcon_Base";
    let Some(&entity) = res_ui_element_entity.get_by_string(key) else {
        return;
    };

    let Ok(skin) = q_skin.single() else {
        return;
    };

    let skin = res_assets_skin_character_data_properties.load_hash( skin.key)
        .unwrap();

    let icon_name = skin
        .icon_avatar
        .clone()
        .unwrap_or(skin.icon_circle.clone().unwrap());

    let &child = q_children.get(entity).unwrap().get(0).unwrap();
    if q_image_node.get_mut(child).is_ok() {
        return;
    }

    commands.entity(entity).insert((
        ImageNode {
            image: asset_server.load(&format!("{}#srgb", icon_name)),
            ..default()
        },
        Visibility::Visible,
    ));
}
