use bevy::prelude::*;
use lol_base::character::Skin;
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::level::Level;
use lol_core::life::Health;

use crate::controller::Controller;
use crate::skin::skin::SkinReady;
use crate::ui::element::{CommandUpdateUIElement, NodeType, SizeType, UIElementEntity, UIState};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HealthFade {
    pub value: f32,
    pub max: f32,
}

#[derive(Default)]
pub struct PluginUIPlayer;

impl Plugin for PluginUIPlayer {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_level,
                update_player_health,
                update_player_health_fade,
                update_player_ability_resource,
                update_player_icon.run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<SkinReady>)>)
                        .and_then(run_once),
                ),
            ),
        );
    }
}

fn update_level(
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

fn update_player_health(
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

fn update_player_health_fade(
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

fn update_player_ability_resource(
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

fn update_player_icon(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    q_image_node: Query<&ImageNode>,
    q_skin: Query<&Skin, (With<Controller>, With<SkinReady>)>,
    q_children: Query<&Children>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    let key = "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/Player_Frame/PlayerIcon_Base";
    let Some(&entity) = res_ui_element_entity.get_by_string(key) else {
        info!("未找到玩家头像的父节点");
        return;
    };

    let Ok(skin) = q_skin.single() else {
        info!("未找到玩家的皮肤");
        return;
    };

    if skin.avatar.is_empty() {
        info!("玩家的皮肤为空");
        return;
    }

    let Ok(children) = q_children.get(entity) else {
        info!("未找到玩家头像的子节点");
        return;
    };
    let Some(&child) = children.get(0) else {
        info!("未找到玩家头像的子节点");
        return;
    };

    if q_image_node.get(child).is_ok() {
        info!("玩家头像已经加载");
        return;
    }

    commands.entity(child).insert((
        ImageNode {
            image: asset_server.load(&skin.avatar),
            ..default()
        },
        Visibility::Visible,
    ));
}
