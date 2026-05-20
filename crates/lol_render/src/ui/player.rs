use bevy::prelude::*;
use lol_base::character::Skin;
use lol_base::ui::{LOLPlayerFrameViewController, LOLPlayerInventoryViewController};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::gold::Gold;
use lol_core::base::level::Level;
use lol_core::life::Health;

use crate::controller::Controller;
use crate::skin::skin::SkinReady;
use crate::ui::element::{CommandUpdateUIElement, NodeType, SizeType, UIElementEntity, UIState};
use crate::ui::text::UiTextState;

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
                    any_match_filter::<(With<Controller>, With<SkinReady>)>.and_then(run_once),
                ),
                update_stat_page_buttons.run_if(run_once),
                update_gold,
            )
                .run_if(in_state(UIState::Loaded)),
        );
    }
}

fn update_level(
    mut commands: Commands,
    res_player_frame: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_level: Query<&Level, With<Controller>>,
) {
    let entity = res_ui_element_entity.get_entity(&res_player_frame.resource_bars.experience_bar);

    let Ok(level) = q_level.single() else {
        return;
    };

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Height,
        value: level.experience as f32 / level.experience_to_next_level as f32,
        node_type: NodeType::Parent,
        flip: true,
    });
}

fn update_player_health(
    mut commands: Commands,
    res_player_frame: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_health: Query<&Health, With<Controller>>,
) {
    let value = if let Ok(health) = q_health.single() {
        health.value as f32 / health.max as f32
    } else {
        0.0
    };

    // Update Green Bar (Immediate) using health_animated_meter_skin
    let Some(first_bar) = res_player_frame
        .resource_bars
        .health_animated_meter_skin
        .bar_elements
        .first()
    else {
        return;
    };
    let entity = res_ui_element_entity.get_entity(first_bar);

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Width,
        value,
        node_type: NodeType::Parent,
        flip: false,
    });
}

fn update_player_health_fade(
    mut commands: Commands,
    time: Res<Time>,
    res_player_frame: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_health: Query<&Health, With<Controller>>,
    mut q_health_fade: Query<&mut HealthFade>,
) {
    let health_data = q_health.single().ok();

    // Update Fade Bar (Animated)
    let entity =
        res_ui_element_entity.get_entity(&res_player_frame.resource_bars.health_meter.fade_bar);

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
        flip: false,
    });
}

fn update_player_ability_resource(
    mut commands: Commands,
    res_player_frame: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_ability_resource: Query<&AbilityResource, With<Controller>>,
) {
    let lol_base::ui::LOLEnumResourceMeter::ResourceMeterGroupData(ref group_data) =
        res_player_frame
            .resource_bars
            .par_meter_data
            .ability_resource_bars
    else {
        return;
    };
    let Some(first_bar_element) = group_data
        .meter_skins
        .default_meter_skin
        .bar_elements
        .first()
    else {
        return;
    };
    let entity = res_ui_element_entity.get_entity(first_bar_element);

    let Ok(ability_resource) = q_ability_resource.single() else {
        return;
    };

    commands.entity(entity).insert(Visibility::Visible);

    commands.trigger(CommandUpdateUIElement {
        entity,
        size_type: SizeType::Width,
        value: ability_resource.value as f32 / ability_resource.max as f32,
        node_type: NodeType::Parent,
        flip: false,
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

fn update_stat_page_buttons(
    res_player_frame: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    mut commands: Commands,
) {
    for stat_page in &res_player_frame.stat_pages {
        let entity = res_ui_element_entity.get_entity(&stat_page.button);

        commands.entity(entity).insert(Visibility::Visible);
    }
}

fn update_gold(
    res_player_inventory: Option<Res<LOLPlayerInventoryViewController>>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_gold: Query<&Gold, With<Controller>>,
    mut q_ui_text_state: Query<(&mut UiTextState, &mut Visibility)>,
    mut q_visibility: Query<&mut Visibility, Without<UiTextState>>,
) {
    let Some(res_player_inventory) = res_player_inventory else {
        return;
    };

    let Ok(gold) = q_gold.single() else {
        return;
    };

    // 1. 显示商店按钮实体
    let shop_btn_entity =
        res_ui_element_entity.get_entity(&res_player_inventory.shop_button.shop_button);
    if let Ok(mut vis) = q_visibility.get_mut(shop_btn_entity) {
        if *vis != Visibility::Visible {
            *vis = Visibility::Visible;
        }
    }

    // 2. 显示不活跃图标实体
    let inactive_icon_entity =
        res_ui_element_entity.get_entity(&res_player_inventory.shop_button.inactive_icon);
    if let Ok(mut vis) = q_visibility.get_mut(inactive_icon_entity) {
        if *vis != Visibility::Visible {
            *vis = Visibility::Visible;
        }
    }

    // 3. 显示并更新金币文本
    let text_entity = res_ui_element_entity.get_entity(&res_player_inventory.shop_button.text_link);
    let Ok((mut text_state, mut visibility)) = q_ui_text_state.get_mut(text_entity) else {
        return;
    };

    if *visibility != Visibility::Visible {
        *visibility = Visibility::Visible;
    }

    let gold_str = (gold.current as u32).to_string();
    if text_state.text != gold_str {
        text_state.text = gold_str;
    }
}
