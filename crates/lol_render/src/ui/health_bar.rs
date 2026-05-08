use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_base::ui::{
    LOLEnumUiPosition, LOLFloatingInfoBarViewController, LOLHeroFloatingInfoBarData,
    LOLStructureFloatingInfoBarData, LOLUiElementIconData, LOLUiElementRegionData,
    LOLUnitFloatingInfoBarData, LOLUiHandles,
};
use lol_base::ui_components::{HealthBind, UIBind, UIElement};
use lol_core::base::bounding::Bounding;
use lol_core::life::Health;

use crate::ui::element::UIState;

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

fn update_spawn_health_bar(
    mut commands: Commands,
    q_added_health_bar: Query<(Entity, &HealthBar, &Bounding), Added<Bounding>>,
    res_assets_floating_info_bar_view_controller: Res<Assets<LOLFloatingInfoBarViewController>>,
    res_assets_unit_floating_info_bar_data: Res<Assets<LOLUnitFloatingInfoBarData>>,
    res_assets_hero_floating_info_bar_data: Res<Assets<LOLHeroFloatingInfoBarData>>,
    res_assets_structure_floating_info_bar_data: Res<Assets<LOLStructureFloatingInfoBarData>>,
    res_assets_ui_element_region_data: Res<Assets<LOLUiElementRegionData>>,
    res_assets_ui_element_icon_data: Res<Assets<LOLUiElementIconData>>,
    res_ui_handles: Res<LOLUiHandles>,
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
            let Some(handle) = res_ui_handles.icon_handles.get(hash) else {
                warn!("未找到健康条边界图标 handle: {}", hash);
                return;
            };
            let ui_element_data = res_assets_ui_element_icon_data.get(handle).unwrap();
            let ui_element_entity = commands
                .spawn((
                    ZIndex(ui_element_data.layer.unwrap_or(0) as i32),
                    UIElement::Handle(handle.clone()),
                    Visibility::Visible,
                ))
                .id();
            commands
                .entity(health_bar_entity)
                .add_child(ui_element_entity);
        };

        let apply_anchor = |commands: &mut Commands, anchor_hash: &u32| {
            let anchor_region = res_assets_ui_element_region_data
                .load_hash(anchor_hash)
                .unwrap();

            if let Some(LOLEnumUiPosition::UiPositionRect(anchor_position)) =
                &anchor_region.position
            {
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

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
pub struct HealthBar {
    pub bar_type: u8,
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
    }
}
