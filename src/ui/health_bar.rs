use bevy::prelude::*;
use league_core::{
    EnumUiPosition, FloatingInfoBarViewController, HeroFloatingInfoBarData,
    StructureFloatingInfoBarData, UiElementIconData, UiElementRegionData, UnitFloatingInfoBarData,
};
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::{spawn_ui_element, Bounding, Health, UIBind, UIState};

pub struct PluginUIHealthBar;

impl Plugin for PluginUIHealthBar {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_spawn_health_bar.run_if(in_state(UIState::Loaded)),
        );
        app.add_systems(Update, update_health);
    }
}

#[derive(Component)]
pub struct HealthBind(pub Entity);

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
pub struct HealthBar {
    pub bar_type: u8,
}

fn update_spawn_health_bar(
    mut commands: Commands,
    q_added_health_bar: Query<(Entity, &HealthBar, &Bounding), Added<Bounding>>,
    res_assets_floating_info_bar_view_controller: Res<Assets<FloatingInfoBarViewController>>,
    res_assets_unit_floating_info_bar_data: Res<Assets<UnitFloatingInfoBarData>>,
    res_assets_hero_floating_info_bar_data: Res<Assets<HeroFloatingInfoBarData>>,
    res_assets_structure_floating_info_bar_data: Res<Assets<StructureFloatingInfoBarData>>,
    res_assets_ui_element_region_data: Res<Assets<UiElementRegionData>>,
    res_assets_ui_element_icon_data: Res<Assets<UiElementIconData>>,
    res_asset_server: Res<AssetServer>,
) {
    let controller = res_assets_floating_info_bar_view_controller
        .iter()
        .next()
        .unwrap()
        .1;

    for (entity, health_bar, bounding) in q_added_health_bar.iter() {
        let health_bar_entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ))
            .id();

        let bar_controller = controller
            .info_bar_style_source_map
            .get(&health_bar.bar_type)
            .unwrap();

        let spawn_and_attach = |commands: &mut Commands, hash: &u32| {
            let ui_element_data = res_assets_ui_element_icon_data.load_hash(hash).unwrap();
            if let Some(ui_element_entity) =
                spawn_ui_element(commands, &res_asset_server, ui_element_data)
            {
                commands
                    .entity(ui_element_entity)
                    .insert(Visibility::Visible);
                commands
                    .entity(health_bar_entity)
                    .add_child(ui_element_entity);
            }
        };

        let apply_anchor = |commands: &mut Commands, anchor_hash: &u32| {
            let anchor_region = res_assets_ui_element_region_data
                .load_hash(anchor_hash)
                .unwrap();

            if let Some(EnumUiPosition::UiPositionRect(anchor_position)) = &anchor_region.position {
                let anchor_ui_rect = anchor_position.ui_rect.as_ref().unwrap();
                commands.entity(health_bar_entity).insert(UIBind {
                    entity,
                    position: Vec3::ZERO.with_y(bounding.height),
                    offset: Vec2::ZERO,
                    anchor: anchor_ui_rect.position.clone().unwrap() + Vec2::new(50.0, 50.0),
                });
            }
        };

        match health_bar.bar_type {
            0 | 1 | 2 | 5 | 6 | 7 | 9 => {
                let bar_data = res_assets_unit_floating_info_bar_data
                    .load_hash(bar_controller)
                    .unwrap();
                apply_anchor(&mut commands, &bar_data.anchor);
                spawn_and_attach(&mut commands, &bar_data.border);
            }
            3 | 4 | 8 => {
                let bar_data = res_assets_structure_floating_info_bar_data
                    .load_hash(bar_controller)
                    .unwrap();
                apply_anchor(&mut commands, &bar_data.anchor);
                spawn_and_attach(&mut commands, &bar_data.border);
            }
            10 | 12 => {
                let bar_data = res_assets_hero_floating_info_bar_data
                    .load_hash(bar_controller)
                    .unwrap();

                let green_bar = bar_data
                    .health_bar
                    .health_bar
                    .additional_bar_types
                    .as_ref()
                    .unwrap()
                    .get(&hash_bin("green"))
                    .unwrap();

                apply_anchor(&mut commands, &bar_data.anchor);

                spawn_and_attach(&mut commands, &bar_data.borders.default_border.border);
                spawn_and_attach(&mut commands, green_bar);
            }
            _ => {}
        }
    }
}

fn update_health(
    mut q_health_bind: Query<(Entity, &mut Node, &HealthBind)>,
    q_health: Query<(&Health, &HealthBar)>,
) {
    for (_entity, mut node, health_bind) in q_health_bind.iter_mut() {
        let Ok((health, _health_bar)) = q_health.get(health_bind.0) else {
            continue;
        };

        node.width = Val::Percent(health.value / health.max * 100.0);

        // 英雄血条需要生成每 100 点血的标记
        // if health_bar.bar_type == HealthBarType::Champion {
        //     commands.entity(entity).despawn_children();
        //     commands.entity(entity).with_children(|parent| {
        //         let health_indicator_num = (health.value / 100.0) as usize;
        //         let health_bar_width = ((100.0 / health.max) * 104.0).floor();
        //         for i in 0..health_indicator_num {
        //             parent.spawn((
        //                 Node {
        //                     width: Val::Px(1.0),
        //                     height: Val::Px(6.0),
        //                     left: Val::Px(health_bar_width * (i + 1) as f32),
        //                     position_type: PositionType::Absolute,
        //                     ..default()
        //                 },
        //                 BackgroundColor(Color::BLACK),
        //             ));
        //         }
        //     });
        // }
    }
}
